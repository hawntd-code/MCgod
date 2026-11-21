//! Pruebas del transporte de salida y de la detención, con comandos simples en lugar de Gradle.
//! La prueba con Gradle real es el ejemplo `cargo run -p mcgod-gradle --example gradle`.

use mcgod_gradle::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn collect() -> (
    Arc<Mutex<Vec<GradleEvent>>>,
    impl Fn(GradleEvent) + Send + 'static,
) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = events.clone();
    (events, move |e| sink_events.lock().unwrap().push(e))
}

fn piped(mut cmd: Command) -> Command {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

fn shell(script: &str) -> Command {
    if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", script]);
        piped(c)
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", script]);
        piped(c)
    }
}

fn lines_of(events: &[GradleEvent]) -> Vec<OutputLine> {
    events
        .iter()
        .flat_map(|e| match e {
            GradleEvent::Lines { lines } => lines.clone(),
            _ => vec![],
        })
        .collect()
}

#[test]
fn streams_stdout_and_stderr_and_ends_with_success() {
    let (events, sink) = collect();
    let handle = spawn_and_stream(shell("echo uno&& echo dos 1>&2&& echo tres"), sink).unwrap();
    handle.wait();
    let events = events.lock().unwrap();

    let lines = lines_of(&events);
    let texts: Vec<_> = lines.iter().map(|l| l.text.trim()).collect();
    assert!(texts.contains(&"uno"));
    assert!(texts.contains(&"tres"));
    assert!(
        lines
            .iter()
            .any(|l| l.stream == Stream::Stderr && l.text.trim() == "dos")
    );
    // Sin saltos de línea sobrantes.
    assert!(lines.iter().all(|l| !l.text.ends_with('\r')));

    match events.last().unwrap() {
        GradleEvent::Finished {
            outcome, exit_code, ..
        } => {
            assert_eq!(*outcome, Outcome::Success);
            assert_eq!(*exit_code, Some(0));
        }
        other => panic!("el último evento debe ser Finished, fue {other:?}"),
    }
}

#[test]
fn a_failing_command_reports_failed_with_its_exit_code() {
    let (events, sink) = collect();
    spawn_and_stream(shell("exit 3"), sink).unwrap().wait();
    let events = events.lock().unwrap();
    assert!(matches!(
        events.last(),
        Some(GradleEvent::Finished {
            outcome: Outcome::Failed,
            exit_code: Some(3),
            ..
        })
    ));
}

#[test]
fn many_lines_arrive_in_order_and_in_batches() {
    let (events, sink) = collect();
    let script = if cfg!(windows) {
        "for /L %i in (1,1,2000) do @echo linea %i"
    } else {
        "i=1; while [ $i -le 2000 ]; do echo linea $i; i=$((i+1)); done"
    };
    spawn_and_stream(shell(script), sink).unwrap().wait();
    let events = events.lock().unwrap();
    let lines = lines_of(&events);
    assert_eq!(lines.len(), 2000);
    assert_eq!(lines[0].text.trim(), "linea 1");
    assert_eq!(lines[1999].text.trim(), "linea 2000");
    let batches = events
        .iter()
        .filter(|e| matches!(e, GradleEvent::Lines { .. }))
        .count();
    assert!(
        batches >= 4,
        "2000 líneas deben llegar en varias tandas, llegaron en {batches}"
    );
    for e in events.iter() {
        if let GradleEvent::Lines { lines } = e {
            assert!(lines.len() <= MAX_BATCH);
        }
    }
}

#[test]
fn stop_kills_the_process_tree_and_reports_cancelled() {
    let (events, sink) = collect();
    // Un proceso hijo que dormiría 60 s.
    let cmd = if cfg!(windows) {
        let mut c = Command::new("powershell");
        c.args([
            "-NoProfile",
            "-Command",
            "Write-Output listo; Start-Sleep -Seconds 60",
        ]);
        piped(c)
    } else {
        shell("echo listo; sleep 60")
    };
    let started = Instant::now();
    let handle = spawn_and_stream(cmd, sink).unwrap();
    std::thread::sleep(Duration::from_millis(1500));
    assert!(!handle.is_finished());
    handle.stop().unwrap();
    handle.wait();
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "detener debe cortar el proceso"
    );
    let events = events.lock().unwrap();
    assert!(matches!(
        events.last(),
        Some(GradleEvent::Finished {
            outcome: Outcome::Cancelled,
            ..
        })
    ));
}

