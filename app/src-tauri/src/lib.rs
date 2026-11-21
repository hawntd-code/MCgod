mod environment;

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
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![environment])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar la aplicación");
}
