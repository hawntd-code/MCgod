//! Comandos que la interfaz llama con `invoke`.
//!
//! Cada comando delega en una función normal que recibe `&AppState`, para poder
//! probarla sin levantar la ventana.

use crate::settings::{AppSettings, RecentWorkspace, SettingsStore};
use mcgod_gradle::java::JavaInfo;
use mcgod_gradle::{GradleEvent, GradleRequest, RunHandle};
use mcgod_workspace::model::ElementDefinition;
use mcgod_workspace::{Workspace, WorkspaceError, WorkspaceSummary};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::ipc::Channel;
use tauri::{Manager, State};

/// Java mínimo para los generadores de NeoForge de MCreator.
pub const REQUIRED_JAVA: u32 = 21;
/// Memoria para el proceso de Gradle; MCreator usa un valor parecido por defecto.
pub const GRADLE_HEAP_MB: u32 = 2048;

pub struct AppState {
    pub settings: SettingsStore,
    pub workspace: Mutex<Option<Workspace>>,
    pub gradle: Mutex<Option<RunHandle>>,
}

impl AppState {
    pub fn new(settings: SettingsStore) -> AppState {
        AppState {
            settings,
            workspace: Mutex::new(None),
            gradle: Mutex::new(None),
        }
    }

    /// Detiene la tarea de Gradle en curso, si la hay.
    pub fn stop_gradle(&self) {
        if let Some(run) = self.gradle.lock().expect("gradle bloqueado").as_ref()
            && let Err(e) = run.stop()
        {
            log::warn!("no se pudo detener Gradle: {e}");
        }
    }
}

/// Tareas de Gradle que la interfaz puede pedir. Los nombres son los de MCreator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GradleTask {
    Build,
    RunClient,
}

impl GradleTask {
    pub fn gradle_name(self) -> &'static str {
        match self {
            GradleTask::Build => "build",
            GradleTask::RunClient => "runClient",
        }
    }
}

/// Error con un código estable: la interfaz lo traduce al idioma de la persona.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
    pub detail: String,
}

impl CommandError {
    fn new(code: &'static str, detail: impl ToString) -> Self {
        CommandError {
            code,
            detail: detail.to_string(),
        }
    }
}

impl From<WorkspaceError> for CommandError {
    fn from(e: WorkspaceError) -> Self {
        let code = match &e {
            WorkspaceError::NotFound(_) => "notFound",
            WorkspaceError::NoWorkspaceFile(_) => "noWorkspaceFile",
            WorkspaceError::Ambiguous(_) => "ambiguous",
            WorkspaceError::Io { .. } => "io",
            WorkspaceError::Json { .. } => "invalidJson",
            WorkspaceError::UnknownElement(_) => "unknownElement",
            WorkspaceError::NoDefinition(_) => "noDefinition",
        };
        CommandError::new(code, e)
    }
}

type CmdResult<T> = Result<T, CommandError>;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// --- Lógica -------------------------------------------------------------------

pub fn finish_onboarding_in(state: &AppState) -> CmdResult<AppSettings> {
    state
        .settings
        .update(|s| s.onboarding_done = true)
        .map_err(|e| CommandError::new("settingsWrite", e))
}

pub fn forget_recent_in(state: &AppState, path: &str) -> CmdResult<AppSettings> {
    state
        .settings
        .update(|s| s.forget(path))
        .map_err(|e| CommandError::new("settingsWrite", e))
}

pub fn open_workspace_in(state: &AppState, path: &str) -> CmdResult<WorkspaceSummary> {
    let workspace = Workspace::open(path)?;
    // Una tarea de Gradle del proyecto anterior no debe seguir corriendo.
    state.stop_gradle();
    let summary = workspace.summary();
    log::info!(
        "proyecto abierto: {} ({} elementos)",
        summary.modid,
        summary.elements.len()
    );

    let entry = RecentWorkspace {
        path: summary.file.clone(),
        name: summary.name.clone(),
        modid: summary.modid.clone(),
        opened_at: now(),
    };
    if let Err(e) = state.settings.update(|s| s.remember(entry)) {
        // No poder guardar la lista de recientes no impide abrir el proyecto.
        log::warn!("no se pudo guardar la lista de recientes: {e}");
    }
    *state.workspace.lock().expect("workspace bloqueado") = Some(workspace);
    Ok(summary)
}

pub fn read_element_in(state: &AppState, name: &str) -> CmdResult<ElementDefinition> {
    let guard = state.workspace.lock().expect("workspace bloqueado");
    let workspace = guard
        .as_ref()
        .ok_or_else(|| CommandError::new("noWorkspace", "no hay un proyecto abierto"))?;
    Ok(workspace.read_element(name)?)
}

pub fn close_workspace_in(state: &AppState) {
    state.stop_gradle();
    *state.workspace.lock().expect("workspace bloqueado") = None;
}

