//! Índice de clases para los imports (`ImportTreeBuilder` de MCreator): nombre simple →
//! nombres completos, a partir de los JAR de la compilación del proyecto, de los
//! módulos del JDK (`jmods/*.jmod`) y de las clases del propio mod.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImportTree {
    pub classes: BTreeMap<String, Vec<String>>,
}

impl ImportTree {
    /// Agrega una clase por su nombre completo.
    pub fn add(&mut self, fqn: &str) {
        let simple = fqn.rsplit('.').next().unwrap_or(fqn).to_string();
        let list = self.classes.entry(simple).or_default();
        if !list.iter().any(|x| x == fqn) {
            list.push(fqn.to_string());
        }
    }

    pub fn len(&self) -> usize {
        self.classes.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    /// Clases públicas de un JAR o un JMOD.
    pub fn add_archive(&mut self, path: &Path) -> Result<usize, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let is_jmod = path.extension().is_some_and(|x| x == "jmod");
        // Un JMOD es un ZIP con 4 bytes de cabecera ("JM" 1 0) delante.
        let data = if is_jmod && bytes.starts_with(b"JM") {
            &bytes[4..]
        } else {
            &bytes[..]
        };
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(data))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let mut added = 0;
        for i in 0..zip.len() {
            let Ok(mut entry) = zip.by_index(i) else {
                continue;
            };
            let mut name = entry.name().to_string();
            if is_jmod {
                match name.strip_prefix("classes/") {
                    Some(rest) => name = rest.to_string(),
                    None => continue,
                }
            }
            if !wanted(&name) {
                continue;
            }
            let mut header = Vec::new();
            if entry.read_to_end(&mut header).is_err() || !is_public_class(&header) {
                continue;
            }
            let fqn = name.trim_end_matches(".class").replace(['/', '\\'], ".");
            self.add(&fqn);
            added += 1;
        }
        Ok(added)
    }

    /// Clases del mod: los `.java` bajo la raíz de código, por su ruta.
    pub fn add_sources(&mut self, source_root: &Path) {
        fn walk(dir: &Path, root: &Path, tree: &mut ImportTree) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for e in entries.filter_map(|e| e.ok()) {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, root, tree);
                } else if p.extension().is_some_and(|x| x == "java")
                    && let Some(fqn) = source_fqn(root, &p)
                {
                    tree.add(&fqn);
                }
            }
        }
        walk(source_root, source_root, self);
    }

    pub fn load(path: &Path) -> Option<ImportTree> {
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, serde_json::to_string(self).unwrap_or_default())
    }

    /// Índice de las librerías del proyecto (lista de JAR) y de los módulos del JDK.
    pub fn from_classpath(jars: &[PathBuf], java_home: Option<&Path>) -> (ImportTree, Vec<String>) {
        let mut tree = ImportTree::default();
        let mut problems = Vec::new();
        let mut archives: Vec<PathBuf> = jars
            .iter()
            .filter(|p| p.extension().is_some_and(|x| x == "jar") && p.is_file())
            .cloned()
            .collect();
        if let Some(home) = java_home {
            let jmods = home.join("jmods");
            match std::fs::read_dir(&jmods) {
                Ok(entries) => {
                    let mut mods: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.extension().is_some_and(|x| x == "jmod"))
                        .collect();
                    mods.sort();
                    archives.extend(mods);
                }
                Err(_) => problems.push(format!(
                    "El JDK en {} no tiene la carpeta jmods: faltarán los imports de Java (java.util…).",
                    home.display()
                )),
            }
        }
        for a in archives {
            if let Err(e) = tree.add_archive(&a) {
                problems.push(e);
            }
        }
        (tree, problems)
    }
}

/// Nombre completo de una clase del mod por su ruta bajo la raíz de código.
pub fn source_fqn(root: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(root).ok()?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    Some(rel.strip_suffix(".java")?.replace('/', "."))
}

fn wanted(name: &str) -> bool {
    name.ends_with(".class")
        && !name.contains('$')
        && !name.starts_with("META-INF/")
        && !name.starts_with("jdk/internal/")
        && !name.starts_with("sun/")
        && !name.starts_with("com/sun/")
        && !name.starts_with("org/antlr")
        && !name.starts_with("org/checkerframework")
        && !name.ends_with("package-info.class")
        && !name.ends_with("module-info.class")
}

