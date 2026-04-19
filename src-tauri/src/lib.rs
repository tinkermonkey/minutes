mod api;
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

fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time must be after Unix epoch")
        .as_millis() as i64
}

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
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let db_path = data_dir.join("minutes.db");
            let conn = db::open(&db_path)?;

            // Load persisted settings before constructing AppState.
            let preferred_device =
                db::settings::get(&conn, "preferred_audio_device").unwrap_or(None);

            let port = db::settings::get(&conn, "speech_swift_port")
                .unwrap_or(None)
                .unwrap_or_else(|| "8080".to_string());
            let base_url = format!("http://localhost:{port}");
            let app_state = AppState {
                db: Mutex::new(conn),
                speech_swift: Mutex::new(SpeechSwiftStatus { reachable: false }),
                speech_swift_url: base_url.clone(),
                pipelines: Mutex::new(HashMap::new()),
                preferred_device: Mutex::new(preferred_device),
            };
            app.manage(app_state);

            // Probe the audio-server in the background; push an event if it is
            // unreachable so the frontend can show a warning immediately.
            // On success, sync the full speaker registry so the local DB stays
            // consistent with speech-swift's state.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Poll until speech-swift is reachable, emitting an unreachable
                // event on the first failure and a reachable event when it comes up.
                let mut previously_reachable = true;
                loop {
                    let reachable = client::speech_swift::health_check(&base_url).await;
                    {
                        let state = handle.state::<AppState>();
                        state
                            .speech_swift
                            .lock()
                            .expect("speech_swift mutex poisoned")
                            .reachable = reachable;
                    }
                    if reachable {
                        if !previously_reachable {
                            let _ = handle.emit("speech_swift_reachable", ());
                        }
                        break;
                    }
                    if previously_reachable {
                        let _ = handle.emit("speech_swift_unreachable", ());
                        previously_reachable = false;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }

                // Full registry sync: upsert every known speaker and propagate
                // display names from speech-swift into the local DB.
                match client::speech_swift::list_speakers(&base_url).await {
                    Err(e) => eprintln!("startup registry sync failed: {e}"),
                    Ok(records) => {
                        let state = handle.state::<AppState>();
                        let now_ms = unix_ms();
                        let db = state.db.lock().expect("db mutex poisoned");
                        for record in records {
                            match db::speakers::upsert_speaker(&db, record.id, now_ms) {
                                Err(e) => {
                                    eprintln!("registry sync error for speaker {}: {e}", record.id)
                                }
                                Ok((speaker, _is_new)) => {
                                    // Sync display_name from speech-swift when present.
                                    if record.display_name.is_some() {
                                        let _ = db.execute(
                                            "UPDATE speakers SET display_name = ?1 \
                                             WHERE speech_swift_id = ?2",
                                            rusqlite::params![record.display_name, record.id],
                                        );
                                    }
                                    // Emit new_speaker for speakers still lacking a name so
                                    // the frontend can prompt the user to label them.
                                    if speaker.display_name.is_none() {
                                        let _ = handle.emit(
                                            "new_speaker",
                                            events::SpeakerEvent {
                                                id:              speaker.id,
                                                speech_swift_id: speaker.speech_swift_id,
                                                display_name:    speaker.display_name,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // Spawn the read-only REST API on 127.0.0.1:8765.
            let api_db_path = db_path.clone();
            tauri::async_runtime::spawn(async move {
                api::serve(api_db_path).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_speech_swift_status,
            commands::start_session,
            commands::stop_session,
            commands::devices::get_audio_devices,
            commands::devices::get_preferred_device,
            commands::devices::set_audio_device,
            commands::health::retry_health_check,
            commands::health::get_speech_swift_port,
            commands::health::set_speech_swift_port,
            commands::speakers::get_speakers,
            commands::speakers::rename_speaker,
            commands::speakers::merge_speakers,
            commands::speakers::delete_speaker,
            commands::speakers::reset_speaker_registry,
            commands::speakers::get_speaker_sample_path,
            commands::speakers::read_audio_bytes,
            commands::speakers::get_similar_speakers,
            commands::speakers::get_speaker_detail,
            commands::sessions::get_sessions,
            commands::sessions::get_session,
            commands::sessions::get_segments,
            commands::sessions::delete_all_sessions,
            commands::search::search_segments,
            commands::settings::get_vad_mode,
            commands::settings::set_vad_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
