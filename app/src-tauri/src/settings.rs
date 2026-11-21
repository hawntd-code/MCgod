//! Ajustes de la app que sobreviven entre sesiones: si ya se vio la bienvenida
//! y la lista de proyectos recientes. Se guardan en `settings.json` dentro de la
//! carpeta de configuración de la app.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const MAX_RECENT: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub onboarding_done: bool,
    #[serde(default)]
    pub recent: Vec<RecentWorkspace>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentWorkspace {
    /// Ruta del archivo `.mcreator`.
    pub path: String,
    pub name: String,
    pub modid: String,
    /// Segundos desde 1970 (UTC).
    pub opened_at: u64,
}

impl AppSettings {
    /// Pone el proyecto al principio de la lista, sin duplicados y con un máximo de `MAX_RECENT`.
    pub fn remember(&mut self, entry: RecentWorkspace) {
        self.recent.retain(|r| !same_path(&r.path, &entry.path));
        self.recent.insert(0, entry);
        self.recent.truncate(MAX_RECENT);
    }

    pub fn forget(&mut self, path: &str) {
        self.recent.retain(|r| !same_path(&r.path, path));
    }
}

/// En Windows las rutas no distinguen mayúsculas.
fn same_path(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<AppSettings>,
}

impl SettingsStore {
    /// Carga los ajustes. Si el archivo no existe o está dañado, empieza con los valores por defecto.
    pub fn load(path: PathBuf) -> SettingsStore {
        let current = fs::read_to_string(&path)
            .ok()
            .and_then(|text| match serde_json::from_str(&text) {
                Ok(s) => Some(s),
                Err(e) => {
                    log::warn!("settings.json dañado, se ignora: {e}");
                    None
                }
            })
            .unwrap_or_default();
        SettingsStore {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> AppSettings {
        self.current.lock().expect("ajustes bloqueados").clone()
    }

    /// Cambia los ajustes y los guarda en disco antes de devolverlos.
    pub fn update(&self, change: impl FnOnce(&mut AppSettings)) -> io::Result<AppSettings> {
        let mut guard = self.current.lock().expect("ajustes bloqueados");
        let mut next = guard.clone();
        change(&mut next);
        write_atomic(&self.path, &next)?;
        *guard = next.clone();
        Ok(next)
    }
}

fn write_atomic(path: &Path, settings: &AppSettings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recent(path: &str, at: u64) -> RecentWorkspace {
        RecentWorkspace {
            path: path.into(),
            name: format!("Mod {at}"),
            modid: format!("mod{at}"),
            opened_at: at,
        }
    }

    #[test]
    fn first_run_has_onboarding_pending() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::load(dir.path().join("settings.json"));
        assert!(!store.get().onboarding_done);
        assert!(store.get().recent.is_empty());
    }

    #[test]
    fn onboarding_is_remembered_across_restarts() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config").join("settings.json");
        SettingsStore::load(file.clone())
            .update(|s| s.onboarding_done = true)
            .unwrap();
        assert!(SettingsStore::load(file).get().onboarding_done);
    }

    #[test]
    fn recent_list_moves_reopened_projects_to_the_top_without_duplicates() {
        let mut s = AppSettings::default();
        s.remember(recent("C:/a/a.mcreator", 1));
        s.remember(recent("C:/b/b.mcreator", 2));
        s.remember(recent("C:/a/a.mcreator", 3));
        let paths: Vec<_> = s.recent.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, ["C:/a/a.mcreator", "C:/b/b.mcreator"]);
        assert_eq!(s.recent[0].opened_at, 3);
    }

    #[test]
    fn recent_list_is_capped() {
        let mut s = AppSettings::default();
        for i in 0..(MAX_RECENT as u64 + 3) {
            s.remember(recent(&format!("C:/p{i}.mcreator"), i));
        }
        assert_eq!(s.recent.len(), MAX_RECENT);
        assert_eq!(s.recent[0].path, format!("C:/p{}.mcreator", MAX_RECENT + 2));
    }

    #[test]
    fn forget_removes_a_project() {
        let mut s = AppSettings::default();
        s.remember(recent("C:/a.mcreator", 1));
        s.remember(recent("C:/b.mcreator", 2));
        s.forget("C:/a.mcreator");
        assert_eq!(s.recent.len(), 1);
        assert_eq!(s.recent[0].path, "C:/b.mcreator");
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_ignore_case() {
        let mut s = AppSettings::default();
        s.remember(recent("C:/Mods/A.mcreator", 1));
        s.remember(recent("c:/mods/a.mcreator", 2));
        assert_eq!(s.recent.len(), 1);
    }

    #[test]
    fn a_damaged_file_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, "{ esto no es json").unwrap();
        let store = SettingsStore::load(file.clone());
        assert_eq!(store.get(), AppSettings::default());
        // Y se puede volver a guardar encima.
        store.update(|s| s.onboarding_done = true).unwrap();
        assert!(SettingsStore::load(file).get().onboarding_done);
    }
}
