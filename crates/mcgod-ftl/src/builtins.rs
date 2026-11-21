//! Built-ins (`x?nombre`). Están los que usan las plantillas de MCreator y algunos afines.

use crate::FtlError;
use crate::ast::Expr;
use crate::eval::{Run, describe, fail, missing};
use crate::value::{Value, format_c_number, format_number};

pub(crate) fn apply(
    run: &mut Run,
    target: &Expr,
    name: &str,
    args: Option<&[Expr]>,
) -> Result<Value, FtlError> {
    // Built-ins de variables de bucle: el operando es el nombre de la variable.
    if matches!(
        name,
        "index" | "counter" | "has_next" | "is_first" | "is_last" | "item_parity"
    ) {
        let Expr::Var(var) = target else {
            return Err(fail(format!("?{name} solo sirve con una variable de list")));
        };
        let l = run
            .find_loop(var)
            .ok_or_else(|| fail(format!("{var} no es una variable de list (para ?{name})")))?;
        return Ok(match name {
            "index" => Value::Num(l.index as f64),
            "counter" => Value::Num(l.index as f64 + 1.0),
            "has_next" => Value::Bool(l.has_next),
            "is_first" => Value::Bool(l.index == 0),
            "is_last" => Value::Bool(!l.has_next),
            _ => Value::str(if l.index % 2 == 0 { "odd" } else { "even" }),
        });
    }

    // `?then` evalúa solo la rama elegida.
    if name == "then" {
        let args = args
            .filter(|a| a.len() == 2)
            .ok_or_else(|| fail("?then necesita dos argumentos"))?;
        let cond = run.eval(target)?;
        let pick = if run.truthy(&cond)? {
            &args[0]
        } else {
            &args[1]
        };
        return run.eval(pick);
    }

    if name == "has_content" {
        return Ok(Value::Bool(match run.eval(target) {
            Ok(v) => has_content(&v),
            Err(FtlError::Missing(_)) => false,
            Err(e) => return Err(e),
        }));
    }

    let v = run.eval(target)?;
    let mut a = Vec::new();
    for e in args.unwrap_or(&[]) {
        a.push(run.eval(e)?);
    }
    if v.is_null()
        && !matches!(
            name,
            "is_boolean" | "is_string" | "is_number" | "is_sequence" | "is_hash"
        )
    {
        return Err(missing(format!(
            "{} no tiene valor (al aplicar ?{name})",
            describe(target)
        )));
    }
    let arg_str = |i: usize| -> Result<String, FtlError> {
        match a.get(i) {
            Some(Value::Str(s)) => Ok(s.to_string()),
            Some(Value::Num(n)) => Ok(format_number(*n)),
            Some(other) => Err(fail(format!(
                "?{name}: el argumento {} debe ser texto, es {}",
                i + 1,
                other.type_name()
            ))),
            None => Err(fail(format!("?{name}: falta el argumento {}", i + 1))),
        }
    };
    let arg_num = |i: usize| -> Result<f64, FtlError> {
        match a.get(i) {
            Some(Value::Num(n)) => Ok(*n),
            _ => Err(fail(format!(
                "?{name}: el argumento {} debe ser un número",
                i + 1
            ))),
        }
    };

    match (name, &v) {
        // --- Tipos ---
        ("is_boolean", _) => Ok(Value::Bool(matches!(v, Value::Bool(_)))),
        ("is_string", _) => Ok(Value::Bool(matches!(v, Value::Str(_)))),
        ("is_number", _) => Ok(Value::Bool(matches!(v, Value::Num(_)))),
        ("is_sequence", _) => Ok(Value::Bool(matches!(v, Value::Seq(_)))),
        ("is_hash", _) => Ok(Value::Bool(matches!(v, Value::Hash(_) | Value::Object(_)))),

        // --- Textos ---
        ("lower_case", Value::Str(s)) => Ok(Value::str(s.to_lowercase())),
        ("upper_case", Value::Str(s)) => Ok(Value::str(s.to_uppercase())),
        ("cap_first", Value::Str(s)) => Ok(Value::str(cap_first(s))),
        ("uncap_first", Value::Str(s)) => Ok(Value::str(uncap_first(s))),
        ("trim", Value::Str(s)) => Ok(Value::str(s.trim())),
        ("length", Value::Str(s)) => Ok(Value::Num(s.chars().count() as f64)),
        ("replace", Value::Str(s)) => Ok(Value::str(s.replace(&arg_str(0)?, &arg_str(1)?))),
        ("starts_with", Value::Str(s)) => Ok(Value::Bool(s.starts_with(&arg_str(0)?))),
        ("ends_with", Value::Str(s)) => Ok(Value::Bool(s.ends_with(&arg_str(0)?))),
        ("contains", Value::Str(s)) => Ok(Value::Bool(s.contains(&arg_str(0)?))),
        ("keep_before", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(s.find(&p).map(|i| &s[..i]).unwrap_or(s)))
        }
        ("keep_before_last", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(s.rfind(&p).map(|i| &s[..i]).unwrap_or(s)))
        }
        ("keep_after", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(
                s.find(&p).map(|i| &s[i + p.len()..]).unwrap_or(""),
            ))
        }
        ("keep_after_last", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(
                s.rfind(&p).map(|i| &s[i + p.len()..]).unwrap_or(""),
            ))
        }
        ("remove_beginning", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(s.strip_prefix(p.as_str()).unwrap_or(s)))
        }
        ("remove_ending", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(s.strip_suffix(p.as_str()).unwrap_or(s)))
        }
        ("ensure_starts_with", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(if s.starts_with(&p) {
                s.to_string()
            } else {
                format!("{p}{s}")
            }))
        }
        ("ensure_ends_with", Value::Str(s)) => {
            let p = arg_str(0)?;
            Ok(Value::str(if s.ends_with(&p) {
                s.to_string()
            } else {
                format!("{s}{p}")
            }))
        }
        ("substring", Value::Str(s)) => {
            let chars: Vec<char> = s.chars().collect();
            let from = (arg_num(0)? as usize).min(chars.len());
            let to = if a.len() > 1 {
                (arg_num(1)? as usize).min(chars.len())
            } else {
                chars.len()
            };
            Ok(Value::str(
                chars[from..to.max(from)].iter().collect::<String>(),
            ))
        }
        ("index_of", Value::Str(s)) => Ok(Value::Num(char_index(s, s.find(&arg_str(0)?)))),
        ("last_index_of", Value::Str(s)) => Ok(Value::Num(char_index(s, s.rfind(&arg_str(0)?)))),
        ("split", Value::Str(s)) => {
            let sep = arg_str(0)?;
            Ok(Value::seq(s.split(sep.as_str()).map(Value::str).collect()))
        }
        ("matches", Value::Str(s)) => {
            let re = regex::Regex::new(&format!("^(?:{})$", arg_str(0)?))
                .map_err(|e| fail(format!("?matches: {e}")))?;
            Ok(Value::Bool(re.is_match(s)))
        }
        ("number", Value::Str(s)) => s
            .trim()
            .parse::<f64>()
            .map(Value::Num)
            .map_err(|_| fail(format!("?number: {s:?} no es un número"))),
        ("number", Value::Num(_)) => Ok(v.clone()),
        ("boolean", Value::Str(s)) => match s.as_ref() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(fail(format!("?boolean: {s:?} no es true ni false"))),
        },
        ("j_string", Value::Str(s)) => Ok(Value::str(java_escape(s))),
        ("json_string", Value::Str(s)) => {
            let q = serde_json::to_string(s.as_ref()).unwrap_or_default();
            Ok(Value::str(&q[1..q.len() - 1]))
        }
        ("eval_json", Value::Str(s)) => serde_json::from_str::<serde_json::Value>(s)
            .map(|j| Value::from_json(&j))
            .map_err(|e| fail(format!("?eval_json: {e}"))),
        ("interpret", Value::Str(s)) => {
            let name: std::rc::Rc<str> = format!("{}->interpret", run.template_name()).into();
            let body = crate::parser::parse_template(&name, s)
                .map_err(|e| fail(format!("?interpret, línea {}: {}", e.line, e.message)))?;
            Ok(Value::Template(body, name))
        }

        // --- Números ---
        ("int", Value::Num(n)) => Ok(Value::Num(n.trunc())),
        ("floor", Value::Num(n)) => Ok(Value::Num(n.floor())),
        ("ceiling", Value::Num(n)) => Ok(Value::Num(n.ceil())),
        ("round", Value::Num(n)) => Ok(Value::Num((n + 0.5).floor())),
        ("abs", Value::Num(n)) => Ok(Value::Num(n.abs())),

        // --- Conversión a texto ---
        ("c", Value::Num(n)) => Ok(Value::str(format_c_number(*n))),
        ("c", Value::Bool(b)) => Ok(Value::str(b.to_string())),
        ("c", Value::Str(s)) => Ok(Value::str(
            serde_json::to_string(s.as_ref()).unwrap_or_default(),
        )),
        ("string", Value::Str(_)) => Ok(v.clone()),
        ("string", Value::Num(n)) => Ok(Value::str(format_number(*n))),
        ("string", Value::Bool(b)) => {
            if a.len() == 2 {
                Ok(Value::str(if *b { arg_str(0)? } else { arg_str(1)? }))
            } else {
                Ok(Value::str(b.to_string()))
            }
        }

        // --- Listas ---
        ("size", Value::Seq(s)) => Ok(Value::Num(s.len() as f64)),
        ("size", Value::Hash(m)) => Ok(Value::Num(m.len() as f64)),
        ("first", Value::Seq(s)) => s
            .first()
            .cloned()
            .ok_or_else(|| missing("?first de una lista vacía")),
        ("last", Value::Seq(s)) => s
            .last()
            .cloned()
            .ok_or_else(|| missing("?last de una lista vacía")),
        ("reverse", Value::Seq(s)) => Ok(Value::seq(s.iter().rev().cloned().collect())),
        ("join", Value::Seq(s)) => {
            let sep = arg_str(0)?;
            let mut parts = Vec::with_capacity(s.len());
            for x in s.iter() {
                if !x.is_null() {
                    parts.push(run.display(x)?);
                }
            }
            if parts.is_empty() && a.len() > 1 {
                return Ok(Value::str(arg_str(1)?));
            }
            let mut out = parts.join(&sep);
            if !parts.is_empty() && a.len() > 2 {
                out.push_str(&arg_str(2)?);
            }
            Ok(Value::str(out))
        }
        ("seq_contains", Value::Seq(s)) => {
            let needle = a
                .first()
                .ok_or_else(|| fail("?seq_contains necesita un argumento"))?;
            Ok(Value::Bool(s.iter().any(|x| loose_eq(x, needle))))
        }
        ("seq_index_of", Value::Seq(s)) => {
            let needle = a
                .first()
                .ok_or_else(|| fail("?seq_index_of necesita un argumento"))?;
            Ok(Value::Num(
                s.iter()
                    .position(|x| loose_eq(x, needle))
                    .map(|i| i as f64)
                    .unwrap_or(-1.0),
            ))
        }
        ("chunk", Value::Seq(s)) => {
            let n = arg_num(0)?.max(1.0) as usize;
            Ok(Value::seq(
                s.chunks(n).map(|c| Value::seq(c.to_vec())).collect(),
            ))
        }
        ("min" | "max", Value::Seq(s)) => {
            let mut best: Option<f64> = None;
            for x in s.iter() {
                match x {
                    Value::Num(n) => {
                        best = Some(match best {
                            None => *n,
                            Some(b) if name == "min" => b.min(*n),
                            Some(b) => b.max(*n),
                        })
                    }
                    Value::Null => {}
                    other => {
                        return Err(fail(format!(
                            "?{name} solo compara números, no {}",
                            other.type_name()
                        )));
                    }
                }
            }
            Ok(best.map(Value::Num).unwrap_or(Value::Null))
        }
        ("filter" | "map", Value::Seq(s)) => {
            let f = a
                .first()
                .cloned()
                .ok_or_else(|| fail(format!("?{name} necesita una función")))?;
            let mut out = Vec::new();
            for x in s.iter() {
                let r = run.apply(&f, vec![x.clone()])?;
                if name == "map" {
                    out.push(r);
                } else if run.truthy(&r)? {
                    out.push(x.clone());
                }
            }
            Ok(Value::seq(out))
        }
        ("sort", Value::Seq(s)) => {
            let mut items = s.as_ref().clone();
            items.sort_by(|x, y| match (x, y) {
                (Value::Num(a), Value::Num(b)) => {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::Str(a), Value::Str(b)) => a.cmp(b),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::seq(items))
        }

        // --- Mapas ---
        ("keys", Value::Hash(m)) => Ok(Value::seq(
            m.keys().map(|k| Value::str(k.as_str())).collect(),
        )),
        ("values", Value::Hash(m)) => Ok(Value::seq(m.values().cloned().collect())),

        _ => Err(fail(format!(
            "?{name} no se puede aplicar a un {}",
            v.type_name()
        ))),
    }
}

pub(crate) fn has_content(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Str(s) => !s.is_empty(),
        Value::Seq(s) => !s.is_empty(),
        Value::Hash(m) => !m.is_empty(),
        _ => true,
    }
}

fn loose_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        _ => false,
    }
}

fn char_index(s: &str, byte: Option<usize>) -> f64 {
    byte.map(|b| s[..b].chars().count() as f64).unwrap_or(-1.0)
}

fn cap_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn uncap_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn java_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
