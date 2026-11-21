//! Código Java generado: imports y formato.
//!
//! Las plantillas de MCreator no escriben imports. MCreator los agrega después
//! (`ImportFormat`): busca cada nombre que el código usa en un índice de todas las
//! clases del proyecto y sus librerías, y resuelve los nombres repetidos con las
//! reglas del generador. Después formatea con Eclipse; aquí hay un formateador más
//! simple con el mismo estilo general (tabuladores, una línea por sentencia).

pub mod classpath;
pub mod format;

use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};

pub use classpath::ImportTree;

/// Palabras reservadas de Java (`JavaConventions.JAVA_RESERVED_WORDS`).
pub const RESERVED: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "void",
    "volatile",
    "while",
];

/// Reemplaza el contenido de textos y caracteres por espacios y los comentarios por
/// `#` (que no aparece en código Java), sin cambiar la longitud ni los saltos de línea.
pub fn mask(code: &str) -> String {
    #[derive(PartialEq)]
    enum S {
        Code,
        Line,
        Block,
        Str,
        Char,
        TextBlock,
    }
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::with_capacity(code.len());
    let mut s = S::Code;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match s {
            S::Code => {
                if c == '/' && next == Some('/') {
                    s = S::Line;
                    out.push_str("##");
                    i += 2;
                    continue;
                } else if c == '/' && next == Some('*') {
                    s = S::Block;
                    out.push_str("##");
                    i += 2;
                    continue;
                } else if c == '"' && next == Some('"') && chars.get(i + 2) == Some(&'"') {
                    s = S::TextBlock;
                    out.push_str("\"\"\"");
                    i += 3;
                    continue;
                } else if c == '"' {
                    s = S::Str;
                } else if c == '\'' {
                    s = S::Char;
                }
                out.push(c);
            }
            S::Line => {
                if c == '\n' {
                    s = S::Code;
                    out.push('\n');
                } else {
                    out.push('#');
                }
            }
            S::Block => {
                if c == '*' && next == Some('/') {
                    s = S::Code;
                    out.push_str("##");
                    i += 2;
                    continue;
                }
                out.push(if c == '\n' { '\n' } else { '#' });
            }
            S::Str | S::Char => {
                let quote = if s == S::Str { '"' } else { '\'' };
                if c == '\\' && next.is_some() {
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                if c == quote {
                    s = S::Code;
                    out.push(c);
                } else if c == '\n' {
                    // Texto sin cerrar: termina en la línea, como el escáner de MCreator.
                    s = S::Code;
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
            S::TextBlock => {
                if c == '"' && next == Some('"') && chars.get(i + 2) == Some(&'"') {
                    s = S::Code;
                    out.push_str("\"\"\"");
                    i += 3;
                    continue;
                }
                out.push(if c == '\n' { '\n' } else { ' ' });
            }
        }
        i += 1;
    }
    out
}

fn ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Nombres sin calificar que usa el código: identificadores que no son palabras
/// reservadas ni van después de un punto (`JavaMemberExtractor.getMemberList`).
pub fn members(code: &str) -> HashSet<String> {
    let masked = mask(code);
    let mut found = HashSet::new();
    let mut name = String::new();
    let mut prev = ' ';
    for c in masked.chars() {
        if name.is_empty() {
            if ident_start(c) && !ident_part(prev) && prev != '.' {
                name.push(c);
            }
        } else if ident_part(c) {
            name.push(c);
        } else {
            if !RESERVED.contains(&name.as_str()) {
                found.insert(std::mem::take(&mut name));
            }
            name.clear();
        }
        prev = c;
    }
    if !name.is_empty() && !RESERVED.contains(&name.as_str()) {
        found.insert(name);
    }
    found
}

/// Reglas del generador para elegir entre clases con el mismo nombre.
#[derive(Default, Clone)]
pub struct ImportRules {
    /// Paquete base del mod (`net.mcreator.rubymod`).
    pub project_package: String,
    /// Prefijos de paquetes aceptados cuando hay nombres repetidos.
    pub duplicates_whitelist: Vec<String>,
    /// Clase → nombre completo preferido (`Nullable` → `javax.annotation.Nullable`).
    pub priority: HashMap<String, String>,
}

/// Código separado en: lo que va antes de `package` (comentarios), `package`,
/// imports existentes y el resto.
struct Parts {
    preamble: String,
    package: Option<String>,
    imports: Vec<String>,
    static_imports: Vec<String>,
    body: String,
}

fn split(code: &str) -> Parts {
    let masked = mask(code);
    let package_re = Regex::new(r"^\s*package\s+([\w.]+)\s*;").expect("regex");
    let import_re =
        Regex::new(r"^\s*import\s+(static\s+)?([\w.]+?)(\.\*)?\s*;\s*$").expect("regex");
    let mut preamble = String::new();
    let mut package = None;
    let mut imports = Vec::new();
    let mut static_imports = Vec::new();
    let mut body = String::new();
    let mut in_header = true;
    let raw_lines: Vec<&str> = code.split('\n').collect();
    let masked_lines: Vec<&str> = masked.split('\n').collect();
    let has_package = masked_lines.iter().any(|m| package_re.is_match(m));
    for (i, raw) in raw_lines.iter().enumerate() {
        let m = masked_lines.get(i).copied().unwrap_or("");
        if in_header {
            if package.is_none()
                && let Some(c) = package_re.captures(m)
            {
                package = Some(c[1].to_string());
                continue;
            }
            // Comentarios antes de `package`: se quedan arriba, como en ImportFormat.
            if has_package && package.is_none() {
                preamble.push_str(raw.trim_end());
                preamble.push('\n');
                continue;
            }
            if let Some(c) = import_re.captures(m) {
                let name = c[2].to_string();
                if c.get(1).is_some() {
                    static_imports.push(format!("{name}{}", c.get(3).map_or("", |m| m.as_str())));
                } else if c.get(3).is_none() {
                    imports.push(name);
                }
                continue;
            }
            let t = m.trim();
            if !t.is_empty() && !t.chars().all(|c| c == '#') {
                in_header = false;
            }
        }
        body.push_str(raw);
        body.push('\n');
    }
    Parts {
        preamble,
        package,
        imports,
        static_imports,
        body,
    }
}

/// Agrega los imports que el código necesita, como `ImportFormat.arrangeImports`,
/// y lo devuelve con formato.
pub fn arrange_imports(code: &str, tree: &ImportTree, rules: &ImportRules) -> String {
    let parts = split(code);
    let used = members(&parts.body);

    let mut candidates: BTreeSet<String> = BTreeSet::new();
    for m in &used {
        if let Some(list) = tree.classes.get(m) {
            candidates.extend(list.iter().cloned());
        }
    }
    for imp in &parts.imports {
        if used.contains(simple_name(imp)) {
            candidates.insert(imp.clone());
        }
    }
    let chosen = resolve_duplicates(candidates, &parts.imports, parts.package.as_deref(), rules);

    let mut out = String::new();
    let preamble = parts.preamble.trim();
    if !preamble.is_empty() {
        out.push_str(preamble);
        out.push_str("\n\n");
    }
    if let Some(p) = &parts.package {
        out.push_str(&format!("package {p};\n\n"));
    }
    let mut sorted: Vec<&String> = chosen.iter().collect();
    sorted.sort_by(|a, b| b.cmp(a));
    let mut prev_group: Option<String> = None;
    for imp in &sorted {
        let group: String = imp.split('.').take(2).collect();
        if prev_group.as_ref().is_some_and(|g| *g != group) {
            out.push('\n');
        }
        prev_group = Some(group);
        out.push_str(&format!("import {imp};\n"));
    }
    for imp in &parts.static_imports {
        out.push_str(&format!("import static {imp};\n"));
    }
    if !sorted.is_empty() || !parts.static_imports.is_empty() {
        out.push('\n');
    }
    out.push_str(&format::format(&parts.body));
    out
}

fn simple_name(fqn: &str) -> &str {
    fqn.rsplit('.').next().unwrap_or(fqn)
}

fn package_of(fqn: &str) -> &str {
    fqn.rfind('.').map(|i| &fqn[..i]).unwrap_or("")
}

/// `ImportFormat.cleanupAndResolveDuplicates`.
fn resolve_duplicates(
    candidates: BTreeSet<String>,
    existing: &[String],
    current_package: Option<&str>,
    rules: &ImportRules,
) -> BTreeSet<String> {
    let java_lang = Regex::new(r"^java\.lang\.[A-Z]\w+$").expect("regex");
    let project_prefix = format!("{}.", rules.project_package);
    let mut remove: HashSet<String> = HashSet::new();
    let mut by_class: HashMap<&str, Vec<&String>> = HashMap::new();
    for imp in &candidates {
        by_class.entry(simple_name(imp)).or_default().push(imp);
        if java_lang.is_match(imp) {
            remove.insert(imp.clone());
        }
        if current_package.is_some_and(|p| package_of(imp) == p) {
            remove.insert(imp.clone());
        }
    }
    for (class, options) in by_class.iter().filter(|(_, o)| o.len() > 1) {
        // 1. Un import que ya estaba en el archivo decide.
        if let Some(kept) = existing.iter().find(|e| simple_name(e) == *class) {
            remove.extend(options.iter().filter(|o| *o != &kept).map(|o| (*o).clone()));
            continue;
        }
        // 2. Una clase del propio mod gana.
        if !rules.project_package.is_empty()
            && options.iter().any(|o| o.starts_with(&project_prefix))
        {
            remove.extend(
                options
                    .iter()
                    .filter(|o| !o.starts_with(&project_prefix))
                    .map(|o| (*o).clone()),
            );
            continue;
        }
        // 3. La preferida del generador.
        if let Some(preferred) = rules.priority.get(*class) {
            remove.extend(
                options
                    .iter()
                    .filter(|o| *o != &preferred)
                    .map(|o| (*o).clone()),
            );
            continue;
        }
        // 4. Solo las de los paquetes aceptados.
        remove.extend(
            options
                .iter()
                .filter(|o| {
                    !rules
                        .duplicates_whitelist
                        .iter()
                        .any(|w| o.starts_with(w.as_str()))
                })
                .map(|o| (*o).clone()),
        );
    }
    candidates
        .into_iter()
        .filter(|c| !remove.contains(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> ImportTree {
        let mut t = ImportTree::default();
        for fqn in [
            "net.minecraft.world.item.Item",
            "net.minecraft.world.item.Rarity",
            "java.util.List",
            "java.awt.List",
            "java.lang.String",
            "javax.annotation.Nullable",
            "org.jetbrains.annotations.Nullable",
            "net.neoforged.neoforge.registries.DeferredRegister",
            "net.neoforged.neoforge.registries.DeferredItem",
            "net.mcreator.rubymod.item.RubyItem",
            "net.mcreator.rubymod.RubymodMod",
            "org.apache.logging.log4j.Level",
            "net.minecraft.world.level.Level",
        ] {
            t.add(fqn);
        }
        t
    }

    fn rules() -> ImportRules {
        ImportRules {
            project_package: "net.mcreator.rubymod".into(),
            duplicates_whitelist: vec!["java.util".into(), "net.minecraft".into()],
            priority: [(
                "Nullable".to_string(),
                "javax.annotation.Nullable".to_string(),
            )]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn members_skip_strings_comments_and_qualified_names() {
        let m = members(
            "public class A extends Item { // Rarity\n String s = \"List\"; new Item.Properties(); @Nullable Level l; }",
        );
        assert!(m.contains("Item"));
        assert!(m.contains("Nullable"));
        assert!(m.contains("Level"));
        assert!(m.contains("A"));
        assert!(!m.contains("Rarity"));
        assert!(!m.contains("List"));
        assert!(!m.contains("Properties"));
        assert!(!m.contains("public"));
    }

    #[test]
    fn adds_and_groups_imports_like_mcreator() {
        let code = "package net.mcreator.rubymod.init;\n\npublic class RubymodModItems {\npublic static final DeferredRegister.Items REGISTRY = DeferredRegister.createItems(RubymodMod.MODID);\npublic static final DeferredItem<Item> RUBY = REGISTRY.register(\"ruby\", RubyItem::new);\nList<String> names; @Nullable Level level;\n}\n";
        let out = arrange_imports(code, &tree(), &rules());
        let expected_imports = "package net.mcreator.rubymod.init;\n\n\
import net.neoforged.neoforge.registries.DeferredRegister;\n\
import net.neoforged.neoforge.registries.DeferredItem;\n\
\n\
import net.minecraft.world.level.Level;\n\
import net.minecraft.world.item.Item;\n\
\n\
import net.mcreator.rubymod.item.RubyItem;\n\
import net.mcreator.rubymod.RubymodMod;\n\
\n\
import javax.annotation.Nullable;\n\
\n\
import java.util.List;\n\n";
        assert!(out.starts_with(expected_imports), "{out}");
        assert!(!out.contains("java.lang.String"));
        assert!(!out.contains("java.awt"));
        assert!(!out.contains("log4j"));
    }

    #[test]
    fn existing_imports_win() {
        let code = "package a;\nimport org.jetbrains.annotations.Nullable;\nclass B { @Nullable Object o; }\n";
        let out = arrange_imports(code, &tree(), &rules());
        assert!(out.contains("import org.jetbrains.annotations.Nullable;"));
        assert!(!out.contains("javax.annotation"));
    }

    #[test]
    fn mask_keeps_length() {
        let src = "a(\"x//y\", 'c'); // fin\n/* b\n c */ d";
        let m = mask(src);
        assert_eq!(m.chars().count(), src.chars().count());
        assert_eq!(m.lines().count(), src.lines().count());
        assert!(!m.contains("x//y"));
        assert!(m.contains(" d"));
    }
}
