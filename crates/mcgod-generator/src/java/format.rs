//! Formato del código Java generado, con el estilo de MCreator: tabuladores, las
//! sentencias partidas en varias líneas se unen (`join_wrapped_lines`) y hay una línea
//! en blanco entre métodos. No es el formateador de Eclipse, pero el resultado se lee igual.

use super::mask;

const MAX_LINE: usize = 250;

struct Line {
    raw: String,
    masked: String,
}

pub fn format(code: &str) -> String {
    let masked = mask(code);
    let lines: Vec<Line> = code
        .split('\n')
        .zip(masked.split('\n'))
        .map(|(r, m)| Line {
            raw: r.trim().to_string(),
            masked: m.trim().to_string(),
        })
        .collect();
    let joined = join_wrapped(lines);
    indent(joined)
}

fn net_parens(masked: &str) -> i32 {
    masked.chars().fold(0, |n, c| match c {
        '(' | '[' => n + 1,
        ')' | ']' => n - 1,
        _ => n,
    })
}

fn has_comment(masked: &str) -> bool {
    masked.contains('#')
}

/// Une las líneas de una sentencia que quedó abierta en un paréntesis, salvo que
/// abra un bloque o tenga comentarios.
fn join_wrapped(lines: Vec<Line>) -> Vec<Line> {
    let mut out: Vec<Line> = Vec::new();
    let mut open = 0;
    for line in lines {
        if line.raw.is_empty() && open > 0 {
            continue;
        }
        let continued = out
            .last()
            .is_some_and(|prev: &Line| ends_with_operator(&prev.masked));
        let joinable = (open > 0 || continued)
            && out.last().is_some_and(|prev: &Line| {
                !prev.masked.ends_with('{')
                    && !has_comment(&prev.masked)
                    && !has_comment(&line.masked)
                    && prev.raw.len() + line.raw.len() < MAX_LINE
            });
        if joinable {
            let prev = out.last_mut().expect("línea anterior");
            let glue = needs_space(&prev.raw, &line.raw);
            if glue {
                prev.raw.push(' ');
                prev.masked.push(' ');
            }
            prev.raw.push_str(&line.raw);
            prev.masked.push_str(&line.masked);
            open = net_parens(&prev.masked).max(0);
        } else {
            open = net_parens(&line.masked).max(0);
            out.push(line);
        }
        if out.last().is_some_and(|l| l.masked.ends_with('{')) {
            open = 0;
        }
    }
    out
}

/// Una sentencia que sigue en la línea siguiente: `RUBY =`, `a +`, `x &&`, `() ->`.
fn ends_with_operator(masked: &str) -> bool {
    let t = masked.trim_end();
    ["=", "+", "&&", "||", "?", "->"]
        .iter()
        .any(|op| t.ends_with(op))
        && !t.ends_with("==")
        && !t.ends_with("!=")
}

fn needs_space(prev: &str, next: &str) -> bool {
    let p = prev.chars().last().unwrap_or(' ');
    let n = next.chars().next().unwrap_or(' ');
    !(matches!(p, '(' | '[' | '.' | '!') || matches!(n, '.' | ')' | ']' | ',' | ';' | '('))
}

fn leading_closers(masked: &str) -> i32 {
    masked
        .chars()
        .take_while(|c| matches!(c, '}' | ')' | ']'))
        .count() as i32
}

fn net_depth(masked: &str) -> i32 {
    masked.chars().fold(0, |n, c| match c {
        '{' | '(' | '[' => n + 1,
        '}' | ')' | ']' => n - 1,
        _ => n,
    })
}

/// `if (…)`, `for (…)`, `while (…)` o `else` sin llave: la línea siguiente es su cuerpo.
fn is_braceless_header(masked: &str) -> bool {
    let t = masked.trim_start_matches('}').trim();
    if t == "else" {
        return true;
    }
    let keyword = ["if", "else if", "for", "while"]
        .iter()
        .any(|k| t.starts_with(&format!("{k} (")) || t.starts_with(&format!("{k}(")));
    keyword && t.ends_with(')') && net_parens(t) == 0
}

fn is_type_declaration(masked: &str) -> bool {
    let words: Vec<&str> = masked
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .collect();
    masked.ends_with('{')
        && words
            .iter()
            .any(|w| matches!(*w, "class" | "interface" | "enum" | "record"))
}

