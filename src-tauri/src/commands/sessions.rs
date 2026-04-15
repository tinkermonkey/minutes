use crate::{db, state::AppState};

#[tauri::command]
pub fn get_sessions(
    filter: db::sessions::SessionFilter,
    state: tauri::State<AppState>,
) -> Result<db::sessions::SessionsPage, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::sessions::list_sessions(&db, &filter).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(
    session_id: i64,
    state: tauri::State<AppState>,
) -> Result<Option<db::sessions::SessionRow>, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::sessions::get_session_by_id(&db, session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_all_sessions(
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::sessions::delete_all(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_segments(
    session_id: i64,
    state: tauri::State<AppState>,
) -> Result<Vec<db::segments::SegmentWithSpeaker>, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::segments::get_segments_with_speakers(&db, session_id).map_err(|e| e.to_string())
}
