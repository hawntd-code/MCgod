//! Genera el código de un proyecto de MCreator con el mismo código que usa la app.
//!
//! Uso:
//!   cargo run -p mcgod-generator --example generate -- <carpeta> [--write] [--print <ruta>] [--sin-gradle]
//!
//! Si falta el índice de clases (o cambió la configuración de Gradle), primero le pide
//! a Gradle el classpath del proyecto, salvo con `--sin-gradle`. Sin `--write` solo
//! muestra lo que generaría.

use mcgod_generator::{Generator, index};
use mcgod_gradle::{GradleEvent, classpath_request, java, start};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(folder) = args.first() else {
        eprintln!("uso: generate <carpeta> [--write] [--print <ruta>]");
        std::process::exit(2);
    };
    let write = args.iter().any(|a| a == "--write");
    let print = args
        .iter()
        .position(|a| a == "--print")
        .and_then(|i| args.get(i + 1));

    let ws = mcgod_workspace::Workspace::open(folder).expect("no se pudo abrir el proyecto");
    let plugins = std::env::var("MCGOD_PLUGINS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins"));
    let generator_id = ws
        .data
        .workspace_settings
        .current_generator
        .clone()
        .expect("el proyecto no indica su generador");
    let generator =
        Generator::load(&plugins, &generator_id).expect("no se pudo cargar el generador");

    let java = java::detect();
    let java_home = java.as_ref().and_then(|j| j.home_path());

    let skip_gradle = args.iter().any(|a| a == "--sin-gradle");
    if !skip_gradle && index::classpath_is_stale(&ws.root) {
        println!("[mcgod] pidiendo el classpath a Gradle…");
        let script = index::write_init_script(&ws.root, mcgod_gradle::CLASSPATH_INIT_SCRIPT)
            .expect("no se pudo escribir el script de Gradle");
        let p = index::paths(&ws.root);
        let req = classpath_request(&ws.root, &script, &p.classpath, java_home.clone(), 2048);
        let handle = start(&req, |event| match event {
            GradleEvent::Lines { lines } => {
                for l in lines {
                    println!("  {}", l.text);
                }
            }
            GradleEvent::Finished { outcome, .. } => println!("[mcgod] Gradle: {outcome:?}"),
        })
        .expect("no se pudo iniciar Gradle");
        handle.wait();
    }

    let started = std::time::Instant::now();
    let tree = index::load_tree(&ws.root, java_home.as_deref());
    if let Some(t) = &tree {
        println!(
            "[mcgod] índice de clases: {} clases{} en {} ms",
            t.tree.len(),
            if t.rebuilt { " (armado ahora)" } else { "" },
            started.elapsed().as_millis()
        );
        for p in &t.problems {
            println!("[mcgod] aviso: {p}");
        }
    }

    let started = std::time::Instant::now();
    let generation = generator
        .generate(&ws, tree.as_ref().map(|t| &t.tree))
        .expect("falló la generación");
    println!(
        "[mcgod] {} archivos en {} ms",
        generation.files.len(),
        started.elapsed().as_millis()
    );
    for f in &generation.files {
        println!(
            "  {:<70} {}",
            f.relative,
            f.element.as_deref().unwrap_or("(mod)")
        );
    }
    for s in &generation.skipped {
        println!(
            "[mcgod] omitido: {} ({}) {:?}",
            s.name, s.element_type, s.reason
        );
    }
    for n in &generation.notes {
        match &n.detail {
            Some(d) => println!("[mcgod] nota: {:?} ({d})", n.code),
            None => println!("[mcgod] nota: {:?}", n.code),
        }
    }
    for e in &generation.errors {
        println!("[mcgod] ERROR: {e}");
    }
    if let Some(rel) = print {
        match generation.files.iter().find(|f| &f.relative == rel) {
            Some(f) => println!("----- {rel}\n{}\n-----", f.contents),
            None => println!("[mcgod] no se generó {rel}"),
        }
    }
    if write && tree.is_none() {
        eprintln!("[mcgod] sin índice de clases el código Java no compilaría: no se escribe nada");
        std::process::exit(1);
    }
    if write {
        let r = generation
            .write()
            .expect("no se pudieron escribir los archivos");
        println!(
            "[mcgod] escritos: {}, sin cambios: {}",
            r.written, r.unchanged
        );
    }
    if !generation.errors.is_empty() {
        std::process::exit(1);
    }
}
