use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
pub struct SegmentEvent {
    pub id:              i64,
    pub session_id:      i64,
    pub speaker_id:      i64,
    pub speaker_label:   String,
    pub display_name:    Option<String>,
    pub start_ms:        i64,
    pub end_ms:          i64,
    pub transcript_text: String,
}

#[derive(serde::Serialize, Clone)]
pub struct SpeakerEvent {
    pub id:              i64,
    pub speech_swift_id: i64,
    pub display_name:    Option<String>,
}

pub fn emit_segment_added(app: &tauri::AppHandle, seg: SegmentEvent) {
    let _ = app.emit("segment_added", seg);
}

pub fn emit_new_speaker(app: &tauri::AppHandle, speaker: SpeakerEvent) {
    let _ = app.emit("new_speaker", speaker);
}
