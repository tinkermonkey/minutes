use rusqlite::OptionalExtension;
use crate::{client, db, events, state::AppState};

/// A speaker paired with how similar they are to the queried speaker.
#[derive(Debug, serde::Serialize)]
pub struct SimilarSpeaker {
    pub speaker:          db::speakers::SpeakerWithStats,
    pub similarity_score: f32,
}

/// Detail view for a single speaker: recent sessions and confirmed segments.
#[derive(Debug, serde::Serialize)]
pub struct SpeakerDetail {
    pub recent_sessions: Vec<db::speakers::SpeakerSession>,
    pub recent_segments: Vec<db::speakers::SpeakerSegment>,
}

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
    app: tauri::AppHandle,
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
    drop(db);
    events::emit_speaker_renamed(&app, events::SpeakerRenamedEvent {
        speech_swift_id,
        display_name: name,
    });
    Ok(())
}

#[tauri::command]
pub async fn merge_speakers(
    app: tauri::AppHandle,
    src_id: i64,
    dst_id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::merge_speakers(&state.speech_swift_url, src_id, dst_id)
        .await
        .map_err(|e| log_err("merge_speakers", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::merge_speaker_local(&db, src_id, dst_id)
        .map_err(|e| log_err("merge_speakers/db", e))?;
    let dst_display_name: Option<String> = db
        .query_row(
            "SELECT display_name FROM speakers WHERE speech_swift_id = ?1",
            rusqlite::params![dst_id],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()
        .ok()
        .flatten()
        .flatten();
    drop(db);
    events::emit_speakers_merged(&app, events::SpeakersMergedEvent {
        src_id,
        dst_id,
        dst_display_name,
    });
    Ok(())
}

#[tauri::command]
pub async fn delete_speaker(
    app: tauri::AppHandle,
    speech_swift_id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::delete_speaker(&state.speech_swift_url, speech_swift_id)
        .await
        .map_err(|e| log_err("delete_speaker", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::delete_speaker_local(&db, speech_swift_id)
        .map_err(|e| log_err("delete_speaker/db", e))?;
    drop(db);
    events::emit_speaker_deleted(&app, events::SpeakerDeletedEvent { speech_swift_id });
    Ok(())
}

#[tauri::command]
pub async fn reset_speaker_registry(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    client::speech_swift::reset_registry(&state.speech_swift_url)
        .await
        .map_err(|e| log_err("reset_speaker_registry", e))?;
    let db = state.db.lock().expect("db mutex poisoned");
    db::speakers::reset_all(&db)
        .map_err(|e| log_err("reset_speaker_registry/db", e))?;
    drop(db);
    events::emit_speaker_registry_reset(&app);
    Ok(())
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

/// Return speakers that are acoustically similar to `speech_swift_id`.
///
/// Calls `GET /registry/speakers/{id}/similar` on the audio-server and
/// cross-references each returned id against the local DB. Speakers not found
/// locally (e.g. not yet upserted) are silently omitted. Returns an empty Vec
/// when the audio-server responds with 404.
#[tauri::command]
pub async fn get_similar_speakers(
    speech_swift_id: i64,
    limit:           Option<i32>,
    state:           tauri::State<'_, AppState>,
) -> Result<Vec<SimilarSpeaker>, String> {
    let limit = limit.unwrap_or(10) as i64;
    let records = client::speech_swift::get_similar_speakers(
        &state.speech_swift_url,
        speech_swift_id,
        limit,
    )
    .await
    .map_err(|e| log_err("get_similar_speakers", e))?;

    // Load all local speakers once and look up by speech_swift_id.
    let db = state.db.lock().expect("db mutex poisoned");
    let all_local = db::speakers::list_with_stats(&db)
        .map_err(|e| log_err("get_similar_speakers/db", e))?;
    drop(db);

    let result = records
        .into_iter()
        .filter_map(|rec| {
            all_local
                .iter()
                .find(|s| s.speech_swift_id == rec.id)
                .map(|s| SimilarSpeaker {
                    speaker:          s.clone(),
                    similarity_score: rec.similarity,
                })
        })
        .collect();

    Ok(result)
}

/// Return recent sessions and confirmed segments for `speech_swift_id`.
#[tauri::command]
pub fn get_speaker_detail(
    speech_swift_id: i64,
    state:           tauri::State<'_, AppState>,
) -> Result<SpeakerDetail, String> {
    let db = state.db.lock().expect("db mutex poisoned");
    let recent_sessions = db::speakers::recent_sessions_for_speaker(&db, speech_swift_id, 10)
        .map_err(|e| log_err("get_speaker_detail/sessions", e))?;
    let recent_segments = db::speakers::recent_segments_for_speaker(&db, speech_swift_id, 20)
        .map_err(|e| log_err("get_speaker_detail/segments", e))?;
    Ok(SpeakerDetail { recent_sessions, recent_segments })
}
