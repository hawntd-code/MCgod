//! Escritura de JSON como `JSONWriter` de MCreator: se lee lo que produjo la plantilla
//! y se vuelve a escribir con el formato de Gson (`setPrettyPrinting`, dos espacios).
//! Si el JSON no es válido, se escribe tal cual (MCreator hace lo mismo y lo registra).

use serde_json::Value;

pub fn format(src: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(src).map_err(|e| e.to_string())?;
    Ok(pretty(&v))
}

/// Formato de Gson: dos espacios, `"clave": valor`, `[]` y `{}` vacíos en una línea.
pub fn pretty(v: &Value) -> String {
    let mut out = String::new();
    write(v, 0, &mut out);
    out
}

fn write(v: &Value, indent: usize, out: &mut String) {
    match v {
        Value::Array(a) if !a.is_empty() => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(indent + 1, out);
                write(x, indent + 1, out);
            }
            newline(indent, out);
            out.push(']');
        }
        Value::Object(o) if !o.is_empty() => {
            out.push('{');
            for (i, (k, x)) in o.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(indent + 1, out);
                string(k, out);
                out.push_str(": ");
                write(x, indent + 1, out);
            }
            newline(indent, out);
            out.push('}');
        }
        Value::Array(_) => out.push_str("[]"),
        Value::Object(_) => out.push_str("{}"),
        Value::String(s) => string(s, out),
        other => out.push_str(&other.to_string()),
    }
}

fn newline(indent: usize, out: &mut String) {
    out.push('\n');
    for _ in 0..indent {
        out.push_str("  ");
    }
}

/// Texto JSON como Gson sin escape HTML: comillas, barra invertida y controles.
fn string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_gson() {
        let src = r#"{
            "parent": "item/generated",
                "textures": {   "layer0": "rubymod:item/ruby" }, "overrides": [], "x": {}
        }"#;
        assert_eq!(
            format(src).unwrap(),
            "{\n  \"parent\": \"item/generated\",\n  \"textures\": {\n    \"layer0\": \"rubymod:item/ruby\"\n  },\n  \"overrides\": [],\n  \"x\": {}\n}"
        );
    }

    #[test]
    fn keeps_unicode_and_escapes_quotes() {
        let v: Value = serde_json::json!({"item.rubymod.ruby": "Rubí \"rojo\""});
        assert_eq!(
            pretty(&v),
            "{\n  \"item.rubymod.ruby\": \"Rubí \\\"rojo\\\"\"\n}"
        );
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(format("{ \"a\": }").is_err());
    }
}
