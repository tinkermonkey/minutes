use crate::{client, db, state::AppState};

/// Log `err` to stderr and return it as a `String` for the Tauri command result.
fn log_err(context: &str, err: impl std::fmt::Display) -> String {
    let msg = err.to_string();
    eprintln!("[{context}] {msg}");
    msg
}

#[tauri::command]
pub fn get_speakers(
    state: tauri::State<AppState>,
) -> Result<Vec<db::speakers::SpeakerWithStats>, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::list_with_stats(&db).map_err(|e| log_err("get_speakers", e))
}

#[tauri::command]
pub async fn rename_speaker(
    speech_swift_id: i64,
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::rename_speaker(&state.speech_swift_url, speech_swift_id, &name)
        .await
        .map_err(|e| log_err("rename_speaker", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db.execute(
        "UPDATE speakers SET display_name = ?1 WHERE speech_swift_id = ?2",
        rusqlite::params![name, speech_swift_id],
    )
    .map_err(|e| log_err("rename_speaker/db", e))?;
    Ok(())
}

#[tauri::command]
pub async fn merge_speakers(
    src_id: i64,
    dst_id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::merge_speakers(&state.speech_swift_url, src_id, dst_id)
        .await
        .map_err(|e| log_err("merge_speakers", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::merge_speaker_local(&db, src_id, dst_id)
        .map_err(|e| log_err("merge_speakers/db", e))
}

#[tauri::command]
pub async fn delete_speaker(
    speech_swift_id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::delete_speaker(&state.speech_swift_url, speech_swift_id)
        .await
        .map_err(|e| log_err("delete_speaker", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::delete_speaker_local(&db, speech_swift_id)
        .map_err(|e| log_err("delete_speaker/db", e))
}

#[tauri::command]
pub async fn reset_speaker_registry(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::reset_registry(&state.speech_swift_url)
        .await
        .map_err(|e| log_err("reset_speaker_registry", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::reset_all(&db)
        .map_err(|e| log_err("reset_speaker_registry/db", e))
}

#[tauri::command]
pub fn get_speaker_sample_path(
    speech_swift_id: i64,
    state: tauri::State<AppState>,
) -> Result<Option<String>, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::get_sample_path(&db, speech_swift_id)
        .map_err(|e| log_err("get_speaker_sample_path", e))
}

/// Read a WAV sample file and return its raw bytes.
///
/// The frontend uses this to construct a Blob URL for the <audio> element,
/// bypassing the need for the Tauri asset protocol.
#[tauri::command]
pub fn read_audio_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("read_audio_bytes: {e}"))
}
