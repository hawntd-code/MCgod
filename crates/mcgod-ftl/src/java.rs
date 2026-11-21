//! Métodos de Java sobre textos, listas, mapas y números.
//!
//! MCreator expone sus datos con el `BeansWrapper` de FreeMarker, así que las plantillas
//! llaman directamente a métodos de Java: `nombre.startsWith("CUSTOM:")`,
//! `mapa.keySet()`, `lista.size()`, `entrada.getKey()`. Aquí están los que aparecen
//! en las plantillas, con el mismo resultado que en Java.

use crate::FtlError;
use crate::eval::fail;
use crate::value::{HostObject, Map, Value, format_number};
use std::rc::Rc;

/// Miembro `name` de un valor que no es un objeto del anfitrión. `None` si no existe.
pub(crate) fn member(v: &Value, name: &str) -> Option<Value> {
    match v {
        Value::Str(s) => string_method(s.clone(), name),
        Value::Seq(s) => list_method(s.clone(), name),
        Value::Hash(m) => map_method(m.clone(), name),
        Value::Num(n) => number_method(*n, name),
        Value::Bool(b) => bool_method(*b, name),
        _ => None,
    }
}

fn method(f: impl Fn(&[Value]) -> Result<Value, FtlError> + 'static) -> Option<Value> {
    Some(Value::host_fn(f))
}

fn text_arg(args: &[Value], i: usize, method_name: &str) -> Result<String, FtlError> {
    match args.get(i) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        Some(Value::Num(n)) => Ok(format_number(*n)),
        Some(Value::Object(o)) => o
            .to_text()
            .ok_or_else(|| fail(format!("{method_name}: el argumento debe ser texto"))),
        Some(Value::Bool(b)) => Ok(b.to_string()),
        _ => Err(fail(format!(
            "{method_name}: falta el argumento {} o no es texto",
            i + 1
        ))),
    }
}

fn index_arg(args: &[Value], i: usize, method_name: &str) -> Result<usize, FtlError> {
    match args.get(i) {
        Some(Value::Num(n)) if *n >= 0.0 => Ok(*n as usize),
        _ => Err(fail(format!(
            "{method_name}: el argumento {} debe ser un índice",
            i + 1
        ))),
    }
}

/// Igualdad de Java `equals` entre valores de plantilla.
pub(crate) fn java_equals(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Seq(x), Value::Seq(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| java_equals(p, q))
        }
        _ => match (a.text(), b.text()) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        },
    }
}