/// Revisa todo lo necesario antes de lanzar Gradle y arma la petición.
pub fn gradle_request_in(
    state: &AppState,
    task: GradleTask,
    java: Option<JavaInfo>,
) -> CmdResult<GradleRequest> {
    let root = state
        .workspace
        .lock()
        .expect("workspace bloqueado")
        .as_ref()
        .map(|w| w.root.clone())
        .ok_or_else(|| CommandError::new("noWorkspace", "no hay un proyecto abierto"))?;
    if !mcgod_gradle::is_gradle_project(&root) {
        return Err(CommandError::new(
            "notGradleProject",
            root.display().to_string(),
        ));
    }
    let java = java.ok_or_else(|| CommandError::new("javaMissing", "java no está en el PATH"))?;
    if java.major.is_none_or(|m| m < REQUIRED_JAVA) {
        return Err(CommandError::new("javaOld", &java.version));
    }
    if state
        .gradle
        .lock()
        .expect("gradle bloqueado")
        .as_ref()
        .is_some_and(|run| !run.is_finished())
    {
        return Err(CommandError::new("gradleBusy", task.gradle_name()));
    }
    mcgod_gradle::ensure_mcreator_gradle(&root)
        .map_err(|e| CommandError::new("io", format!("mcreator.gradle: {e}")))?;
    Ok(GradleRequest {
        root,
        tasks: vec![task.gradle_name().to_string()],
        java_home: java.home_path(),
        max_heap_mb: GRADLE_HEAP_MB,
    })
}

// --- Comandos de Tauri --------------------------------------------------------

#[tauri::command]
pub fn app_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.get()
}

#[tauri::command]
pub fn finish_onboarding(state: State<'_, AppState>) -> CmdResult<AppSettings> {
    finish_onboarding_in(&state)
}

#[tauri::command]
pub fn forget_recent(path: String, state: State<'_, AppState>) -> CmdResult<AppSettings> {
    forget_recent_in(&state, &path)
}

/// Lee el disco, así que corre en un hilo aparte para no congelar la ventana.
#[tauri::command]
pub async fn open_workspace(path: String, app: tauri::AppHandle) -> CmdResult<WorkspaceSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        open_workspace_in(&state, &path)
    })
    .await
    .map_err(|e| CommandError::new("internal", e))?
}

#[tauri::command]
pub fn close_workspace(state: State<'_, AppState>) {
    close_workspace_in(&state)
}

#[tauri::command]
pub fn read_element(name: String, state: State<'_, AppState>) -> CmdResult<ElementDefinition> {
    read_element_in(&state, &name)
}

/// Compila (`build`) o abre Minecraft (`runClient`). La salida llega por `on_event`;
/// el comando responde en cuanto Gradle arranca.
#[tauri::command]
pub async fn gradle_run(
    task: GradleTask,
    on_event: Channel<GradleEvent>,
    app: tauri::AppHandle,
) -> CmdResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let req = gradle_request_in(&state, task, mcgod_gradle::java::detect())?;
        log::info!("gradle {} en {}", task.gradle_name(), req.root.display());
        let app_for_sink = app.clone();
        let handle = mcgod_gradle::start(&req, move |event| {
            let finished = matches!(event, GradleEvent::Finished { .. });
            if let GradleEvent::Finished { outcome, .. } = &event {
                log::info!("gradle terminó: {outcome:?}");
            }
            let _ = on_event.send(event);
            if finished {
                *app_for_sink
                    .state::<AppState>()
                    .gradle
                    .lock()
                    .expect("gradle bloqueado") = None;
            }
        })
        .map_err(|e| CommandError::new("gradleStart", e))?;
        *state.gradle.lock().expect("gradle bloqueado") = Some(handle);
        Ok(())
    })
    .await
    .map_err(|e| CommandError::new("internal", e))?
}

