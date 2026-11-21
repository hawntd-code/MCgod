//! Tokens de las rutas de los generadores (`GeneratorTokens` de MCreator):
//! `@SRCROOT`, `@RESROOT`, `@MODASSETSROOT`, `@modid`, `@JavaModName`, etc.

use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Tokens {
    pub workspace_root: String,
    pub source_root: String,
    pub res_root: String,
    pub mod_assets_root: String,
    pub mod_data_root: String,
    pub modid: String,
    pub java_mod_name: String,
    pub mod_picture: String,
    pub mod_icon: String,
    pub base_package: String,
}

impl Tokens {
    /// Calcula las raíces a partir de `source_root`, `res_root`, `mod_assets_root` y
    /// `mod_data_root` de `generator.yaml`.
    pub fn new(
        workspace_root: &Path,
        roots: [&str; 4],
        modid: &str,
        java_mod_name: &str,
        base_package: &str,
        mod_picture: Option<&str>,
        mod_icon: Option<&str>,
    ) -> Tokens {
        let mut t = Tokens {
            workspace_root: slash(workspace_root),
            source_root: String::new(),
            res_root: String::new(),
            mod_assets_root: String::new(),
            mod_data_root: String::new(),
            modid: modid.to_string(),
            java_mod_name: java_mod_name.to_string(),
            mod_picture: mod_picture.unwrap_or_default().to_string(),
            mod_icon: mod_icon.unwrap_or_default().to_string(),
            base_package: base_package.to_string(),
        };
        t.source_root = t.replace(roots[0]);
        t.res_root = t.replace(roots[1]);
        t.mod_assets_root = t.replace(roots[2]);
        t.mod_data_root = t.replace(roots[3]);
        t
    }

    pub fn replace(&self, raw: &str) -> String {
        let mut s = raw.to_string();
        // El orden importa: las raíces más largas antes que sus prefijos.
        if s.contains("@SRCROOT") {
            s = s.replace("@SRCROOT", &self.source_root);
        }
        if s.contains("@RESROOT") {
            s = s.replace("@RESROOT", &self.res_root);
        }
        if s.contains("@MODASSETSROOT") {
            s = s.replace("@MODASSETSROOT", &self.mod_assets_root);
        }
        if s.contains("@MODDATAROOT") {
            s = s.replace("@MODDATAROOT", &self.mod_data_root);
        }
        s.replace("@WORKSPACEROOT", &self.workspace_root)
            .replace("@modid", &self.modid)
            .replace("@JavaModName", &self.java_mod_name)
            .replace("@modpicture", &self.mod_picture)
            .replace("@modicon", &self.mod_icon)
            .replace("@BASEPACKAGEPATH", &self.base_package.replace('.', "/"))
            .replace("@BASEPACKAGE", &self.base_package)
    }

    /// Ruta de un archivo de salida, o `None` si queda fuera del proyecto
    /// (MCreator descarta esas plantillas con `isFileInWorkspace`).
    pub fn path_in_workspace(&self, raw: &str) -> Option<PathBuf> {
        let full = normalize(Path::new(&self.replace(raw)));
        let root = normalize(Path::new(&self.workspace_root));
        full.starts_with(&root).then_some(full)
    }
}

fn slash(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Quita `.` y `..` sin tocar el disco.
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Nombre de clase Java del mod a partir del modid (`WorkspaceSettings.getJavaModName`):
/// `ruby_mod` → `RubyModMod`.
pub fn java_mod_name(modid: &str) -> String {
    let camel: String = modid
        .split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect();
    format!("{camel}Mod")
}

/// Nombre de registro a partir del nombre del elemento (`RegistryNameFixer.fromCamelCase`):
/// `RubyBlock` → `ruby_block`.
pub fn registry_name_from_camel_case(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower =
                i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if i > 0 && (prev_lower || (next_lower && chars[i - 1].is_uppercase())) {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else if c == ' ' || c == '-' {
            out.push('_');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Tokens {
        Tokens::new(
            Path::new("C:/mods/rubymod"),
            [
                "@WORKSPACEROOT/src/main/java",
                "@WORKSPACEROOT/src/main/resources",
                "@RESROOT/assets/@modid",
                "@RESROOT/data/@modid",
            ],
            "rubymod",
            "RubymodMod",
            "net.mcreator.rubymod",
            None,
            None,
        )
    }

    #[test]
    fn replaces_roots_and_names() {
        let t = tokens();
        assert_eq!(
            t.replace("@SRCROOT/@BASEPACKAGEPATH/init/@JavaModNameItems.java"),
            "C:/mods/rubymod/src/main/java/net/mcreator/rubymod/init/RubymodModItems.java"
        );
        assert_eq!(
            t.replace("@MODASSETSROOT/models/item/ruby.json"),
            "C:/mods/rubymod/src/main/resources/assets/rubymod/models/item/ruby.json"
        );
    }

    #[test]
    fn rejects_paths_outside_the_project() {
        let t = tokens();
        assert!(t.path_in_workspace("@WORKSPACEROOT/../evil.txt").is_none());
        assert!(t.path_in_workspace("@RESROOT/pack.mcmeta").is_some());
    }

    #[test]
    fn names_like_mcreator() {
        assert_eq!(java_mod_name("rubymod"), "RubymodMod");
        assert_eq!(java_mod_name("ruby_mod"), "RubyModMod");
        assert_eq!(registry_name_from_camel_case("RubyBlock"), "ruby_block");
        assert_eq!(registry_name_from_camel_case("Ruby"), "ruby");
        assert_eq!(registry_name_from_camel_case("TNTBlock"), "tnt_block");
    }
}
