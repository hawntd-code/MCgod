//! Valores que maneja una plantilla.

use crate::FtlError;
use crate::ast::{Callable, Expr};
use indexmap::IndexMap;
use std::fmt;
use std::rc::Rc;

pub type Map = IndexMap<String, Value>;

/// Función del programa anfitrión (por ejemplo, `generator.map(...)`).
pub type HostFn = Rc<dyn Fn(&[Value]) -> Result<Value, FtlError>>;

/// Directiva del programa anfitrión que transforma la salida de su cuerpo
/// (por ejemplo, `<@javacompress>`).
pub type HostDirective = Rc<dyn Fn(&Map, String) -> Result<String, FtlError>>;

/// Objeto del programa anfitrión con propiedades y métodos (como un objeto Java
/// expuesto por el BeansWrapper de FreeMarker).
pub trait HostObject {
    /// Propiedad o método. Un método se devuelve como `Value::Fn`.
    fn get(&self, key: &str) -> Option<Value>;
    fn type_name(&self) -> &str {
        "objeto"
    }
}

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(Rc<str>),
    Seq(Rc<Vec<Value>>),
    Hash(Rc<Map>),
    Object(Rc<dyn HostObject>),
    Fn(HostFn),
    Directive(HostDirective),
    Macro(Rc<Callable>, usize),
    Lambda(String, Rc<Expr>),
    /// Resultado de `?interpret`: se ejecuta en el contexto de quien lo llama.
    Template(crate::ast::Body, Rc<str>),
}

impl Value {
    pub fn str(s: impl Into<Rc<str>>) -> Value {
        Value::Str(s.into())
    }

    pub fn seq(items: Vec<Value>) -> Value {
        Value::Seq(Rc::new(items))
    }

    pub fn hash(map: Map) -> Value {
        Value::Hash(Rc::new(map))
    }

    pub fn host_fn(f: impl Fn(&[Value]) -> Result<Value, FtlError> + 'static) -> Value {
        Value::Fn(Rc::new(f))
    }

    pub fn directive(f: impl Fn(&Map, String) -> Result<String, FtlError> + 'static) -> Value {
        Value::Directive(Rc::new(f))
    }

    pub fn object(o: impl HostObject + 'static) -> Value {
        Value::Object(Rc::new(o))
    }

    /// Convierte JSON (definiciones de elementos, ajustes) en valores de plantilla.
    pub fn from_json(v: &serde_json::Value) -> Value {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => Value::Num(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Value::str(s.as_str()),
            serde_json::Value::Array(a) => Value::seq(a.iter().map(Value::from_json).collect()),
            serde_json::Value::Object(o) => Value::hash(
                o.iter()
                    .map(|(k, v)| (k.clone(), Value::from_json(v)))
                    .collect(),
            ),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Null => "nada (valor indefinido)",
            Value::Bool(_) => "booleano",
            Value::Num(_) => "número",
            Value::Str(_) => "texto",
            Value::Seq(_) => "lista",
            Value::Hash(_) => "mapa",
            Value::Object(o) => o.type_name(),
            Value::Fn(_) => "función",
            Value::Directive(_) => "directiva",
            Value::Macro(c, _) if c.is_function => "función de plantilla",
            Value::Macro(..) => "macro",
            Value::Lambda(..) => "lambda",
            Value::Template(..) => "plantilla interpretada",
        }
    }

    /// Texto si es un texto. Para mensajes de error y comparaciones.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Clave o propiedad: mapas, objetos del anfitrión.
    pub fn get(&self, key: &str) -> Option<Value> {
        match self {
            Value::Hash(m) => m.get(key).cloned(),
            Value::Object(o) => o.get(key),
            _ => None,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Num(n) => write!(f, "{}", format_number(*n)),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Seq(s) => f.debug_list().entries(s.iter()).finish(),
            Value::Hash(m) => f.debug_map().entries(m.iter()).finish(),
            other => write!(f, "<{}>", other.type_name()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Num(a), Value::Num(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Seq(a), Value::Seq(b)) => a == b,
            (Value::Hash(a), Value::Hash(b)) => a == b,
            _ => false,
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::str(s)
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::str(s)
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Num(n)
    }
}
impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Num(n as f64)
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::seq(v)
    }
}

