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

pub fn emit_audio_level(app: &tauri::AppHandle, level: f32) {
    let _ = app.emit("audio_level", level);
}

/// Emitted just before a WAV chunk is POSTed to speech-swift.
#[derive(serde::Serialize, Clone)]
pub struct ChunkSentEvent {
    /// Position of this chunk in the recording (ms from session start).
    pub start_ms:   u64,
    pub end_ms:     u64,
    /// Wall-clock time the chunk was dispatched (Unix ms).
    pub sent_at_ms: u64,
}

/// Emitted after speech-swift returns a successful response for a chunk.
#[derive(serde::Serialize, Clone)]
pub struct ChunkProcessedEvent {
    /// Matches the `start_ms` of the corresponding `ChunkSentEvent`.
    pub start_ms:      u64,
    /// Round-trip time for the speech-swift HTTP call (ms).
    pub response_ms:   u64,
    pub word_count:    u32,
    pub speaker_count: u32,
}

pub fn emit_chunk_sent(app: &tauri::AppHandle, ev: ChunkSentEvent) {
    let _ = app.emit("chunk_sent", ev);
}

pub fn emit_chunk_processed(app: &tauri::AppHandle, ev: ChunkProcessedEvent) {
    let _ = app.emit("chunk_processed", ev);
}
