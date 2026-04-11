use crate::{client, db, state::AppState};

#[tauri::command]
pub fn get_speakers(
    state: tauri::State<AppState>,
) -> Result<Vec<db::speakers::SpeakerWithStats>, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::list_with_stats(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_speaker(
    speech_swift_id: i64,
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::rename_speaker(&state.speech_swift_url, speech_swift_id, &name)
        .await
        .map_err(|e| e.to_string())?;
    let db = state.db.lock().expect("db mutex poisoned");
    db.execute(
        "UPDATE speakers SET display_name = ?1 WHERE speech_swift_id = ?2",
        rusqlite::params![name, speech_swift_id],
    )
    .map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::merge_speaker_local(&db, src_id, dst_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_speaker(
    speech_swift_id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::delete_speaker(&state.speech_swift_url, speech_swift_id)
        .await
        .map_err(|e| e.to_string())?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::delete_speaker_local(&db, speech_swift_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_speaker_sample_path(
    speech_swift_id: i64,
    state: tauri::State<AppState>,
) -> Result<Option<String>, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::get_sample_path(&db, speech_swift_id).map_err(|e| e.to_string())
}