/// Sangra por profundidad de llaves y paréntesis, quita líneas vacías de más y
/// separa los miembros de cada clase con una línea en blanco.
fn indent(lines: Vec<Line>) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut depth: i32 = 0;
    // Profundidades a las que están los miembros de una clase.
    let mut member_levels: Vec<i32> = Vec::new();
    let mut prev_masked = String::new();
    let mut prev_closed_member = false;
    let mut pending_blank = false;

    for line in lines {
        if line.raw.is_empty() {
            pending_blank = !out.is_empty();
            continue;
        }
        let mut level = (depth - leading_closers(&line.masked)).max(0);
        if is_braceless_header(&prev_masked) && !line.masked.starts_with('{') {
            level += 1;
        }
        let at_member_level = member_levels.last() == Some(&level);
        let starts_closing = line.masked.starts_with('}');

        let mut blank = pending_blank;
        if prev_masked.ends_with('{') || starts_closing {
            blank = false;
        }
        if at_member_level && !starts_closing && !prev_masked.starts_with('@') {
            let member_start = line.masked.starts_with('@') || line.masked.ends_with('{');
            if prev_closed_member || (member_start && !prev_masked.ends_with('{')) {
                blank = true;
            }
        }
        if blank && !out.is_empty() && out.last().is_some_and(|l| !l.is_empty()) {
            out.push(String::new());
        }
        pending_blank = false;

        let mut text = String::new();
        for _ in 0..level {
            text.push('\t');
        }
        if line.raw.starts_with('*') {
            text.push(' ');
        }
        text.push_str(&line.raw);
        out.push(text);

        if is_type_declaration(&line.masked) {
            member_levels.push(level + 1);
        }
        depth = (depth + net_depth(&line.masked)).max(0);
        while member_levels.last().is_some_and(|l| *l > depth) {
            member_levels.pop();
        }
        prev_closed_member = starts_closing
            && line.masked.trim_end_matches(';').trim_end_matches(')') == "}"
            && member_levels.last() == Some(&depth);
        prev_masked = line.masked;
    }
    let mut s = out.join("\n");
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indents_joins_and_separates_methods() {
        let src = "public class RubyItem extends Item {\npublic RubyItem() {\nsuper(new Item.Properties()\n.fireResistant()\n.rarity(Rarity.UNCOMMON)\n);\n}\n@Override public int getEnchantmentValue() {\nreturn 3;\n}\n\n\n\n}\n";
        assert_eq!(
            format(src),
            "public class RubyItem extends Item {\n\tpublic RubyItem() {\n\t\tsuper(new Item.Properties().fireResistant().rarity(Rarity.UNCOMMON));\n\t}\n\n\t@Override public int getEnchantmentValue() {\n\t\treturn 3;\n\t}\n}\n"
        );
    }

    #[test]
    fn keeps_fields_together_and_blocks_open() {
        let src = "public class A {\npublic static final int X = 1;\npublic static final int Y = 2;\nstatic {\nfoo(() -> {\nbar();\n});\n}\n}";
        assert_eq!(
            format(src),
            "public class A {\n\tpublic static final int X = 1;\n\tpublic static final int Y = 2;\n\n\tstatic {\n\t\tfoo(() -> {\n\t\t\t\tbar();\n\t\t});\n\t}\n}\n"
        );
    }

    #[test]
    fn joins_assignments_split_after_the_operator() {
        let src = "class A {\nstatic {\nRUBY =\nREGISTRY.register(\"ruby\", RubyItem::new);\n}\n}";
        assert!(format(src).contains("\t\tRUBY = REGISTRY.register(\"ruby\", RubyItem::new);\n"));
    }

    #[test]
    fn indents_the_body_of_braceless_ifs() {
        let src =
            "class A {\nvoid f() {\nif (x)\nthrow new IllegalStateException(\"no\");\ny();\n}\n}";
        assert!(
            format(src)
                .contains("\t\tif (x)\n\t\t\tthrow new IllegalStateException(\"no\");\n\t\ty();\n")
        );
    }

    #[test]
    fn comments_are_not_joined() {
        let src = "class A {\nvoid f() {\ng(1, // uno\n2);\n}\n}";
        let out = format(src);
        assert!(out.contains("g(1, // uno\n\t\t\t2);"), "{out}");
    }
}
