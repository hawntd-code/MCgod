//! Ejecución de Gradle sobre un workspace de MCreator.
//!
//! Reproduce lo que hace `net.mcreator.gradle.GradleUtils`:
//! - Java fijo (`JAVA_HOME` y `org.gradle.java.installations.paths`), sin autodetección.
//! - Entorno limpio: sin variables de Java o Gradle del sistema que cambien el resultado.
//! - Gradle en inglés (`-Duser.language=en`) para poder leer sus errores.
//! - `-Declipse.application=net.mcreator`: ModDevGradle se comporta como dentro de un IDE.
//!
//! La salida se entrega en tandas para no saturar la interfaz: Minecraft en modo
//! desarrollo escribe miles de líneas por segundo.

pub mod java;

use serde::Serialize;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Variables que MCreator quita del entorno antes de llamar a Gradle.
pub const CLEARED_ENV: &[&str] = &[
    "JAVA_HOME",
    "JRE_HOME",
    "JDK_HOME",
    "CLASSPATH",
    "_JAVA_OPTIONS",
    "JAVA_OPTS",
    "JAVA_TOOL_OPTIONS",
    "JAVACMD",
    "JDK_JAVA_OPTIONS",
    "GRADLE_HOME",
    "GRADLE_OPTS",
    "GRADLE_USER_HOME",
    // https://github.com/neoforged/ModDevGradle/pull/320
    "CI",
];

/// Cada cuánto se entrega una tanda de líneas, como máximo.
pub const FLUSH_EVERY: Duration = Duration::from_millis(100);
/// Tamaño máximo de una tanda.
pub const MAX_BATCH: usize = 500;

