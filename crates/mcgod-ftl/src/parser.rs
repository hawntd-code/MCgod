//! Convierte las piezas de una plantilla en un árbol de nodos.

use crate::ast::{Args, AssignOp, Body, Callable, Node, Scope};
use crate::expr::{self, Parser, T};
use crate::lexer::{self, Tok};
use std::rc::Rc;

pub struct ParseError {
    pub line: u32,
    pub message: String,
}

fn perr(line: u32, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}

pub fn parse_template(name: &Rc<str>, src: &str) -> Result<Body, ParseError> {
    let toks = lexer::tokenize(src).map_err(|e| perr(e.line, e.message))?;
    let mut p = TParser {
        toks,
        pos: 0,
        template: name.clone(),
    };
    let (body, end) = p.block(&[])?;
    if let Some(t) = end {
        return Err(perr(
            tok_line(&t),
            format!("etiqueta de cierre inesperada: {t:?}"),
        ));
    }
    Ok(body)
}

fn tok_line(t: &Tok) -> u32 {
    match t {
        Tok::Interp(_, l) => *l,
        Tok::Open { line, .. } | Tok::Close { line, .. } => *line,
        Tok::UserOpen { line, .. } | Tok::UserClose { line } => *line,
        _ => 0,
    }
}

struct TParser {
    toks: Vec<Tok>,
    pos: usize,
    template: Rc<str>,
}

/// Fin de un bloque: lo que lo terminó (ya consumido).
type BlockEnd = Option<Tok>;

impl TParser {
    /// Lee nodos hasta un cierre o hasta una de las directivas `stops` (`else`, `elseif`, `sep`).
    fn block(&mut self, stops: &[&str]) -> Result<(Body, BlockEnd), ParseError> {
        let mut nodes = Vec::new();
        while self.pos < self.toks.len() {
            let tok = self.toks[self.pos].clone();
            self.pos += 1;
            match tok {
                Tok::Text(s) => nodes.push(Node::Text(s)),
                Tok::Comment => {}
                Tok::Interp(src, line) => {
                    let e =
                        expr::parse(&src).map_err(|m| perr(line, format!("en ${{{src}}}: {m}")))?;
                    nodes.push(Node::Interp(e, line));
                }
                Tok::Close { .. } | Tok::UserClose { .. } => return Ok((nodes.into(), Some(tok))),
                Tok::Open { ref name, .. } if stops.contains(&name.as_str()) => {
                    return Ok((nodes.into(), Some(tok)));
                }
                Tok::Open {
                    name,
                    params,
                    self_closing,
                    line,
                } => {
                    if let Some(n) = self.directive(&name, &params, self_closing, line)? {
                        nodes.push(n);
                    }
                }
                Tok::UserOpen {
                    params,
                    self_closing,
                    line,
                } => nodes.push(self.user_call(&params, self_closing, line)?),
            }
        }
        Ok((nodes.into(), None))
    }

    fn expect_close(&mut self, end: BlockEnd, name: &str, line: u32) -> Result<(), ParseError> {
        match end {
            Some(Tok::Close { name: n, .. }) if n == name => Ok(()),
            Some(other) => Err(perr(
                tok_line(&other),
                format!("se esperaba </#{name}> (abierto en la línea {line}) y llegó {other:?}"),
            )),
            None => Err(perr(line, format!("falta </#{name}>"))),
        }
    }

