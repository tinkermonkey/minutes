use tauri::Emitter;
use crate::{client, db, state::{AppState, SpeechSwiftStatus}};

/// Re-probe the speech-swift audio-server and update cached reachability state.
///
/// Emits `speech_swift_reachable` or `speech_swift_unreachable` so the
/// frontend can update its status indicator without polling.
#[tauri::command]
pub async fn retry_health_check(
    state: tauri::State<'_, AppState>,
    app:   tauri::AppHandle,
) -> Result<bool, String> {
    let reachable = client::speech_swift::health_check(&state.speech_swift_url).await;
    *state.speech_swift.lock().expect("speech_swift mutex poisoned") =
        SpeechSwiftStatus { reachable };
    if reachable {
        let _ = app.emit("speech_swift_reachable", ());
    } else {
        let _ = app.emit("speech_swift_unreachable", ());
    }
    Ok(reachable)
}

/// Persist a custom speech-swift port to settings so it survives restarts.
///
/// The live `speech_swift_url` on `AppState` is NOT updated here — a restart
/// is required for the port change to take effect. This keeps the URL field
/// immutable for the lifetime of the process, which avoids races with in-flight
/// HTTP requests.
#[tauri::command]
pub fn set_speech_swift_port(
    port:  u16,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::settings::set(&db, "speech_swift_port", &port.to_string())
        .map_err(|e| e.to_string())
}