#[tauri::command]
pub fn gradle_stop(state: State<'_, AppState>) {
    state.stop_gradle();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/mcgod-workspace/tests/fixtures/rubymod/rubymod.mcreator")
    }

    fn state(dir: &tempfile::TempDir) -> AppState {
        AppState::new(SettingsStore::load(dir.path().join("settings.json")))
    }

    #[test]
    fn opening_a_project_returns_its_summary_and_remembers_it() {
        let dir = tempfile::tempdir().unwrap();
        let s = state(&dir);
        let summary = open_workspace_in(&s, fixture().to_str().unwrap()).unwrap();
        assert_eq!(summary.modid, "rubymod");
        assert_eq!(summary.elements.len(), 7);

        let recent = s.settings.get().recent;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].modid, "rubymod");
        assert!(recent[0].opened_at > 0);
        // Se guardó en disco.
        let reloaded = SettingsStore::load(dir.path().join("settings.json")).get();
        assert_eq!(reloaded.recent, recent);
    }

    #[test]
    fn reading_elements_needs_an_open_project() {
        let dir = tempfile::tempdir().unwrap();
        let s = state(&dir);
        assert_eq!(read_element_in(&s, "Ruby").unwrap_err().code, "noWorkspace");

        open_workspace_in(&s, fixture().to_str().unwrap()).unwrap();
        assert_eq!(read_element_in(&s, "Ruby").unwrap().element_type, "item");
        assert_eq!(
            read_element_in(&s, "RubyPickaxe").unwrap_err().code,
            "noDefinition"
        );
        assert_eq!(
            read_element_in(&s, "Nope").unwrap_err().code,
            "unknownElement"
        );

        close_workspace_in(&s);
        assert_eq!(read_element_in(&s, "Ruby").unwrap_err().code, "noWorkspace");
    }

    #[test]
    fn errors_carry_a_stable_code_and_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let s = state(&dir);
        let missing = dir.path().join("no-existe.mcreator");
        let err = open_workspace_in(&s, missing.to_str().unwrap()).unwrap_err();
        assert_eq!(err.code, "notFound");
        assert!(err.detail.contains("no-existe.mcreator"));

        let bad = dir.path().join("roto.mcreator");
        std::fs::write(&bad, "{").unwrap();
        assert_eq!(
            open_workspace_in(&s, bad.to_str().unwrap())
                .unwrap_err()
                .code,
            "invalidJson"
        );
        // Un intento fallido no agrega nada a recientes.
        assert!(s.settings.get().recent.is_empty());
    }

    #[test]
    fn onboarding_and_recents_go_through_the_store() {
        let dir = tempfile::tempdir().unwrap();
        let s = state(&dir);
        assert!(finish_onboarding_in(&s).unwrap().onboarding_done);
        let path = fixture();
        let summary = open_workspace_in(&s, path.to_str().unwrap()).unwrap();
        let after = forget_recent_in(&s, &summary.file).unwrap();
        assert!(after.recent.is_empty());
        assert!(after.onboarding_done);
    }

    fn example_project() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rubymod/rubymod.mcreator")
    }

    fn java(major: u32) -> Option<JavaInfo> {
        Some(JavaInfo {
            version: format!("{major}.0.1"),
            major: Some(major),
            vendor: None,
            home: Some("C:/jdk".into()),
        })
    }

    #[test]
    fn gradle_needs_an_open_gradle_project_and_java_21() {
        let dir = tempfile::tempdir().unwrap();
        let s = state(&dir);
        let code = |r: CmdResult<GradleRequest>| r.unwrap_err().code;

        assert_eq!(
            code(gradle_request_in(&s, GradleTask::Build, java(21))),
            "noWorkspace"
        );

        // El workspace de prueba del lector no trae archivos de Gradle.
        open_workspace_in(&s, fixture().to_str().unwrap()).unwrap();
        assert_eq!(
            code(gradle_request_in(&s, GradleTask::Build, java(21))),
            "notGradleProject"
        );

        open_workspace_in(&s, example_project().to_str().unwrap()).unwrap();
        assert_eq!(
            code(gradle_request_in(&s, GradleTask::Build, None)),
            "javaMissing"
        );
        assert_eq!(
            code(gradle_request_in(&s, GradleTask::Build, java(17))),
            "javaOld"
        );

        let req = gradle_request_in(&s, GradleTask::RunClient, java(21)).unwrap();
        assert_eq!(req.tasks, ["runClient"]);
        assert_eq!(req.java_home, Some(PathBuf::from("C:/jdk")));
        assert_eq!(req.max_heap_mb, GRADLE_HEAP_MB);
        assert!(req.root.join("mcreator.gradle").is_file());
    }

    #[test]
    fn only_one_gradle_task_at_a_time_and_closing_stops_it() {
        let dir = tempfile::tempdir().unwrap();
        let s = state(&dir);
        open_workspace_in(&s, example_project().to_str().unwrap()).unwrap();

        // Un proceso largo en lugar de Gradle.
        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 60"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let run = mcgod_gradle::spawn_and_stream(cmd, |_| {}).unwrap();
        *s.gradle.lock().unwrap() = Some(run);

        assert_eq!(
            gradle_request_in(&s, GradleTask::Build, java(21))
                .unwrap_err()
                .code,
            "gradleBusy"
        );

        close_workspace_in(&s);
        let run = s.gradle.lock().unwrap().take().unwrap();
        run.wait();
    }

    #[test]
    fn gradle_task_names_match_mcreator() {
        let parse = |s: &str| serde_json::from_value::<GradleTask>(serde_json::json!(s)).unwrap();
        assert_eq!(parse("build").gradle_name(), "build");
        assert_eq!(parse("runClient").gradle_name(), "runClient");
    }

    #[test]
    fn command_errors_serialize_for_the_ui() {
        let e = CommandError::new("notFound", "C:/x");
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({"code": "notFound", "detail": "C:/x"})
        );
    }
}
