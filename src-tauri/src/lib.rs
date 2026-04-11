mod audio;
mod client;
mod commands;
mod db;
mod embed;
mod events;
mod state;

use state::{AppState, SpeechSwiftStatus};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

/// Returns whether the speech-swift audio-server was reachable at startup.
///
/// The frontend can call this at any time to get the last-known status.
/// Live status changes are pushed via the `speech_swift_unreachable` event.
#[tauri::command]
fn get_speech_swift_status(state: tauri::State<AppState>) -> bool {
    state
        .speech_swift
        .lock()
        .expect("speech_swift mutex poisoned")
        .reachable
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let conn = db::open(&data_dir.join("minutes.db"))?;

            let base_url = "http://localhost:8080".to_string();
            let app_state = AppState {
                db: Mutex::new(conn),
                speech_swift: Mutex::new(SpeechSwiftStatus { reachable: false }),
                speech_swift_url: base_url.clone(),
                pipelines: Mutex::new(HashMap::new()),
            };
            app.manage(app_state);

            // Probe the audio-server in the background; push an event if it is
            // unreachable so the frontend can show a warning immediately.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let reachable = client::speech_swift::health_check(&base_url).await;
                {
                    let state = handle.state::<AppState>();
                    state
                        .speech_swift
                        .lock()
                        .expect("speech_swift mutex poisoned")
                        .reachable = reachable;
                }
                if !reachable {
                    let _ = handle.emit("speech_swift_unreachable", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_speech_swift_status,
            commands::start_session,
            commands::stop_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
