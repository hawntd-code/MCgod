//! Caché del índice de clases de un proyecto, en `.mcreator/mcgod/` (la carpeta de
//! caché de MCreator, que los proyectos ya excluyen de Git).
//!
//! 1. Gradle escribe el classpath de compilación en `classpath.txt` (tarea
//!    `mcgodClasspath`, ver `mcgod_gradle::CLASSPATH_INIT_SCRIPT`).
//! 2. Con esos JAR y los módulos del JDK se arma `import-tree.json`.
//!
//! El classpath se vuelve a pedir cuando cambia la configuración de Gradle del proyecto.

use crate::java::ImportTree;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct IndexPaths {
    pub dir: PathBuf,
    pub init_script: PathBuf,
    pub classpath: PathBuf,
    pub tree: PathBuf,
}

/// Rutas absolutas: Gradle corre dentro del proyecto y una ruta relativa se duplicaría.
pub fn paths(workspace_root: &Path) -> IndexPaths {
    let root = std::path::absolute(workspace_root).unwrap_or_else(|_| workspace_root.to_path_buf());
    let dir = root.join(".mcreator").join("mcgod");
    IndexPaths {
        init_script: dir.join("classpath.gradle"),
        classpath: dir.join("classpath.txt"),
        tree: dir.join("import-tree.json"),
        dir,
    }
}

fn modified(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Archivos de Gradle que, si cambian, cambian el classpath.
const GRADLE_FILES: &[&str] = &[
    "build.gradle",
    "build.gradle.kts",
    "settings.gradle",
    "settings.gradle.kts",
    "gradle.properties",
    "mcreator.gradle",
];

/// `true` si hay que pedirle el classpath a Gradle (no existe o es más viejo que
/// la configuración del proyecto).
pub fn classpath_is_stale(workspace_root: &Path) -> bool {
    let p = paths(workspace_root);
    let Some(cp_time) = modified(&p.classpath) else {
        return true;
    };
    GRADLE_FILES
        .iter()
        .filter_map(|f| modified(&workspace_root.join(f)))
        .any(|t| t > cp_time)
}

/// Guarda el script de inicio que agrega la tarea `mcgodClasspath`.
pub fn write_init_script(workspace_root: &Path, script: &str) -> std::io::Result<PathBuf> {
    let p = paths(workspace_root);
    std::fs::create_dir_all(&p.dir)?;
    std::fs::write(&p.init_script, script)?;
    Ok(p.init_script)
}

/// JAR del classpath que escribió Gradle.
pub fn read_classpath(workspace_root: &Path) -> Option<Vec<PathBuf>> {
    let text = std::fs::read_to_string(paths(workspace_root).classpath).ok()?;
    Some(
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .collect(),
    )
}

pub struct LoadedTree {
    pub tree: ImportTree,
    /// `true` si se armó ahora (no venía de la caché).
    pub rebuilt: bool,
    pub problems: Vec<String>,
}

/// Índice de clases del proyecto: de la caché si sigue vigente, si no se arma con el
/// classpath y el JDK. `None` si todavía no hay classpath.
pub fn load_tree(workspace_root: &Path, java_home: Option<&Path>) -> Option<LoadedTree> {
    let p = paths(workspace_root);
    let cp_time = modified(&p.classpath)?;
    if modified(&p.tree).is_some_and(|t| t >= cp_time)
        && let Some(tree) = ImportTree::load(&p.tree)
        && !tree.is_empty()
    {
        return Some(LoadedTree {
            tree,
            rebuilt: false,
            problems: Vec::new(),
        });
    }
    let jars = read_classpath(workspace_root)?;
    let (tree, mut problems) = ImportTree::from_classpath(&jars, java_home);
    if let Err(e) = tree.save(&p.tree) {
        problems.push(format!("No se pudo guardar el índice de clases: {e}"));
    }
    Some(LoadedTree {
        tree,
        rebuilt: true,
        problems,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_until_classpath_exists_and_is_newer() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("build.gradle"), "").unwrap();
        assert!(classpath_is_stale(root));
        let p = paths(root);
        std::fs::create_dir_all(&p.dir).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&p.classpath, "").unwrap();
        assert!(!classpath_is_stale(root));
        assert!(load_tree(root, None).is_some_and(|t| t.tree.is_empty() && t.rebuilt));
    }
}