fn string_method(s: Rc<str>, name: &str) -> Option<Value> {
    let n = name.to_string();
    match name {
        "startsWith" => method(move |a| Ok(Value::Bool(s.starts_with(&text_arg(a, 0, &n)?)))),
        "endsWith" => method(move |a| Ok(Value::Bool(s.ends_with(&text_arg(a, 0, &n)?)))),
        "contains" => method(move |a| Ok(Value::Bool(s.contains(&text_arg(a, 0, &n)?)))),
        "equals" => method(move |a| {
            Ok(Value::Bool(
                a.first()
                    .is_some_and(|x| java_equals(&Value::Str(s.clone()), x)),
            ))
        }),
        "equalsIgnoreCase" => method(move |a| {
            Ok(Value::Bool(
                s.to_lowercase() == text_arg(a, 0, &n)?.to_lowercase(),
            ))
        }),
        "replace" => method(move |a| {
            Ok(Value::str(
                s.replace(&text_arg(a, 0, &n)?, &text_arg(a, 1, &n)?),
            ))
        }),
        "replaceAll" | "replaceFirst" => method(move |a| {
            let re =
                regex::Regex::new(&text_arg(a, 0, &n)?).map_err(|e| fail(format!("{n}: {e}")))?;
            let with = text_arg(a, 1, &n)?.replace("$", "$$");
            Ok(Value::str(if n == "replaceAll" {
                re.replace_all(&s, with.as_str()).into_owned()
            } else {
                re.replace(&s, with.as_str()).into_owned()
            }))
        }),
        "isEmpty" => method(move |_| Ok(Value::Bool(s.is_empty()))),
        "isBlank" => method(move |_| Ok(Value::Bool(s.trim().is_empty()))),
        "length" => method(move |_| Ok(Value::Num(s.encode_utf16().count() as f64))),
        "toUpperCase" => method(move |_| Ok(Value::str(s.to_uppercase()))),
        "toLowerCase" => method(move |_| Ok(Value::str(s.to_lowercase()))),
        "trim" | "strip" => method(move |_| Ok(Value::str(s.trim()))),
        "toString" | "intern" => method(move |_| Ok(Value::Str(s.clone()))),
        "indexOf" => method(move |a| {
            let needle = text_arg(a, 0, &n)?;
            Ok(Value::Num(
                s.find(&needle)
                    .map(|b| s[..b].chars().count() as f64)
                    .unwrap_or(-1.0),
            ))
        }),
        "lastIndexOf" => method(move |a| {
            let needle = text_arg(a, 0, &n)?;
            Ok(Value::Num(
                s.rfind(&needle)
                    .map(|b| s[..b].chars().count() as f64)
                    .unwrap_or(-1.0),
            ))
        }),
        "substring" => method(move |a| {
            let chars: Vec<char> = s.chars().collect();
            let from = index_arg(a, 0, &n)?;
            let to = if a.len() > 1 {
                index_arg(a, 1, &n)?
            } else {
                chars.len()
            };
            if from > to || to > chars.len() {
                return Err(fail(format!(
                    "substring({from}, {to}) fuera de rango en un texto de {} caracteres",
                    chars.len()
                )));
            }
            Ok(Value::str(chars[from..to].iter().collect::<String>()))
        }),
        "charAt" => method(move |a| {
            let i = index_arg(a, 0, &n)?;
            s.chars()
                .nth(i)
                .map(|c| Value::str(c.to_string()))
                .ok_or_else(|| fail(format!("charAt({i}) fuera de rango")))
        }),
        "split" => method(move |a| {
            let re =
                regex::Regex::new(&text_arg(a, 0, &n)?).map_err(|e| fail(format!("split: {e}")))?;
            // Java descarta las cadenas vacías del final.
            let mut parts: Vec<&str> = re.split(&s).collect();
            while parts.len() > 1 && parts.last() == Some(&"") {
                parts.pop();
            }
            Ok(Value::seq(parts.into_iter().map(Value::str).collect()))
        }),
        "matches" => method(move |a| {
            let re = regex::Regex::new(&format!("^(?:{})$", text_arg(a, 0, &n)?))
                .map_err(|e| fail(format!("matches: {e}")))?;
            Ok(Value::Bool(re.is_match(&s)))
        }),
        "hashCode" => method(move |_| {
            // String.hashCode de Java (UTF-16, aritmética de 32 bits).
            let mut h: i32 = 0;
            for u in s.encode_utf16() {
                h = h.wrapping_mul(31).wrapping_add(u as i32);
            }
            Ok(Value::Num(h as f64))
        }),
        _ => None,
    }
}

fn list_method(s: Rc<Vec<Value>>, name: &str) -> Option<Value> {
    let n = name.to_string();
    match name {
        "size" => method(move |_| Ok(Value::Num(s.len() as f64))),
        "isEmpty" => method(move |_| Ok(Value::Bool(s.is_empty()))),
        "get" => method(move |a| {
            let i = index_arg(a, 0, &n)?;
            s.get(i).cloned().ok_or_else(|| {
                fail(format!(
                    "get({i}) fuera de rango en una lista de {} elementos",
                    s.len()
                ))
            })
        }),
        "getFirst" => method(move |_| {
            s.first()
                .cloned()
                .ok_or_else(|| fail("getFirst() de una lista vacía"))
        }),
        "getLast" => method(move |_| {
            s.last()
                .cloned()
                .ok_or_else(|| fail("getLast() de una lista vacía"))
        }),
        "contains" => method(move |a| {
            let needle = a.first().cloned().unwrap_or(Value::Null);
            Ok(Value::Bool(s.iter().any(|x| java_equals(x, &needle))))
        }),
        "indexOf" => method(move |a| {
            let needle = a.first().cloned().unwrap_or(Value::Null);
            Ok(Value::Num(
                s.iter()
                    .position(|x| java_equals(x, &needle))
                    .map(|i| i as f64)
                    .unwrap_or(-1.0),
            ))
        }),
        "subList" => method(move |a| {
            let from = index_arg(a, 0, &n)?;
            let to = index_arg(a, 1, &n)?;
            if from > to || to > s.len() {
                return Err(fail(format!("subList({from}, {to}) fuera de rango")));
            }
            Ok(Value::seq(s[from..to].to_vec()))
        }),
        _ => None,
    }
}

