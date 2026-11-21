//! Analiza todas las plantillas de los plugins de MCreator, si su código está en `reference/`.
//! En la integración continua no está la copia de MCreator, así que la prueba se omite.

use mcgod_ftl::{Engine, MemLoader};
use std::path::{Path, PathBuf};

fn ftl_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            ftl_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "ftl") {
            out.push(p);
        }
    }
}

#[test]
fn parses_every_mcreator_template() {
    let plugins = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/MCreator/plugins");
    if !plugins.is_dir() {
        eprintln!("sin reference/MCreator: se omite");
        return;
    }
    let mut files = Vec::new();
    ftl_files(&plugins, &mut files);
    let engine = Engine::new(MemLoader::default());
    let mut failures = Vec::new();
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap();
        let name = f.strip_prefix(&plugins).unwrap().display().to_string();
        if let Err(e) = engine.parse(&name, &src) {
            failures.push(format!("{e}"));
        }
    }
    eprintln!(
        "plantillas analizadas: {}, con error: {}",
        files.len(),
        failures.len()
    );
    for f in failures.iter().take(40) {
        eprintln!("  {f}");
    }
    assert!(files.len() > 1000, "se esperaban más de 1000 plantillas");
    assert!(
        failures.is_empty(),
        "{} plantillas no se pudieron analizar",
        failures.len()
    );
}
