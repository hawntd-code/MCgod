//! Ejecución de plantillas.

use crate::ast::{Args, AssignOp, BinOp, Body, Callable, Expr, Node, RangeKind, Scope, StrPart};
use crate::value::{Map, Value, format_number};
use crate::{Engine, FtlError};
use std::rc::Rc;

pub(crate) struct Loop {
    pub vars: Vec<String>,
    pub values: Vec<Value>,
    pub index: usize,
    pub has_next: bool,
}

pub(crate) struct Frame {
    pub locals: Map,
    pub loops: Vec<Loop>,
    pub ns: usize,
    /// Cuerpo pasado a la macro y marco de quien la llamó (para `<#nested>`).
    pub nested: Option<(Body, usize)>,
    pub template: Rc<str>,
    pub is_macro: bool,
}

pub(crate) enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

pub(crate) struct Run<'e> {
    pub engine: &'e Engine,
    pub data: &'e Map,
    pub globals: Map,
    pub namespaces: Vec<Map>,
    pub frames: Vec<Frame>,
    pub cur: usize,
    pub out: Vec<String>,
    pub depth: usize,
}

const MAX_DEPTH: usize = 200;

pub(crate) fn missing(what: impl Into<String>) -> FtlError {
    FtlError::Missing(what.into())
}

pub(crate) fn fail(what: impl Into<String>) -> FtlError {
    FtlError::Eval(what.into())
}