fn map_method(m: Rc<Map>, name: &str) -> Option<Value> {
    match name {
        "get" => method(move |a| {
            let key = a.first().and_then(Value::text).unwrap_or_default();
            Ok(m.get(&key).cloned().unwrap_or(Value::Null))
        }),
        "getOrDefault" => method(move |a| {
            let key = a.first().and_then(Value::text).unwrap_or_default();
            Ok(m.get(&key)
                .cloned()
                .unwrap_or_else(|| a.get(1).cloned().unwrap_or(Value::Null)))
        }),
        "containsKey" => method(move |a| {
            let key = a.first().and_then(Value::text).unwrap_or_default();
            Ok(Value::Bool(m.contains_key(&key)))
        }),
        "containsValue" => method(move |a| {
            let needle = a.first().cloned().unwrap_or(Value::Null);
            Ok(Value::Bool(m.values().any(|x| java_equals(x, &needle))))
        }),
        "keySet" => method(move |_| {
            Ok(Value::seq(
                m.keys().map(|k| Value::str(k.as_str())).collect(),
            ))
        }),
        "values" => method(move |_| Ok(Value::seq(m.values().cloned().collect()))),
        "entrySet" => method(move |_| {
            Ok(Value::seq(
                m.iter()
                    .map(|(k, v)| {
                        Value::object(Entry {
                            key: Value::str(k.as_str()),
                            value: v.clone(),
                        })
                    })
                    .collect(),
            ))
        }),
        "size" => method(move |_| Ok(Value::Num(m.len() as f64))),
        "isEmpty" => method(move |_| Ok(Value::Bool(m.is_empty()))),
        _ => None,
    }
}

fn number_method(n: f64, name: &str) -> Option<Value> {
    match name {
        "intValue" | "longValue" | "shortValue" | "byteValue" => {
            method(move |_| Ok(Value::Num(n.trunc())))
        }
        "doubleValue" | "floatValue" => method(move |_| Ok(Value::Num(n))),
        "toString" => method(move |_| {
            Ok(Value::str(if n.fract() == 0.0 && n.abs() < 1e16 {
                format!("{}", n as i64)
            } else {
                format!("{n}")
            }))
        }),
        _ => None,
    }
}

fn bool_method(b: bool, name: &str) -> Option<Value> {
    match name {
        "booleanValue" => method(move |_| Ok(Value::Bool(b))),
        "toString" => method(move |_| Ok(Value::str(b.to_string()))),
        _ => None,
    }
}

/// Entrada de un mapa (`Map.Entry`): `key`/`value` y `getKey()`/`getValue()`.
pub struct Entry {
    pub key: Value,
    pub value: Value,
}

impl HostObject for Entry {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "key" => Some(self.key.clone()),
            "value" => Some(self.value.clone()),
            "getKey" => {
                let k = self.key.clone();
                Some(Value::host_fn(move |_| Ok(k.clone())))
            }
            "getValue" => {
                let v = self.value.clone();
                Some(Value::host_fn(move |_| Ok(v.clone())))
            }
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "entrada de mapa"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(v: &Value, name: &str, args: &[Value]) -> Value {
        match member(v, name) {
            Some(Value::Fn(f)) => f(args).expect("llamada"),
            other => panic!("{name} no es un método: {other:?}"),
        }
    }

    #[test]
    fn strings_behave_like_java() {
        let s = Value::str("CUSTOM:RubyOre.block");
        assert_eq!(
            call(&s, "startsWith", &[Value::str("CUSTOM:")]),
            Value::Bool(true)
        );
        assert_eq!(
            call(&s, "replace", &[Value::str("CUSTOM:"), Value::str("")]),
            Value::str("RubyOre.block")
        );
        assert_eq!(
            call(&s, "split", &[Value::str("\\.")]),
            Value::seq(vec![Value::str("CUSTOM:RubyOre"), Value::str("block")])
        );
        assert_eq!(
            call(&Value::str("a,b,,"), "split", &[Value::str(",")]),
            Value::seq(vec![Value::str("a"), Value::str("b")])
        );
        assert_eq!(
            call(&Value::str("Ruby"), "hashCode", &[]),
            Value::Num(2558458.0)
        );
    }

    #[test]
    fn maps_expose_entries() {
        let mut m = Map::new();
        m.insert("a".into(), Value::Num(1.0));
        let v = Value::hash(m);
        let entries = call(&v, "entrySet", &[]);
        let Value::Seq(list) = entries else {
            panic!("entrySet no es lista")
        };
        assert_eq!(list[0].get("key"), Some(Value::str("a")));
        assert_eq!(call(&v, "keySet", &[]), Value::seq(vec![Value::str("a")]));
        assert_eq!(call(&v, "get", &[Value::str("a")]), Value::Num(1.0));
    }
}