#[test]
fn gradle_args_match_what_mcreator_passes() {
    let req = GradleRequest {
        root: PathBuf::from("C:/mods/rubymod"),
        tasks: vec!["runClient".into()],
        java_home: Some(PathBuf::from(r"C:\Program Files\Java\jdk-21")),
        max_heap_mb: 2048,
        extra_args: Vec::new(),
    };
    let args = gradle_args(&req);
    assert_eq!(args[0], "runClient");
    assert!(args.contains(&"--console=plain".to_string()));
    assert!(
        args.contains(
            &"-Dorg.gradle.jvmargs=-Xmx2048m -Duser.language=en -Declipse.application=net.mcreator"
                .to_string()
        )
    );
    assert!(args.contains(&"-Porg.gradle.java.installations.auto-detect=false".to_string()));
    assert!(args.contains(
        &"-Porg.gradle.java.installations.paths=C:/Program Files/Java/jdk-21".to_string()
    ));

    let no_java = GradleRequest {
        java_home: None,
        ..req
    };
    assert!(
        !gradle_args(&no_java)
            .iter()
            .any(|a| a.contains("installations"))
    );
}

#[test]
fn recognizes_gradle_projects_and_creates_mcreator_gradle() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!is_gradle_project(dir.path()));
    let req = GradleRequest {
        root: dir.path().to_path_buf(),
        tasks: vec!["build".into()],
        java_home: None,
        max_heap_mb: 1024,
        extra_args: Vec::new(),
    };
    assert!(matches!(
        build_command(&req),
        Err(GradleError::NotAGradleProject(_))
    ));

    std::fs::write(dir.path().join("build.gradle"), "").unwrap();
    std::fs::write(wrapper_script(dir.path()), "").unwrap();
    assert!(is_gradle_project(dir.path()));

    assert!(ensure_mcreator_gradle(dir.path()).unwrap());
    std::fs::write(dir.path().join("mcreator.gradle"), "// deps").unwrap();
    assert!(!ensure_mcreator_gradle(dir.path()).unwrap());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("mcreator.gradle")).unwrap(),
        "// deps"
    );
}

#[test]
fn build_command_cleans_the_environment_and_sets_java_home() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("build.gradle"), "").unwrap();
    std::fs::write(wrapper_script(dir.path()), "").unwrap();
    let req = GradleRequest {
        root: dir.path().to_path_buf(),
        tasks: vec!["build".into()],
        java_home: Some(PathBuf::from("C:/jdk21")),
        max_heap_mb: 1024,
        extra_args: Vec::new(),
    };
    let cmd = build_command(&req).unwrap();
    let envs: Vec<_> = cmd.get_envs().collect();
    for var in CLEARED_ENV.iter().filter(|v| **v != "JAVA_HOME") {
        assert!(
            envs.iter()
                .any(|(k, v)| k.to_str() == Some(*var) && v.is_none()),
            "{var} debe quitarse del entorno"
        );
    }
    assert!(
        envs.iter().any(|(k, v)| k.to_str() == Some("JAVA_HOME")
            && v.map(|v| v.to_str()) == Some(Some("C:/jdk21")))
    );
    assert_eq!(cmd.get_current_dir(), Some(dir.path()));
}

#[test]
fn events_serialize_for_the_ui() {
    let e = GradleEvent::Finished {
        outcome: Outcome::Success,
        exit_code: Some(0),
        duration_ms: 1200,
    };
    assert_eq!(
        serde_json::to_value(&e).unwrap(),
        serde_json::json!({"kind": "finished", "outcome": "success", "exitCode": 0, "durationMs": 1200})
    );
    let l = GradleEvent::Lines {
        lines: vec![OutputLine {
            stream: Stream::Stderr,
            text: "x".into(),
        }],
    };
    assert_eq!(
        serde_json::to_value(&l).unwrap(),
        serde_json::json!({"kind": "lines", "lines": [{"stream": "stderr", "text": "x"}]})
    );
}