impl<'e> Run<'e> {
    pub fn new(engine: &'e Engine, data: &'e Map, template: Rc<str>) -> Run<'e> {
        Run {
            engine,
            data,
            globals: Map::new(),
            namespaces: vec![Map::new()],
            frames: vec![Frame {
                locals: Map::new(),
                loops: Vec::new(),
                ns: 0,
                nested: None,
                template,
                is_macro: false,
            }],
            cur: 0,
            out: vec![String::new()],
            depth: 0,
        }
    }

    fn frame(&self) -> &Frame {
        &self.frames[self.cur]
    }

    fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frames[self.cur]
    }

    fn write(&mut self, s: &str) {
        self.out.last_mut().expect("salida").push_str(s);
    }

    pub fn template_name(&self) -> Rc<str> {
        self.frame().template.clone()
    }

    // --- Variables -------------------------------------------------------------

    pub fn lookup(&self, name: &str) -> Value {
        let f = self.frame();
        for l in f.loops.iter().rev() {
            if let Some(i) = l.vars.iter().position(|v| v == name) {
                return l.values[i].clone();
            }
        }
        if let Some(v) = f.locals.get(name) {
            return v.clone();
        }
        if let Some(v) = self.namespaces[f.ns].get(name) {
            return v.clone();
        }
        if let Some(v) = self.globals.get(name) {
            return v.clone();
        }
        if let Some(v) = self.data.get(name) {
            return v.clone();
        }
        self.engine.shared.get(name).cloned().unwrap_or(Value::Null)
    }

    pub fn find_loop(&self, name: &str) -> Option<&Loop> {
        self.frame()
            .loops
            .iter()
            .rev()
            .find(|l| l.vars.iter().any(|v| v == name))
    }

    fn set(&mut self, scope: Scope, name: &str, value: Value) {
        match scope {
            Scope::Local if self.frame().is_macro => {
                self.frame_mut().locals.insert(name.to_string(), value);
            }
            Scope::Global => {
                self.globals.insert(name.to_string(), value);
            }
            _ => {
                let ns = self.frame().ns;
                self.namespaces[ns].insert(name.to_string(), value);
            }
        }
    }

    // --- Ejecución de nodos -------------------------------------------------------

    /// Define las macros y funciones de una plantilla antes de ejecutarla, como FreeMarker.
    pub fn hoist(&mut self, body: &Body) {
        let ns = self.frame().ns;
        for n in body.iter() {
            if let Node::MacroDef(c) = n {
                self.namespaces[ns].insert(c.name.clone(), Value::Macro(c.clone(), ns));
            }
        }
    }

    pub fn exec(&mut self, body: &[Node]) -> Result<Flow, FtlError> {
        for node in body {
            let flow = self.exec_node(node)?;
            if !matches!(flow, Flow::Normal) {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }

    fn located(&self, e: FtlError, line: u32) -> FtlError {
        e.at(&self.frame().template, line)
    }

    fn exec_node(&mut self, node: &Node) -> Result<Flow, FtlError> {
        match node {
            Node::Text(s) => self.write(s),
            Node::Interp(e, line) => {
                let v = self.eval(e).map_err(|err| self.located(err, *line))?;
                let s = self.display(&v).map_err(|err| self.located(err, *line))?;
                self.write(&s);
            }
            Node::If(branches, else_body, line) => {
                for (cond, body) in branches {
                    let c = self
                        .eval(cond)
                        .and_then(|v| self.truthy(&v))
                        .map_err(|e| self.located(e, *line))?;
                    if c {
                        return self.exec(body);
                    }
                }
                if let Some(b) = else_body {
                    return self.exec(b);
                }
            }
            Node::List {
                seq,
                vars,
                body,
                else_body,
                line,
            } => return self.exec_list(seq, vars, body, else_body.as_ref(), *line),
            Node::Sep(body) => {
                if self.frame().loops.last().is_some_and(|l| l.has_next) {
                    return self.exec(body);
                }
            }
            Node::Assign { scope, items, line } => {
                for (name, op, value) in items {
                    let new = self
                        .assign_value(name, *op, value.as_ref())
                        .map_err(|e| self.located(e, *line))?;
                    self.set(*scope, name, new);
                }
            }
            Node::Capture { scope, name, body } => {
                let s = self.capture(body)?;
                self.set(*scope, name, Value::str(s));
            }
            Node::MacroDef(c) => {
                let ns = self.frame().ns;
                self.namespaces[ns].insert(c.name.clone(), Value::Macro(c.clone(), ns));
            }
            Node::Return(e, line) => {
                let v = match e {
                    Some(e) => self.eval(e).map_err(|err| self.located(err, *line))?,
                    None => Value::Null,
                };
                return Ok(Flow::Return(v));
            }
            Node::CallUser {
                callee,
                args,
                body,
                line,
            } => {
                self.call_user(callee, args, body.as_ref())
                    .map_err(|e| self.located(e, *line))?;
            }
            Node::Nested(line) => {
                if let Some((body, caller)) = self.frame().nested.clone() {
                    let saved = self.cur;
                    self.cur = caller;
                    let r = self.exec(&body);
                    self.cur = saved;
                    r.map_err(|e| self.located(e, *line))?;
                }
            }
            Node::Include(path, line) => {
                let p = self.eval(path).map_err(|e| self.located(e, *line))?;
                let p = p
                    .as_str()
                    .ok_or_else(|| self.located(fail("include espera un texto"), *line))?
                    .to_string();
                let name = resolve(&self.frame().template, &p);
                let t = self
                    .engine
                    .template(&name)
                    .map_err(|e| self.located(e, *line))?;
                let saved = std::mem::replace(&mut self.frame_mut().template, t.name.clone());
                self.hoist(&t.body);
                let r = self.exec(&t.body);
                self.frame_mut().template = saved;
                if let Flow::Return(v) = r? {
                    return Ok(Flow::Return(v));
                }
            }
            Node::Import(path, ns_name, line) => {
                let p = self.eval(path).map_err(|e| self.located(e, *line))?;
                let p = p
                    .as_str()
                    .ok_or_else(|| self.located(fail("import espera un texto"), *line))?
                    .to_string();
                let name = resolve(&self.frame().template, &p);
                let t = self
                    .engine
                    .template(&name)
                    .map_err(|e| self.located(e, *line))?;
                let ns = self.namespaces.len();
                self.namespaces.push(Map::new());
                self.frames.push(Frame {
                    locals: Map::new(),
                    loops: Vec::new(),
                    ns,
                    nested: None,
                    template: t.name.clone(),
                    is_macro: false,
                });
                let saved = self.cur;
                self.cur = self.frames.len() - 1;
                self.hoist(&t.body);
                self.out.push(String::new());
                let r = self.exec(&t.body);
                self.out.pop();
                self.frames.pop();
                self.cur = saved;
                r?;
                let exported = Value::hash(self.namespaces[ns].clone());
                self.set(Scope::Namespace, ns_name, exported);
            }
            Node::Break => return Ok(Flow::Break),
            Node::Continue => return Ok(Flow::Continue),
        }
        Ok(Flow::Normal)
    }

    fn exec_list(
        &mut self,
        seq: &Expr,
        vars: &[String],
        body: &Body,
        else_body: Option<&Body>,
        line: u32,
    ) -> Result<Flow, FtlError> {
        let v = self.eval(seq).map_err(|e| self.located(e, line))?;
        let items: Vec<Vec<Value>> = match &v {
            Value::Seq(s) => s.iter().map(|x| vec![x.clone()]).collect(),
            Value::Hash(m) => m
                .iter()
                .map(|(k, x)| {
                    if vars.len() == 2 {
                        vec![Value::str(k.as_str()), x.clone()]
                    } else {
                        vec![Value::str(k.as_str())]
                    }
                })
                .collect(),
            Value::Null => return Err(self.located(missing("la lista no tiene valor"), line)),
            other => {
                return Err(self.located(
                    fail(format!("no se puede recorrer un {}", other.type_name())),
                    line,
                ));
            }
        };
        if items.is_empty() {
            return match else_body {
                Some(b) => self.exec(b),
                None => Ok(Flow::Normal),
            };
        }
        let total = items.len();
        self.frame_mut().loops.push(Loop {
            vars: vars.to_vec(),
            values: Vec::new(),
            index: 0,
            has_next: false,
        });
        let mut result = Ok(Flow::Normal);
        for (i, values) in items.into_iter().enumerate() {
            {
                let l = self.frame_mut().loops.last_mut().expect("bucle");
                l.values = values;
                l.index = i;
                l.has_next = i + 1 < total;
            }
            match self.exec(body) {
                Ok(Flow::Break) => break,
                Ok(Flow::Normal) | Ok(Flow::Continue) => {}
                other => {
                    result = other;
                    break;
                }
            }
        }
        self.frame_mut().loops.pop();
        result
    }

    pub fn capture(&mut self, body: &[Node]) -> Result<String, FtlError> {
        self.out.push(String::new());
        let r = self.exec(body);
        let s = self.out.pop().expect("salida");
        r?;
        Ok(s)
    }

    fn assign_value(
        &mut self,
        name: &str,
        op: AssignOp,
        value: Option<&Expr>,
    ) -> Result<Value, FtlError> {
        let rhs = match value {
            Some(e) => self.eval(e)?,
            None => Value::Num(1.0),
        };
        if op == AssignOp::Set {
            return Ok(rhs);
        }
        let cur = self.lookup(name);
        let bin = match op {
            AssignOp::Add | AssignOp::Inc => BinOp::Add,
            AssignOp::Sub | AssignOp::Dec => BinOp::Sub,
            AssignOp::Mul => BinOp::Mul,
            AssignOp::Div => BinOp::Div,
            AssignOp::Set => unreachable!(),
        };
        self.binary(bin, cur, rhs)
    }

    // --- Macros y funciones ------------------------------------------------------

    fn call_user(
        &mut self,
        callee: &Expr,
        args: &Args,
        body: Option<&Body>,
    ) -> Result<(), FtlError> {
        let target = self.eval(callee)?;
        match target {
            Value::Macro(def, ns) => {
                let (pos, named) = self.eval_args(args)?;
                self.call_callable(&def, ns, pos, named, body.cloned())?;
                Ok(())
            }
            Value::Directive(f) => {
                let (_, named) = self.eval_args(args)?;
                let inner = match body {
                    Some(b) => self.capture(b)?,
                    None => String::new(),
                };
                let s = f(&named, inner)?;
                self.write(&s);
                Ok(())
            }
            Value::Template(body, name) => {
                let saved = std::mem::replace(&mut self.frame_mut().template, name);
                let r = self.exec(&body);
                self.frame_mut().template = saved;
                r.map(|_| ())
            }
            Value::Null => Err(missing(format!("la macro {callee:?} no existe"))),
            other => Err(fail(format!(
                "<@…> espera una macro y recibió un {}",
                other.type_name()
            ))),
        }
    }

    fn eval_args(&mut self, args: &Args) -> Result<(Vec<Value>, Map), FtlError> {
        match args {
            Args::Positional(list) => {
                let mut v = Vec::with_capacity(list.len());
                for e in list {
                    v.push(self.eval(e)?);
                }
                Ok((v, Map::new()))
            }
            Args::Named(list) => {
                let mut m = Map::new();
                for (k, e) in list {
                    m.insert(k.clone(), self.eval(e)?);
                }
                Ok((Vec::new(), m))
            }
        }
    }

    /// Ejecuta una macro (salida en la salida actual) o una función (devuelve su valor).
    pub fn call_callable(
        &mut self,
        def: &Rc<Callable>,
        ns: usize,
        positional: Vec<Value>,
        mut named: Map,
        nested: Option<Body>,
    ) -> Result<Value, FtlError> {
        if self.depth >= MAX_DEPTH {
            return Err(fail(format!(
                "demasiadas llamadas anidadas al llamar a {}",
                def.name
            )));
        }
        if positional.len() > def.params.len() {
            return Err(fail(format!(
                "{} recibe {} argumentos y se le pasaron {}",
                def.name,
                def.params.len(),
                positional.len()
            )));
        }
        let caller = self.cur;
        self.frames.push(Frame {
            locals: Map::new(),
            loops: Vec::new(),
            ns,
            nested: nested.map(|b| (b, caller)),
            template: def.template.clone(),
            is_macro: true,
        });
        let idx = self.frames.len() - 1;
        self.cur = idx;
        self.depth += 1;

        let result = (|| {
            let mut positional = positional.into_iter();
            for (pname, default) in &def.params {
                // Un argumento nulo cuenta como no pasado: toma el valor por defecto
                // (FreeMarker hace lo mismo; las plantillas pasan campos nulos de Java).
                let given = positional.next().or_else(|| named.shift_remove(pname));
                let passed_null = given.as_ref().is_some_and(Value::is_null);
                let value = match (given.filter(|v| !v.is_null()), default) {
                    (Some(v), _) => v,
                    (None, Some(d)) => self.eval(d)?,
                    (None, None) if passed_null => {
                        return Err(missing(format!(
                            "el parámetro {pname} de {} se pasó sin valor",
                            def.name
                        )));
                    }
                    (None, None) => {
                        return Err(fail(format!("falta el parámetro {pname} de {}", def.name)));
                    }
                };
                self.frames[idx].locals.insert(pname.clone(), value);
            }
            if let Some(rest) = &def.catch_all {
                self.frames[idx]
                    .locals
                    .insert(rest.clone(), Value::hash(std::mem::take(&mut named)));
            } else if let Some((k, _)) = named.first() {
                return Err(fail(format!("{} no tiene el parámetro {k}", def.name)));
            }
            if def.is_function {
                self.out.push(String::new());
                let r = self.exec(&def.body);
                self.out.pop();
                match r? {
                    Flow::Return(v) => Ok(v),
                    _ => Ok(Value::Null),
                }
            } else {
                self.exec(&def.body)?;
                Ok(Value::Null)
            }
        })();

        self.depth -= 1;
        self.frames.truncate(idx);
        self.cur = caller;
        result
    }

    /// Aplica una función, lambda o macro-función a unos argumentos.
    pub fn apply(&mut self, f: &Value, args: Vec<Value>) -> Result<Value, FtlError> {
        match f {
            Value::Fn(h) => h(&args),
            Value::Lambda(param, body) => {
                let body = body.clone();
                self.frame_mut().loops.push(Loop {
                    vars: vec![param.clone()],
                    values: vec![args.into_iter().next().unwrap_or(Value::Null)],
                    index: 0,
                    has_next: false,
                });
                let r = self.eval(&body);
                self.frame_mut().loops.pop();
                r
            }
            Value::Macro(def, ns) if def.is_function => {
                self.call_callable(&def.clone(), *ns, args, Map::new(), None)
            }
            Value::Null => Err(missing("se llamó a una función que no existe")),
            other => Err(fail(format!(
                "no se puede llamar a un {}",
                other.type_name()
            ))),
        }
    }

    // --- Expresiones ------------------------------------------------------------------

    pub fn eval(&mut self, e: &Expr) -> Result<Value, FtlError> {
        match e {
            Expr::Str(parts) => {
                let mut s = String::new();
                for p in parts {
                    match p {
                        StrPart::Lit(l) => s.push_str(l),
                        StrPart::Expr(x) => {
                            let v = self.eval(x)?;
                            s.push_str(&self.display(&v)?);
                        }
                    }
                }
                Ok(Value::str(s))
            }
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Var(name) => Ok(self.lookup(name)),
            Expr::Dot(obj, name) => {
                let v = self.eval(obj)?;
                match &v {
                    // Como el MapModel de FreeMarker: primero la clave, después el método de Java.
                    Value::Hash(m) => Ok(m
                        .get(name)
                        .cloned()
                        .or_else(|| crate::java::member(&v, name))
                        .unwrap_or(Value::Null)),
                    Value::Object(o) => Ok(o.get(name).unwrap_or(Value::Null)),
                    Value::Null => Err(missing(format!(
                        "no se puede leer .{name}: lo anterior no tiene valor ({})",
                        describe(obj)
                    ))),
                    other => crate::java::member(other, name).ok_or_else(|| {
                        fail(format!(
                            "no se puede leer .{name} de un {}",
                            other.type_name()
                        ))
                    }),
                }
            }
            Expr::Index(obj, idx) => {
                let v = self.eval(obj)?;
                if let Expr::Range(a, b, kind) = idx.as_ref() {
                    return self.slice(&v, a, b.as_deref(), *kind);
                }
                let i = self.eval(idx)?;
                match (&v, &i) {
                    (Value::Seq(s), Value::Num(n)) => {
                        Ok(s.get(*n as usize).cloned().unwrap_or(Value::Null))
                    }
                    (Value::Str(s), Value::Num(n)) => Ok(s
                        .chars()
                        .nth(*n as usize)
                        .map(|c| Value::str(c.to_string()))
                        .unwrap_or(Value::Null)),
                    (Value::Hash(_) | Value::Object(_), Value::Str(k)) => {
                        Ok(v.get(k).unwrap_or(Value::Null))
                    }
                    (Value::Null, _) => Err(missing(format!(
                        "no se puede indexar: {} no tiene valor",
                        describe(obj)
                    ))),
                    _ => Err(fail(format!(
                        "no se puede indexar un {} con un {}",
                        v.type_name(),
                        i.type_name()
                    ))),
                }
            }
            Expr::Call(callee, args) => {
                let f = self.eval(callee).map_err(|err| match err {
                    FtlError::Missing(m) => {
                        missing(format!("{m} (al llamar a {})", describe(callee)))
                    }
                    other => other,
                })?;
                if f.is_null() {
                    return Err(missing(format!("{} no existe", describe(callee))));
                }
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a)?);
                }
                self.apply(&f, vals)
            }
            Expr::Builtin(target, name, args) => {
                crate::builtins::apply(self, target, name, args.as_deref())
            }
            Expr::Default(x, d) => match self.eval(x) {
                Ok(Value::Null) | Err(FtlError::Missing(_)) => match d {
                    Some(d) => self.eval(d),
                    None => Ok(Value::str("")),
                },
                other => other,
            },
            Expr::Exists(x) => match self.eval(x) {
                Ok(Value::Null) | Err(FtlError::Missing(_)) => Ok(Value::Bool(false)),
                Ok(_) => Ok(Value::Bool(true)),
                Err(e) => Err(e),
            },
            Expr::Not(x) => {
                let v = self.eval(x)?;
                Ok(Value::Bool(!self.truthy(&v)?))
            }
            Expr::Neg(x) => match self.eval(x)? {
                Value::Num(n) => Ok(Value::Num(-n)),
                other => Err(fail(format!("no se puede negar un {}", other.type_name()))),
            },
            Expr::Bin(BinOp::And, a, b) => {
                let av = self.eval(a)?;
                if !self.truthy(&av)? {
                    return Ok(Value::Bool(false));
                }
                let bv = self.eval(b)?;
                Ok(Value::Bool(self.truthy(&bv)?))
            }
            Expr::Bin(BinOp::Or, a, b) => {
                let av = self.eval(a)?;
                if self.truthy(&av)? {
                    return Ok(Value::Bool(true));
                }
                let bv = self.eval(b)?;
                Ok(Value::Bool(self.truthy(&bv)?))
            }
            Expr::Bin(op, a, b) => {
                let av = self.eval(a)?;
                let bv = self.eval(b)?;
                self.binary(*op, av, bv).map_err(|err| match err {
                    FtlError::Missing(m) => {
                        missing(format!("{m}: {} {op:?} {}", describe(a), describe(b)))
                    }
                    other => other,
                })
            }
            Expr::Range(a, b, kind) => {
                let start = self.num(a)?;
                let items: Vec<Value> = match (kind, b) {
                    (RangeKind::Open, _) | (_, None) => {
                        return Err(fail(
                            "un rango abierto (a..) solo sirve para recortar listas",
                        ));
                    }
                    (RangeKind::Inclusive, Some(b)) => {
                        let end = self.num(b)?;
                        if end >= start {
                            (start as i64..=end as i64)
                                .map(|n| Value::Num(n as f64))
                                .collect()
                        } else {
                            (end as i64..=start as i64)
                                .rev()
                                .map(|n| Value::Num(n as f64))
                                .collect()
                        }
                    }
                    (RangeKind::Exclusive, Some(b)) => {
                        let end = self.num(b)?;
                        (start as i64..end as i64)
                            .map(|n| Value::Num(n as f64))
                            .collect()
                    }
                    (RangeKind::Length, Some(b)) => {
                        let len = self.num(b)?;
                        (0..len.max(0.0) as i64)
                            .map(|n| Value::Num(start + n as f64))
                            .collect()
                    }
                };
                Ok(Value::seq(items))
            }
            Expr::Seq(items) => {
                let mut v = Vec::with_capacity(items.len());
                for i in items {
                    v.push(self.eval(i)?);
                }
                Ok(Value::seq(v))
            }
            Expr::Hash(pairs) => {
                let mut m = Map::new();
                for (k, v) in pairs {
                    let key = self.eval(k)?;
                    let key = self.display(&key)?;
                    let val = self.eval(v)?;
                    m.insert(key, val);
                }
                Ok(Value::hash(m))
            }
            Expr::Lambda(p, body) => Ok(Value::Lambda(p.clone(), body.clone())),
        }
    }

    fn slice(
        &mut self,
        v: &Value,
        a: &Expr,
        b: Option<&Expr>,
        kind: RangeKind,
    ) -> Result<Value, FtlError> {
        let len = match v {
            Value::Seq(s) => s.len(),
            Value::Str(s) => s.chars().count(),
            Value::Null => return Err(missing("no se puede recortar: no tiene valor")),
            other => {
                return Err(fail(format!(
                    "no se puede recortar un {}",
                    other.type_name()
                )));
            }
        };
        let start = self.num(a)?.max(0.0) as usize;
        let end = match (kind, b) {
            (RangeKind::Open, _) | (_, None) => len,
            (RangeKind::Inclusive, Some(b)) => self.num(b)? as usize + 1,
            (RangeKind::Exclusive, Some(b)) => self.num(b)? as usize,
            (RangeKind::Length, Some(b)) => (start + self.num(b)? as usize).min(len),
        };
        let (start, end) = (start.min(len), end.min(len).max(start.min(len)));
        Ok(match v {
            Value::Seq(s) => Value::seq(s[start..end].to_vec()),
            Value::Str(s) => {
                Value::str(s.chars().skip(start).take(end - start).collect::<String>())
            }
            _ => unreachable!(),
        })
    }

    pub fn num(&mut self, e: &Expr) -> Result<f64, FtlError> {
        match self.eval(e)? {
            Value::Num(n) => Ok(n),
            Value::Null => Err(missing(format!("{} no tiene valor", describe(e)))),
            other => Err(fail(format!(
                "se esperaba un número y es un {}",
                other.type_name()
            ))),
        }
    }

    pub fn truthy(&self, v: &Value) -> Result<bool, FtlError> {
        match v {
            Value::Bool(b) => Ok(*b),
            Value::Null => Err(missing("la condición no tiene valor")),
            other => Err(fail(format!(
                "se esperaba un booleano y es un {}",
                other.type_name()
            ))),
        }
    }

    pub fn binary(&self, op: BinOp, a: Value, b: Value) -> Result<Value, FtlError> {
        use Value::*;
        if a.is_null() || b.is_null() {
            return Err(missing("uno de los lados no tiene valor"));
        }
        Ok(match op {
            BinOp::Eq | BinOp::Ne => {
                let eq = match (&a, &b) {
                    (Num(x), Num(y)) => x == y,
                    (Str(x), Str(y)) => x == y,
                    (Bool(x), Bool(y)) => x == y,
                    // Un objeto que se escribe como texto se compara como texto (StringModel).
                    (Object(_), Str(_)) | (Str(_), Object(_)) | (Object(_), Object(_))
                        if a.text().is_some() && b.text().is_some() =>
                    {
                        a.text() == b.text()
                    }
                    _ => {
                        return Err(fail(format!(
                            "no se puede comparar un {} con un {}",
                            a.type_name(),
                            b.type_name()
                        )));
                    }
                };
                Bool(if op == BinOp::Eq { eq } else { !eq })
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match (&a, &b) {
                (Num(x), Num(y)) => Bool(match op {
                    BinOp::Lt => x < y,
                    BinOp::Le => x <= y,
                    BinOp::Gt => x > y,
                    _ => x >= y,
                }),
                _ => {
                    return Err(fail(format!(
                        "solo se comparan números con < y >, no {} y {}",
                        a.type_name(),
                        b.type_name()
                    )));
                }
            },
            BinOp::Add => match (&a, &b) {
                (Num(x), Num(y)) => Num(x + y),
                (Seq(x), Seq(y)) => {
                    let mut v = x.as_ref().clone();
                    v.extend(y.iter().cloned());
                    Value::seq(v)
                }
                (Hash(x), Hash(y)) => {
                    let mut m = x.as_ref().clone();
                    for (k, v) in y.iter() {
                        m.insert(k.clone(), v.clone());
                    }
                    Value::hash(m)
                }
                _ => Value::str(format!("{}{}", self.display(&a)?, self.display(&b)?)),
            },
            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => match (&a, &b) {
                (Num(x), Num(y)) => Num(match op {
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => x / y,
                    _ => x % y,
                }),
                _ => {
                    return Err(fail(format!(
                        "operación numérica con un {} y un {}",
                        a.type_name(),
                        b.type_name()
                    )));
                }
            },
            BinOp::And | BinOp::Or => unreachable!("se evalúan en corto"),
        })
    }

    /// Texto de un valor para `${…}`.
    pub fn display(&self, v: &Value) -> Result<String, FtlError> {
        match v {
            Value::Str(s) => Ok(s.to_string()),
            Value::Num(n) => Ok(format_number(*n)),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Err(missing("el valor no existe")),
            Value::Object(o) if o.to_text().is_some() => Ok(o.to_text().unwrap_or_default()),
            other => Err(fail(format!(
                "no se puede escribir un {} como texto",
                other.type_name()
            ))),
        }
    }
}