/// Formato de números de MCreator: `0.##########` (hasta 10 decimales, redondeo
/// al par como `DecimalFormat`, sin separador de miles ni notación científica).
pub fn format_number(n: f64) -> String {
    if !n.is_finite() {
        return if n.is_nan() {
            "NaN".into()
        } else if n > 0.0 {
            "∞".into()
        } else {
            "-∞".into()
        };
    }
    // Java redondea sobre los dígitos más cortos que representan el número,
    // los mismos que da `{:e}` en Rust.
    let sci = format!("{:e}", n.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let digits: String = mantissa.chars().filter(|c| c.is_ascii_digit()).collect();
    let point = 1 + exp; // posición del punto decimal dentro de `digits`
    let (mut int, mut frac) = if point <= 0 {
        (
            "0".to_string(),
            format!("{}{}", "0".repeat((-point) as usize), digits),
        )
    } else if point as usize >= digits.len() {
        (
            format!("{}{}", digits, "0".repeat(point as usize - digits.len())),
            String::new(),
        )
    } else {
        (
            digits[..point as usize].to_string(),
            digits[point as usize..].to_string(),
        )
    };

    const MAX: usize = 10;
    if frac.len() > MAX {
        let rest = frac.split_off(MAX);
        let first = rest.as_bytes()[0];
        let tail_nonzero = rest[1..].bytes().any(|b| b != b'0');
        let last_kept = frac
            .bytes()
            .last()
            .or_else(|| int.bytes().last())
            .unwrap_or(b'0');
        let round_up =
            first > b'5' || (first == b'5' && (tail_nonzero || (last_kept - b'0') % 2 == 1));
        if round_up {
            let mut all: Vec<u8> = format!("{int}{frac}").into_bytes();
            let mut i = all.len();
            loop {
                if i == 0 {
                    all.insert(0, b'1');
                    break;
                }
                i -= 1;
                if all[i] == b'9' {
                    all[i] = b'0';
                } else {
                    all[i] += 1;
                    break;
                }
            }
            let all = String::from_utf8(all).expect("dígitos ASCII");
            let split = all.len() - frac.len();
            int = all[..split].to_string();
            frac = all[split..].to_string();
        }
    }
    let frac = frac.trim_end_matches('0');
    let sign = if n.is_sign_negative() { "-" } else { "" };
    if frac.is_empty() {
        format!("{sign}{int}")
    } else {
        format!("{sign}{int}.{frac}")
    }
}

/// Formato "de computadora" (`?c`): sin redondear a 10 decimales ni separadores.
pub fn format_c_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e16 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_format_matches_mcreator() {
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(64.0), "64");
        assert_eq!(format_number(0.1), "0.1");
        assert_eq!(format_number(1.2), "1.2");
        assert_eq!(format_number(-48.0), "-48");
        assert_eq!(format_number(1.23456789012345), "1.2345678901");
        assert_eq!(format_number(0.00000000005), "0");
        assert_eq!(format_number(0.00000000015), "0.0000000002");
        assert_eq!(format_number(1000000.0), "1000000");
        assert_eq!(format_number(202500315219.0), "202500315219");
        assert_eq!(format_number(0.3), "0.3");
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(0.99999999999), "1");
        assert_eq!(format_number(9.99999999996), "10");
        assert_eq!(format_number(-0.0), "-0");
        assert_eq!(format_number(1e-7), "0.0000001");
    }

    #[test]
    fn c_format() {
        assert_eq!(format_c_number(3.0), "3");
        assert_eq!(format_c_number(0.25), "0.25");
    }
}
