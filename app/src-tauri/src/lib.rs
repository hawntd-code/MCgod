mod commands;
mod environment;
mod settings;

use commands::AppState;
use settings::SettingsStore;
use tauri::Manager;

/// Comando para la interfaz: información del entorno (versión, sistema, Java).
#[tauri::command]
async fn environment() -> Result<environment::Environment, String> {
    let env = tauri::async_runtime::spawn_blocking(environment::detect)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("entorno detectado: {env:?}");
    Ok(env)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let settings_file = app.path().app_config_dir()?.join("settings.json");
            log::info!("ajustes en {}", settings_file.display());
            app.manage(AppState::new(SettingsStore::load(settings_file)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            environment,
            commands::app_settings,
            commands::finish_onboarding,
            commands::forget_recent,
            commands::open_workspace,
            commands::close_workspace,
            commands::read_element,
        ])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar la aplicación");
}
