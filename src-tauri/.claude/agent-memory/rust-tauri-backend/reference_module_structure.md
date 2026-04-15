---
name: rust module structure
description: Where specific responsibilities live in the Rust source tree
type: reference
---

- `src/audio/accumulator.rs` — `SpeechAccumulator`: slow-path silence-stripped frame accumulation, `FrameIndex` type for timestamp mapping, `clip_time_to_session_secs`
- `src/audio/chunker.rs` — `Chunker`: VAD-gated WAV encoding. `encode_wav` is `pub(crate)` so accumulator can reuse it. `push_samples` and `flush` return `(Vec<u8>, Vec<f32>, u64, u64)` — wav bytes, raw speech frames, start_ms, end_ms.
- `src/audio/capture.rs` — CPAL capture, resampling to 16 kHz
- `src/audio/vad.rs` — `VadClassifier`: Silero VAD via `ort` ONNX Runtime. Frame size 512 samples (32 ms @ 16 kHz). LSTM h/c state kept as flat `Vec<f32>` (128 elements each). Model embedded via `include_bytes!` from `models/silero_vad.onnx` (downloaded by `build.rs`, gitignored). Session held in `OnceLock<Session>`.
- `src/client/speech_swift.rs` — HTTP client to audio-server. `transcribe_chunk` delegates to `transcribe_chunk_with_options`. `SegmentResponse.speaker_id` and `speaker_label` are `Option` (null when min_duration guard skips recognition).
- `src/commands/mod.rs` — `handle_chunk` (fast path), `handle_long_clip` (slow path), `run_pipeline`, `start_session`/`stop_session` Tauri commands
- `src/db/segments.rs` — `NewSegment`, `insert_segment`, `resolve_pending_segments`, `get_segments_with_speakers`
- `src/db/migrations.rs` — 4 migrations; migration 4 adds `status`, `chunk_start`, `chunk_end` to segments
- `src/events/mod.rs` — `SegmentEvent` (speaker fields are `Option`), `SpeakerResolvedEvent`, emit functions
- `src/db/speakers.rs` — `upsert_speaker`, `delete_speaker`, `merge_speaker_local`
- `src/db/sessions.rs` — session CRUD
- `src/db/search.rs` — semantic search via sqlite-vec embeddings
- `src/state.rs` — `AppState` (db mutex, pipelines map, preferred_device)
- `src/api/` — axum read-only REST API on 127.0.0.1:8765