    fn directive(
        &mut self,
        name: &str,
        params: &str,
        self_closing: bool,
        line: u32,
    ) -> Result<Option<Node>, ParseError> {
        let e = |m: String| perr(line, format!("<#{name}>: {m}"));
        Ok(Some(match name {
            "if" => {
                let mut branches = Vec::new();
                let mut cond = expr::parse(params).map_err(e)?;
                let else_body;
                loop {
                    let (body, end) = self.block(&["elseif", "else"])?;
                    branches.push((cond, body));
                    match end {
                        Some(Tok::Open {
                            name, params, line, ..
                        }) if name == "elseif" => {
                            cond = expr::parse(&params)
                                .map_err(|m| perr(line, format!("<#elseif>: {m}")))?;
                        }
                        Some(Tok::Open { name, .. }) if name == "else" => {
                            let (body, end) = self.block(&[])?;
                            self.expect_close(end, "if", line)?;
                            else_body = Some(body);
                            break;
                        }
                        end => {
                            self.expect_close(end, "if", line)?;
                            else_body = None;
                            break;
                        }
                    }
                }
                Node::If(branches, else_body, line)
            }
            "list" => {
                let mut p = Parser::new(params).map_err(e)?;
                let seq = p.expr().map_err(e)?;
                if !p.is_ident("as") {
                    return Err(e("falta `as`".into()));
                }
                p.advance();
                let mut vars = vec![p.ident().map_err(e)?];
                if p.eat_op(",") {
                    vars.push(p.ident().map_err(e)?);
                }
                p.expect_eof().map_err(e)?;
                let (body, end) = self.list_body()?;
                let (body, else_body) = match end {
                    Some(Tok::Open { name, .. }) if name == "else" => {
                        let (else_body, end) = self.block(&[])?;
                        self.expect_close(end, "list", line)?;
                        (body, Some(else_body))
                    }
                    end => {
                        self.expect_close(end, "list", line)?;
                        (body, None)
                    }
                };
                Node::List {
                    seq,
                    vars,
                    body,
                    else_body,
                    line,
                }
            }
            "assign" | "local" | "global" => {
                let scope = match name {
                    "local" => Scope::Local,
                    "global" => Scope::Global,
                    _ => Scope::Namespace,
                };
                let mut p = Parser::new(params).map_err(e)?;
                let first = p.ident().map_err(e)?;
                if p.at_eof() && !self_closing {
                    // Captura: <#assign x>…</#assign>
                    let (body, end) = self.block(&[])?;
                    self.expect_close(end, name, line)?;
                    return Ok(Some(Node::Capture {
                        scope,
                        name: first,
                        body,
                    }));
                }
                let mut items = Vec::new();
                let mut target = first;
                loop {
                    let op = match p.advance() {
                        T::Op("=") => AssignOp::Set,
                        T::Op("+=") => AssignOp::Add,
                        T::Op("-=") => AssignOp::Sub,
                        T::Op("*=") => AssignOp::Mul,
                        T::Op("/=") => AssignOp::Div,
                        T::Op("++") => AssignOp::Inc,
                        T::Op("--") => AssignOp::Dec,
                        other => return Err(e(format!("se esperaba = y llegó {other:?}"))),
                    };
                    let value = match op {
                        AssignOp::Inc | AssignOp::Dec => None,
                        _ => Some(p.expr().map_err(e)?),
                    };
                    items.push((target, op, value));
                    p.eat_op(",");
                    if p.at_eof() {
                        break;
                    }
                    target = p.ident().map_err(e)?;
                }
                Node::Assign { scope, items, line }
            }
            "macro" | "function" => {
                let mut p = Parser::new(params).map_err(e)?;
                let mname = p.ident().map_err(e)?;
                let mut params_out = Vec::new();
                let mut catch_all = None;
                p.eat_op("(");
                while !p.at_eof() && !p.is_op(")") {
                    let pname = p.ident().map_err(e)?;
                    if p.eat_op("...") {
                        catch_all = Some(pname);
                    } else if p.eat_op("=") {
                        let default = p.expr().map_err(e)?;
                        params_out.push((pname, Some(default)));
                    } else {
                        params_out.push((pname, None));
                    }
                    p.eat_op(",");
                }
                let (body, end) = self.block(&[])?;
                self.expect_close(end, name, line)?;
                Node::MacroDef(Rc::new(Callable {
                    name: mname,
                    params: params_out,
                    catch_all,
                    body,
                    is_function: name == "function",
                    template: self.template.clone(),
                }))
            }
            "return" => Node::Return(
                if params.is_empty() {
                    None
                } else {
                    Some(expr::parse(params).map_err(e)?)
                },
                line,
            ),
            "include" => {
                let mut p = Parser::new(params).map_err(e)?;
                Node::Include(p.expr().map_err(e)?, line)
            }
            "import" => {
                let mut p = Parser::new(params).map_err(e)?;
                let path = p.expr().map_err(e)?;
                if !p.is_ident("as") {
                    return Err(e("falta `as`".into()));
                }
                p.advance();
                Node::Import(path, p.ident().map_err(e)?, line)
            }
            "nested" => Node::Nested(line),
            "break" => Node::Break,
            "continue" => Node::Continue,
            // Ya aplicados por el lector o sin efecto en la salida.
            "t" | "lt" | "rt" | "nt" | "ftl" => return Ok(None),
            "sep" => {
                // <#sep> fuera de su lugar natural (por ejemplo, dentro de un if en un list).
                let (body, end) = self.block(&[])?;
                self.expect_close(end, "sep", line)?;
                Node::Sep(body)
            }
            other => return Err(e(format!("directiva no soportada: <#{other}>"))),
        }))
    }

    /// Cuerpo de un `list`, con `<#sep>` que puede cerrarse o llegar hasta el final del list.
    fn list_body(&mut self) -> Result<(Body, BlockEnd), ParseError> {
        let (body, end) = self.block(&["else", "sep"])?;
        match end {
            Some(Tok::Open { name, line, .. }) if name == "sep" => {
                let (sep_body, sep_end) = self.block(&["else"])?;
                let mut nodes: Vec<Node> = body.iter().cloned().collect();
                nodes.push(Node::Sep(sep_body));
                match sep_end {
                    Some(Tok::Close { ref name, .. }) if name == "sep" => {
                        // <#sep>…</#sep>: sigue el cuerpo del list.
                        let (rest, end) = self.block(&["else"])?;
                        nodes.extend(rest.iter().cloned());
                        Ok((nodes.into(), end))
                    }
                    end => {
                        let _ = line;
                        Ok((nodes.into(), end))
                    }
                }
            }
            end => Ok((body, end)),
        }
    }

    fn user_call(
        &mut self,
        params: &str,
        self_closing: bool,
        line: u32,
    ) -> Result<Node, ParseError> {
        let e = |m: String| perr(line, format!("<@{params}>: {m}"));
        let mut p = Parser::new(params).map_err(e)?;
        let callee = p.callee().map_err(e)?;
        let named = matches!(p.peek(), T::Ident(_)) && matches!(p.peek_at(1), T::Op("="));
        let args = if named {
            let mut list = Vec::new();
            while !p.at_eof() {
                let n = p.ident().map_err(e)?;
                p.expect_op("=").map_err(e)?;
                list.push((n, p.expr().map_err(e)?));
                p.eat_op(",");
            }
            Args::Named(list)
        } else {
            let mut list = Vec::new();
            while !p.at_eof() {
                list.push(p.expr().map_err(e)?);
                p.eat_op(",");
            }
            Args::Positional(list)
        };
        let body = if self_closing {
            None
        } else {
            let (body, end) = self.block(&[])?;
            match end {
                Some(Tok::UserClose { .. }) => Some(body),
                _ => return Err(perr(line, format!("falta </@> para <@{params}>"))),
            }
        };
        Ok(Node::CallUser {
            callee,
            args,
            body,
            line,
        })
    }
}
