use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
pub struct SegmentEvent {
    pub id:              i64,
    pub session_id:      i64,
    pub speaker_id:      Option<i64>,
    pub speaker_label:   Option<String>,
    pub display_name:    Option<String>,
    pub status:          String,
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

/// Emitted on every VAD state transition (speech→silence or silence→speech).
/// Payload is `true` when speech is detected, `false` when silence.
pub fn emit_vad_state(app: &tauri::AppHandle, active: bool) {
    let _ = app.emit("vad_state", active);
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
    /// Minimum best_score across all segments in this chunk (worst match).
    /// None when speech-swift did not return scores.
    pub best_score:    Option<f32>,
}

pub fn emit_chunk_sent(app: &tauri::AppHandle, ev: ChunkSentEvent) {
    let _ = app.emit("chunk_sent", ev);
}

pub fn emit_chunk_processed(app: &tauri::AppHandle, ev: ChunkProcessedEvent) {
    let _ = app.emit("chunk_processed", ev);
}

/// Emitted when the slow path replaces fast-path segments with its own results.
/// The frontend should remove segments with `removed_ids` and insert `added`.
#[derive(serde::Serialize, Clone)]
pub struct SegmentsReplacedEvent {
    pub removed_ids: Vec<i64>,
    pub added:       Vec<SegmentEvent>,
}

pub fn emit_segments_replaced(app: &tauri::AppHandle, ev: SegmentsReplacedEvent) {
    let _ = app.emit("segments_replaced", ev);
}

/// Emitted after every accumulator append and after every drain (with speech_secs=0.0 on drain).
#[derive(serde::Serialize, Clone)]
pub struct AccumulatorUpdatedEvent {
    /// Seconds of speech currently buffered in the accumulator.
    pub speech_secs:  f64,
    /// Seconds at which the accumulator triggers a slow-path flush.
    pub trigger_secs: f64,
}

/// Emitted after every fast-path accumulator append and after every drain (with speech_secs=0.0 on drain).
#[derive(serde::Serialize, Clone)]
pub struct FastAccumulatorUpdatedEvent {
    /// Seconds of speech currently buffered in the fast-path accumulator.
    pub speech_secs:  f64,
    /// Seconds at which the fast-path accumulator triggers a flush.
    pub trigger_secs: f64,
}

pub fn emit_fast_accumulator_updated(app: &tauri::AppHandle, ev: FastAccumulatorUpdatedEvent) {
    let _ = app.emit("fast_accumulator_updated", ev);
}

/// Emitted just before a slow-path clip is dispatched to speech-swift.
#[derive(serde::Serialize, Clone)]
pub struct SlowPathSentEvent {
    /// Session-relative start of the clip (ms).
    pub start_ms:         u64,
    /// Session-relative end of the clip (ms).
    pub end_ms:           u64,
    /// Seconds of net speech in the clip (excludes silence padding).
    pub clip_speech_secs: f64,
    /// Wall-clock time the clip was dispatched (Unix ms).
    pub sent_at_ms:       u64,
}

/// Emitted after speech-swift returns a successful slow-path response.
#[derive(serde::Serialize, Clone)]
pub struct SlowPathDoneEvent {
    /// Matches the `start_ms` of the corresponding `SlowPathSentEvent`.
    pub start_ms:      u64,
    /// Round-trip time for the speech-swift HTTP call (ms).
    pub response_ms:   u64,
    /// Number of segments in the slow-path response.
    pub segment_count: u32,
    /// Minimum best_score across all segments (worst speaker match confidence).
    /// None when speech-swift did not return scores for any segment.
    pub best_score:    Option<f32>,
}

pub fn emit_accumulator_updated(app: &tauri::AppHandle, ev: AccumulatorUpdatedEvent) {
    let _ = app.emit("accumulator_updated", ev);
}

pub fn emit_slow_path_sent(app: &tauri::AppHandle, ev: SlowPathSentEvent) {
    let _ = app.emit("slow_path_sent", ev);
}

pub fn emit_slow_path_done(app: &tauri::AppHandle, ev: SlowPathDoneEvent) {
    let _ = app.emit("slow_path_done", ev);
}

/// Emitted when speech-swift is unreachable (network error, sidecar crash, etc.).
/// The frontend listens for this to surface an error panel during recording.
/// Payload is `null` — the frontend only needs to know the event occurred.
pub fn emit_speech_swift_unreachable(app: &tauri::AppHandle) {
    let _ = app.emit("speech_swift_unreachable", ());
}

// ── Speaker mutation events ───────────────────────────────────────────────────
//
// Emitted after each speaker management operation so every piece of UI
// (recording sidebar, speaker registry page, session detail) can react
// without polling or manual cache invalidation.

/// Emitted after a speaker is successfully renamed in speech-swift and the local DB.
#[derive(serde::Serialize, Clone)]
pub struct SpeakerRenamedEvent {
    pub speech_swift_id: i64,
    pub display_name:    String,
}

/// Emitted after two speakers are merged.
/// `src_id` was absorbed and deleted; `dst_id` is the survivor.
/// All segments that referenced `src_id` now reference `dst_id`.
#[derive(serde::Serialize, Clone)]
pub struct SpeakersMergedEvent {
    pub src_id:           i64,
    pub dst_id:           i64,
    pub dst_display_name: Option<String>,
}

/// Emitted after a speaker is deleted from speech-swift and the local DB.
/// Segments that referenced this speaker now have `speaker_id = NULL`.
#[derive(serde::Serialize, Clone)]
pub struct SpeakerDeletedEvent {
    pub speech_swift_id: i64,
}

/// Emitted after the entire speaker registry is reset.
/// All speaker associations on segments are now NULL.
#[derive(serde::Serialize, Clone)]
pub struct SpeakerRegistryResetEvent {}

pub fn emit_speaker_renamed(app: &tauri::AppHandle, ev: SpeakerRenamedEvent) {
    let _ = app.emit("speaker_renamed", ev);
}

pub fn emit_speakers_merged(app: &tauri::AppHandle, ev: SpeakersMergedEvent) {
    let _ = app.emit("speakers_merged", ev);
}

pub fn emit_speaker_deleted(app: &tauri::AppHandle, ev: SpeakerDeletedEvent) {
    let _ = app.emit("speaker_deleted", ev);
}

pub fn emit_speaker_registry_reset(app: &tauri::AppHandle) {
    let _ = app.emit("speaker_registry_reset", SpeakerRegistryResetEvent {});
}