/// Lee la cabecera de un `.class` y dice si la clase es pública o protegida.
/// Si no se puede leer, se toma como pública (igual que MCreator).
pub fn is_public_class(bytes: &[u8]) -> bool {
    let mut r = Reader { b: bytes, i: 0 };
    let mut parse = || -> Option<bool> {
        if r.u32()? != 0xCAFE_BABE {
            return None;
        }
        r.u16()?; // versión menor
        r.u16()?; // versión mayor
        let count = r.u16()?;
        let mut i = 1;
        while i < count {
            let tag = r.u8()?;
            match tag {
                1 => {
                    let len = r.u16()? as usize;
                    r.skip(len)?;
                }
                3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => r.skip(4)?,
                5 | 6 => {
                    r.skip(8)?;
                    i += 1;
                }
                7 | 8 | 16 | 19 | 20 => r.skip(2)?,
                15 => r.skip(3)?,
                _ => return None,
            }
            i += 1;
        }
        let flags = r.u16()?;
        Some(flags & 0x0001 != 0 || flags & 0x0004 != 0)
    };
    parse().unwrap_or(true)
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> Option<u8> {
        let v = *self.b.get(self.i)?;
        self.i += 1;
        Some(v)
    }
    fn u16(&mut self) -> Option<u16> {
        Some(((self.u8()? as u16) << 8) | self.u8()? as u16)
    }
    fn u32(&mut self) -> Option<u32> {
        Some(((self.u16()? as u32) << 16) | self.u16()? as u32)
    }
    fn skip(&mut self, n: usize) -> Option<()> {
        if self.i + n > self.b.len() {
            return None;
        }
        self.i += n;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cabecera mínima de un .class con un Utf8 en la tabla de constantes.
    fn class_bytes(flags: u16) -> Vec<u8> {
        let mut b = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 65, 0, 2, 1, 0, 1, b'A'];
        b.extend_from_slice(&flags.to_be_bytes());
        b
    }

    #[test]
    fn reads_access_flags() {
        assert!(is_public_class(&class_bytes(0x0021)));
        assert!(!is_public_class(&class_bytes(0x0020)));
        assert!(is_public_class(b"basura"));
    }

    #[test]
    fn filters_entries_like_mcreator() {
        assert!(wanted("net/minecraft/world/item/Item.class"));
        assert!(!wanted("net/minecraft/world/item/Item$Properties.class"));
        assert!(!wanted("module-info.class"));
        assert!(!wanted("sun/misc/Unsafe.class"));
        assert!(!wanted("META-INF/versions/9/a/B.class"));
    }

    #[test]
    fn reads_jars_and_sources() {
        let dir = tempfile::tempdir().unwrap();
        let jar = dir.path().join("lib.jar");
        {
            let f = std::fs::File::create(&jar).unwrap();
            let mut z = zip::ZipWriter::new(f);
            let opts = zip::write::SimpleFileOptions::default();
            use std::io::Write;
            z.start_file("net/minecraft/world/item/Item.class", opts)
                .unwrap();
            z.write_all(&class_bytes(0x0021)).unwrap();
            z.start_file("net/minecraft/world/item/Hidden.class", opts)
                .unwrap();
            z.write_all(&class_bytes(0x0020)).unwrap();
            z.finish().unwrap();
        }
        let src = dir.path().join("src");
        std::fs::create_dir_all(src.join("net/mcreator/rubymod/item")).unwrap();
        std::fs::write(src.join("net/mcreator/rubymod/item/RubyItem.java"), "").unwrap();

        let (mut tree, problems) = ImportTree::from_classpath(&[jar], None);
        assert!(problems.is_empty(), "{problems:?}");
        tree.add_sources(&src);
        assert_eq!(
            tree.classes.get("Item"),
            Some(&vec!["net.minecraft.world.item.Item".to_string()])
        );
        assert!(!tree.classes.contains_key("Hidden"));
        assert_eq!(
            tree.classes.get("RubyItem"),
            Some(&vec!["net.mcreator.rubymod.item.RubyItem".to_string()])
        );

        let cache = dir.path().join("cache/tree.json");
        tree.save(&cache).unwrap();
        assert_eq!(ImportTree::load(&cache).unwrap().len(), tree.len());
    }
}
