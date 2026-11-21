//! Ejecuta Gradle sobre un workspace con el mismo código que usa la app.
//!
//! Uso:
//!   cargo run -p mcgod-gradle --example gradle -- <carpeta> <tarea> [segundos-antes-de-detener]
//!
//! Ejemplo: compilar el proyecto de ejemplo
//!   cargo run -p mcgod-gradle --example gradle -- examples/rubymod build
//! Ejemplo: abrir Minecraft y cerrarlo a los 120 s
//!   cargo run -p mcgod-gradle --example gradle -- examples/rubymod runClient 120

use mcgod_gradle::{GradleEvent, GradleRequest, Outcome, ensure_mcreator_gradle, java, start};
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("uso: gradle <carpeta> <tarea> [segundos-antes-de-detener]");
        std::process::exit(2);
    }
    let root = PathBuf::from(&args[0]);
    let stop_after = args.get(2).and_then(|s| s.parse::<u64>().ok());

    let java = java::detect();
    println!("[mcgod] java: {java:?}");
    if ensure_mcreator_gradle(&root).unwrap_or(false) {
        println!("[mcgod] se creó mcreator.gradle vacío");
    }

    let req = GradleRequest {
        root,
        tasks: vec![args[1].clone()],
        java_home: java.and_then(|j| j.home_path()),
        max_heap_mb: 2048,
        extra_args: Vec::new(),
    };
    let handle = start(&req, |event| match event {
        GradleEvent::Lines { lines } => {
            for l in lines {
                println!("{}", l.text);
            }
        }
        GradleEvent::Finished {
            outcome,
            exit_code,
            duration_ms,
        } => {
            println!("[mcgod] terminado: {outcome:?} código={exit_code:?} en {duration_ms} ms");
        }
    })
    .expect("no se pudo iniciar Gradle");
    println!("[mcgod] pid {}", handle.pid());

    if let Some(secs) = stop_after {
        let deadline = std::time::Instant::now() + Duration::from_secs(secs);
        while !handle.is_finished() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(250));
        }
        if !handle.is_finished() {
            println!("[mcgod] deteniendo después de {secs} s");
            handle.stop().expect("no se pudo detener");
        }
    }
    handle.wait();
    let _ = Outcome::Success;
}