/// Resuelve una ruta de `include`/`import` relativa a la plantilla actual.
pub fn resolve(current: &str, path: &str) -> String {
    let mut parts: Vec<&str> = if let Some(abs) = path.strip_prefix('/') {
        return normalize(abs.split('/').collect());
    } else {
        current.split('/').collect()
    };
    parts.pop(); // nombre del archivo actual
    parts.extend(path.split('/'));
    normalize(parts)
}

fn normalize(parts: Vec<&str>) -> String {
    let mut out: Vec<&str> = Vec::new();
    for p in parts {
        match p {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out.join("/")
}

/// Descripción corta de una expresión para los mensajes de error.
pub fn describe(e: &Expr) -> String {
    match e {
        Expr::Var(n) => n.clone(),
        Expr::Dot(o, n) => format!("{}.{n}", describe(o)),
        Expr::Call(c, _) => format!("{}(…)", describe(c)),
        Expr::Index(o, _) => format!("{}[…]", describe(o)),
        Expr::Builtin(o, n, _) => format!("{}?{n}", describe(o)),
        Expr::Str(_) => "texto".into(),
        Expr::Num(n) => format_number(*n),
        _ => "expresión".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[test]
    fn include_paths() {
        assert_eq!(
            resolve("item/item.java.ftl", "../procedures.java.ftl"),
            "procedures.java.ftl"
        );
        assert_eq!(resolve("a/b/c.ftl", "d.ftl"), "a/b/d.ftl");
        assert_eq!(resolve("a/b/c.ftl", "/x/y.ftl"), "x/y.ftl");
        assert_eq!(resolve("c.ftl", "./d.ftl"), "d.ftl");
    }
}
