//! Bloques de código del usuario (`UserCodeProcessor`): lo que la persona escribe entre
//! `// Start of user code block X` y `// End of user code block X` se conserva al
//! volver a generar el archivo.

use std::collections::HashMap;

const START: &str = " Start of user code block ";
const END: &str = " End of user code block ";

/// Pasa los bloques del archivo actual (si existe) al código nuevo.
pub fn process(current: Option<&str>, new_code: &str, comment: &str) -> String {
    if !new_code.contains(START) || !new_code.contains(END) {
        return new_code.to_string();
    }
    let blocks = current.map(|c| blocks(c, comment)).unwrap_or_default();
    if blocks.is_empty() {
        return new_code.to_string();
    }
    let start = format!("{comment}{START}");
    let mut out = String::new();
    for line in new_code.lines() {
        let line = line.trim();
        out.push_str(line);
        out.push('\n');
        if let Some(name) = line.strip_prefix(&start)
            && let Some(content) = blocks.get(name.trim())
        {
            out.push_str(content);
        }
    }
    out
}

/// Contenido no vacío de cada bloque, por nombre.
pub fn blocks(code: &str, comment: &str) -> HashMap<String, String> {
    let mut found = HashMap::new();
    if !code.contains(START) || !code.contains(END) {
        return found;
    }
    let start = format!("{comment}{START}");
    let mut current: Option<String> = None;
    let mut content = String::new();
    for line in code.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix(&start) {
            current = Some(name.trim().to_string());
            content.clear();
        } else if let Some(name) = &current
            && line == format!("{comment}{END}{name}")
        {
            if !content.trim().is_empty() {
                found.insert(name.clone(), content.clone());
            }
            current = None;
        } else if current.is_some() {
            content.push_str(line);
            content.push('\n');
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_user_blocks() {
        let old = "class A {\n\t// Start of user code block custom items\n\tpublic static int X = 1;\n\t// End of user code block custom items\n}\n";
        let new = "class A {\n// Start of user code block custom items\n// End of user code block custom items\nint y;\n}";
        let out = process(Some(old), new, "//");
        assert_eq!(
            out,
            "class A {\n// Start of user code block custom items\npublic static int X = 1;\n// End of user code block custom items\nint y;\n}\n"
        );
    }

    #[test]
    fn without_blocks_returns_new_code() {
        assert_eq!(process(Some("x"), "class B {}", "//"), "class B {}");
        let new = "// Start of user code block a\n// End of user code block a";
        assert_eq!(process(None, new, "//"), new);
    }
}