#[derive(Debug, thiserror::Error)]
pub enum GradleError {
    #[error("{0} no es un proyecto de Gradle (falta build.gradle o gradlew)")]
    NotAGradleProject(PathBuf),
    #[error("no se pudo iniciar Gradle: {0}")]
    Spawn(#[source] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct GradleRequest {
    pub root: PathBuf,
    pub tasks: Vec<String>,
    pub java_home: Option<PathBuf>,
    /// Memoria máxima del proceso de Gradle, en MB.
    pub max_heap_mb: u32,
    /// Argumentos adicionales después de las tareas (`--init-script`, `-P…`).
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutputLine {
    pub stream: Stream,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GradleEvent {
    Lines {
        lines: Vec<OutputLine>,
    },
    #[serde(rename_all = "camelCase")]
    Finished {
        outcome: Outcome,
        exit_code: Option<i32>,
        duration_ms: u64,
    },
}

/// Script de Gradle del proyecto según el sistema.
pub fn wrapper_script(root: &Path) -> PathBuf {
    if cfg!(windows) {
        root.join("gradlew.bat")
    } else {
        root.join("gradlew")
    }
}

pub fn is_gradle_project(root: &Path) -> bool {
    (root.join("build.gradle").is_file() || root.join("build.gradle.kts").is_file())
        && wrapper_script(root).is_file()
}

/// MCreator siempre escribe `mcreator.gradle` (dependencias de APIs de mods) y `build.gradle`
/// lo incluye. Si falta, Gradle falla; se crea vacío sin tocar uno existente.
pub fn ensure_mcreator_gradle(root: &Path) -> std::io::Result<bool> {
    let file = root.join("mcreator.gradle");
    if file.exists() {
        return Ok(false);
    }
    std::fs::write(file, "")?;
    Ok(true)
}

/// Argumentos para Gradle, sin el programa.
pub fn gradle_args(req: &GradleRequest) -> Vec<String> {
    let mut args: Vec<String> = req.tasks.clone();
    args.push("--console=plain".into());
    args.push(format!(
        "-Dorg.gradle.jvmargs=-Xmx{}m -Duser.language=en -Declipse.application=net.mcreator",
        req.max_heap_mb
    ));
    if let Some(home) = &req.java_home {
        args.push("-Porg.gradle.java.installations.auto-detect=false".into());
        args.push(format!(
            "-Porg.gradle.java.installations.paths={}",
            home.display().to_string().replace('\\', "/")
        ));
    }
    args.extend(req.extra_args.iter().cloned());
    args
}

/// Script de inicio de Gradle que agrega la tarea `mcgodClasspath`: escribe el classpath
/// de compilación del proyecto (un JAR por línea) en el archivo de la propiedad
/// `mcgod.classpathOut`. MCgod lo usa para saber qué clases puede importar el código
/// generado, como MCreator con la API de Gradle.
pub const CLASSPATH_INIT_SCRIPT: &str = r#"// Generado por MCgod: escribe el classpath de compilación para el índice de imports.
allprojects {
    plugins.withId('java') {
        def outPath = providers.gradleProperty('mcgod.classpathOut')
        if (outPath.isPresent()) {
            def out = file(outPath.get())
            def cp = sourceSets.main.compileClasspath
            tasks.register('mcgodClasspath') {
                inputs.files(cp)
                doLast {
                    out.parentFile.mkdirs()
                    out.text = cp.files.collect { it.absolutePath }.join('\n')
                }
            }
        }
    }
}
"#;

/// Petición para escribir el classpath del proyecto en `out` con `CLASSPATH_INIT_SCRIPT`
/// (que debe estar guardado en `init_script`).
pub fn classpath_request(
    root: &Path,
    init_script: &Path,
    out: &Path,
    java_home: Option<PathBuf>,
    max_heap_mb: u32,
) -> GradleRequest {
    // Gradle corre en la carpeta del proyecto: las rutas tienen que ser absolutas.
    let abs = |p: &Path| std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    let (init_script, out) = (abs(init_script), abs(out));
    GradleRequest {
        root: root.to_path_buf(),
        tasks: vec!["mcgodClasspath".into()],
        java_home,
        max_heap_mb,
        extra_args: vec![
            "--init-script".into(),
            init_script.display().to_string(),
            format!(
                "-Pmcgod.classpathOut={}",
                out.display().to_string().replace('\\', "/")
            ),
            // La tarea se agrega desde fuera del proyecto: sin caché de configuración.
            "--no-configuration-cache".into(),
        ],
    }
}

pub fn build_command(req: &GradleRequest) -> Result<Command, GradleError> {
    if !is_gradle_project(&req.root) {
        return Err(GradleError::NotAGradleProject(req.root.clone()));
    }
    let mut cmd = Command::new(wrapper_script(&req.root));
    cmd.args(gradle_args(req))
        .current_dir(&req.root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for var in CLEARED_ENV {
        cmd.env_remove(var);
    }
    if let Some(home) = &req.java_home {
        cmd.env("JAVA_HOME", home);
    }
    hide_console_and_group(&mut cmd);
    Ok(cmd)
}

#[cfg(windows)]
fn hide_console_and_group(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(unix)]
fn hide_console_and_group(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    // Grupo propio para poder detener Gradle y todo lo que abra.
    cmd.process_group(0);
}

/// Detiene un proceso y todos sus hijos.
pub fn kill_tree(pid: u32) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
    }
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-TERM", &format!("-{pid}")])
            .status()
            .map(|_| ())
    }
}

/// Gradle en marcha.
pub struct RunHandle {
    pid: u32,
    cancelled: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl RunHandle {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    /// Detiene Gradle y lo que haya abierto (por ejemplo, Minecraft).
    pub fn stop(&self) -> std::io::Result<()> {
        if self.is_finished() {
            return Ok(());
        }
        self.cancelled.store(true, Ordering::SeqCst);
        kill_tree(self.pid)
    }

    /// Espera a que termine y a que se entregue el evento final.
    pub fn wait(mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Inicia Gradle y entrega su salida a `sink`. El último evento siempre es `Finished`.
pub fn start(
    req: &GradleRequest,
    sink: impl Fn(GradleEvent) + Send + 'static,
) -> Result<RunHandle, GradleError> {
    let cmd = build_command(req)?;
    spawn_and_stream(cmd, sink)
}

/// Igual que [`start`], con un comando cualquiera. Sirve para probar el transporte.
pub fn spawn_and_stream(
    mut cmd: Command,
    sink: impl Fn(GradleEvent) + Send + 'static,
) -> Result<RunHandle, GradleError> {
    let started = Instant::now();
    let mut child = cmd.spawn().map_err(GradleError::Spawn)?;
    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (tx, rx) = mpsc::channel::<OutputLine>();
    let mut readers = Vec::new();
    if let Some(out) = stdout {
        readers.push(read_lines(out, Stream::Stdout, tx.clone()));
    }
    if let Some(err) = stderr {
        readers.push(read_lines(err, Stream::Stderr, tx.clone()));
    }
    drop(tx);

    let cancelled = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));
    let (cancelled_w, finished_w) = (cancelled.clone(), finished.clone());

    let worker = std::thread::spawn(move || {
        let mut batch: Vec<OutputLine> = Vec::new();
        let mut last_flush = Instant::now();
        loop {
            match rx.recv_timeout(FLUSH_EVERY) {
                Ok(line) => {
                    batch.push(line);
                    if batch.len() >= MAX_BATCH || last_flush.elapsed() >= FLUSH_EVERY {
                        sink(GradleEvent::Lines {
                            lines: std::mem::take(&mut batch),
                        });
                        last_flush = Instant::now();
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !batch.is_empty() {
                        sink(GradleEvent::Lines {
                            lines: std::mem::take(&mut batch),
                        });
                    }
                    last_flush = Instant::now();
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        if !batch.is_empty() {
            sink(GradleEvent::Lines { lines: batch });
        }
        for r in readers {
            let _ = r.join();
        }
        let status = child.wait().ok();
        let exit_code = status.and_then(|s| s.code());
        let outcome = if cancelled_w.load(Ordering::SeqCst) {
            Outcome::Cancelled
        } else if status.is_some_and(|s| s.success()) {
            Outcome::Success
        } else {
            Outcome::Failed
        };
        finished_w.store(true, Ordering::SeqCst);
        sink(GradleEvent::Finished {
            outcome,
            exit_code,
            duration_ms: started.elapsed().as_millis() as u64,
        });
    });

    Ok(RunHandle {
        pid,
        cancelled,
        finished,
        worker: Some(worker),
    })
}

fn read_lines(
    source: impl Read + Send + 'static,
    stream: Stream,
    tx: mpsc::Sender<OutputLine>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(source);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    // Java en Windows no siempre escribe UTF-8: se decodifica sin fallar.
                    let text = String::from_utf8_lossy(&buf)
                        .trim_end_matches(['\r', '\n'])
                        .to_string();
                    if tx.send(OutputLine { stream, text }).is_err() {
                        break;
                    }
                }
            }
        }
    })
}
