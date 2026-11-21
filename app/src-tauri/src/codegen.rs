//! Generación de código antes de compilar, como hace MCreator al pulsar Build o Run.
//!
//! 1. Si falta el índice de clases del proyecto (o cambió su configuración de Gradle),
//!    Gradle escribe el classpath (`mcgodClasspath`).
//! 2. El generador escribe el código y los recursos de los elementos que sabe generar.
//! 3. Después corre la tarea pedida (`build` o `runClient`).
//!
//! La interfaz recibe eventos `codegen` con el avance y un resumen que traduce.

use crate::commands::CommandError;
use mcgod_generator::{GenError, Generator, Note, Skipped, index};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Etapa de la generación que se informa a la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    /// Gradle está escribiendo el classpath para el índice de clases.
    Indexing,
    Generating,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub files: usize,
    pub written: usize,
    pub unchanged: usize,
    pub generated: Vec<String>,
    pub skipped: Vec<Skipped>,
    pub notes: Vec<Note>,
    pub errors: Vec<String>,
}

/// Evento `{"kind": "codegen", "stage": …}` del canal de la tarea.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename = "codegen", rename_all = "camelCase")]
pub struct CodegenEvent {
    pub stage: Stage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<Report>,
}

impl CodegenEvent {
    pub fn stage(stage: Stage) -> Self {
        CodegenEvent {
            stage,
            report: None,
        }
    }
}

/// Carpeta con los plugins de generación: `MCGOD_PLUGINS`, los recursos de la app
/// instalada, o la carpeta `plugins/` del repositorio en desarrollo.
pub fn plugins_dir(resource_dir: Option<PathBuf>) -> Option<PathBuf> {
    let candidates = [
        std::env::var_os("MCGOD_PLUGINS").map(PathBuf::from),
        resource_dir.map(|r| r.join("plugins")),
        Some(PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../plugins"
        ))),
    ];
    candidates.into_iter().flatten().find(|p| p.is_dir())
}

impl From<GenError> for CommandError {
    fn from(e: GenError) -> Self {
        let code = match &e {
            GenError::Plugins(_) => "noPlugins",
            GenError::UnknownGenerator(_) => "unknownGenerator",
            GenError::NoGenerator => "noGenerator",
            GenError::Write { .. } => "codegenWrite",
        };
        CommandError::new(code, e)
    }
}

/// Genera y escribe el código del proyecto. Necesita el índice de clases: sin él el
/// código Java no tendría imports y no compilaría.
pub fn generate(
    workspace_file: &Path,
    plugins: &Path,
    java_home: Option<&Path>,
) -> Result<Report, CommandError> {
    // Se relee del disco: la persona (o la IA) pudo cambiar elementos desde que se abrió.
    let ws = mcgod_workspace::Workspace::open(workspace_file)?;
    let generator_id = ws
        .data
        .workspace_settings
        .current_generator
        .clone()
        .ok_or(GenError::NoGenerator)?;
    let generator = Generator::load(plugins, &generator_id)?;
    let tree = index::load_tree(&ws.root, java_home)
        .ok_or_else(|| CommandError::new("noImportIndex", ws.root.display()))?;
    for p in &tree.problems {
        log::warn!("índice de clases: {p}");
    }
    let generation = generator.generate(&ws, Some(&tree.tree))?;
    let report = if generation.errors.is_empty() {
        generation.write()?
    } else {
        // Con plantillas fallidas no se escribe nada: el proyecto queda como estaba.
        Default::default()
    };
    log::info!(
        "código generado: {} archivos, {} escritos, {} errores",
        generation.files.len(),
        report.written,
        generation.errors.len()
    );
    Ok(Report {
        files: generation.files.len(),
        written: report.written,
        unchanged: report.unchanged,
        generated: generation.generated,
        skipped: generation.skipped,
        notes: generation.notes,
        errors: generation.errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy_example(to: &Path) {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rubymod");
        for f in ["rubymod.mcreator", "elements/Ruby.mod.json"] {
            let dest = to.join(f);
            std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
            std::fs::copy(src.join(f), dest).unwrap();
        }
    }

    #[test]
    fn needs_the_class_index_before_writing() {
        let dir = tempfile::tempdir().unwrap();
        copy_example(dir.path());
        let plugins = plugins_dir(None).expect("carpeta plugins del repositorio");
        let err = generate(&dir.path().join("rubymod.mcreator"), &plugins, None).unwrap_err();
        assert_eq!(err.code, "noImportIndex");
        assert!(!dir.path().join("src").exists());
    }

    #[test]
    fn generates_and_reports() {
        let dir = tempfile::tempdir().unwrap();
        copy_example(dir.path());
        // Classpath vacío: el índice solo tendrá las clases del mod, suficiente para generar.
        let p = index::paths(dir.path());
        std::fs::create_dir_all(&p.dir).unwrap();
        std::fs::write(&p.classpath, "").unwrap();

        let plugins = plugins_dir(None).unwrap();
        let report = generate(&dir.path().join("rubymod.mcreator"), &plugins, None).unwrap();
        assert_eq!(report.generated, ["Ruby"]);
        assert_eq!((report.files, report.written), (9, 9));
        assert!(report.errors.is_empty());
        assert!(
            dir.path()
                .join("src/main/java/net/mcreator/rubymod/item/RubyItem.java")
                .is_file()
        );

        let again = generate(&dir.path().join("rubymod.mcreator"), &plugins, None).unwrap();
        assert_eq!((again.written, again.unchanged), (0, 9));
    }

    #[test]
    fn events_serialize_for_the_ui() {
        let e = CodegenEvent::stage(Stage::Indexing);
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({"kind": "codegen", "stage": "indexing"})
        );
    }
}
