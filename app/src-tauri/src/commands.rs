//! Comandos que la interfaz llama con `invoke`.
//!
//! Cada comando delega en una función normal que recibe `&AppState`, para poder
//! probarla sin levantar la ventana.

use crate::settings::{AppSettings, RecentWorkspace, SettingsStore};
use mcgod_workspace::model::ElementDefinition;
use mcgod_workspace::{Workspace, WorkspaceError, WorkspaceSummary};
use serde::Serialize;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, State};

pub struct AppState {
    pub settings: SettingsStore,
    pub workspace: Mutex<Option<Workspace>>,
}

impl AppState {
    pub fn new(settings: SettingsStore) -> AppState {
        AppState {
            settings,
            workspace: Mutex::new(None),
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
    *state.workspace.lock().expect("workspace bloqueado") = None;
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

    #[test]
    fn command_errors_serialize_for_the_ui() {
        let e = CommandError::new("notFound", "C:/x");
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({"code": "notFound", "detail": "C:/x"})
        );
    }
}
