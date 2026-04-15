---
name: dual-stream pipeline wiring
description: How the dual-stream audio pipeline is wired — fast path saves pending segments, slow path resolves speaker IDs retroactively
type: project
---

The pipeline runs two parallel streams from the same VAD-gated audio:

**Fast path** (`handle_chunk`): sends each VAD chunk to speech-swift immediately, saves segments to SQLite with `status='pending'` when `speaker_id` is None, `status='confirmed'` when a speaker ID is returned. Returns `Vec<i64>` (segment IDs inserted) so the caller can feed the accumulator. Emits `segment_added` Tauri events.

**Slow path** (`run_slow_path`): called when `SpeechAccumulator` triggers (10s of speech, 3s inactivity, or session end). Encodes accumulated frames to WAV via `crate::audio::chunker::encode_wav` (pub(crate)), calls `speech_swift::transcribe_chunk`, maps long-clip segment times back to fast-path DB rows via `AccumulatorClip.chunks`, calls `db::segments::update_segment_speaker`, emits `segment_speaker_resolved` events. Signature: `async fn run_slow_path(base_url: &str, app: &tauri::AppHandle, accumulator: &mut crate::audio::SpeechAccumulator)`.

**ChunkerOutput struct** (in `chunker.rs`): `{ wav_bytes: Vec<u8>, speech_frames: Vec<f32>, start_ms: u64, end_ms: u64 }`. Both `DynChunker::push_samples` and `DynChunker::flush` return `Option<ChunkerOutput>` — the speech_frames field carries raw f32 for the accumulator.

**VAD state events**: `DynChunker::is_speech()` reads `c.vad.last_frame_was_speech` (field added to `VadClassifier`). The OS capture thread tracks `last_vad_active: bool` and calls `events::emit_vad_state` on transitions after each `push_samples` call.

**Key types:**
- `AppendedChunk` in `accumulator.rs`: records `frame_start`, `frame_end` (indices into `SpeechAccumulator::frames`), `session_start_ms`, `session_end_ms`, and `segment_ids` from the fast path
- `AccumulatorClip.chunks`: populated by `drain()`, consumed by `run_slow_path`
- `NewSegment.chunk_start_secs` / `chunk_end_secs`: populated from `chunk.start_ms as f64 / 1000.0`

**Slow-path time math**: `seg.start` / `seg.end` from speech-swift are seconds *relative to the submitted clip*, not the session. Add `clip.clip_start_ms` to convert to session ms before matching against `AppendedChunk.session_start_ms` / `session_end_ms`.

**Inner async fn pattern**: `maybe_run_slow_path` is an `async fn` defined inside `run_pipeline` to avoid threading all parameters through a free function signature. Rust 2021 supports this.

**Why:** The speech-swift registry needs ≥30s of cumulative speech for reliable centroid enrollment. Sending 2–5s VAD chunks causes an exploding registry with no matches. The dual stream gives low-latency transcription AND enrollment-quality speaker IDs.
