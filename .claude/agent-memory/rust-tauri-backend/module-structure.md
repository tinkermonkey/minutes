---
name: Module structure
description: Source tree layout and responsibility map for the Rust backend
type: project
---

`src-tauri/src/` layout after Stage 1:

- `main.rs` — binary entry point, calls `lib::run()`
- `lib.rs` — Tauri app wiring: plugin registration, setup hook, managed state, invoke_handler
- `state.rs` — `AppState` struct: `db: Mutex<Connection>`, `speech_swift: Mutex<SpeechSwiftStatus>`, `speech_swift_url: String`, `pipelines: Mutex<HashMap<i64, oneshot::Sender<()>>>`
- `db/mod.rs` — `open(path) -> anyhow::Result<Connection>`: registers sqlite-vec auto-extension, runs migrations, creates vec0 virtual table
- `db/migrations.rs` — `migrations() -> Migrations<'static>`: all rusqlite_migration `M::up` definitions
- `db/segments.rs` — `insert_segment`, `insert_segment_embedding`
- `db/speakers.rs` — `upsert_speaker` (insert-or-touch last_seen_at), returns `(Speaker, is_new)`; `list_with_stats`, `merge_speaker_local`, `delete_speaker_local`, `get_sample_path`
- `db/samples.rs` — `insert_speaker_sample`
- `client/mod.rs` — re-exports client submodules
- `client/speech_swift.rs` — `health_check`, `transcribe_chunk` (multipart POST → `SessionResponse`); `list_speakers`, `rename_speaker`, `merge_speakers`, `delete_speaker`
- `audio/mod.rs` — re-exports audio submodules
- `audio/capture.rs` — `start_capture(preferred) -> CaptureHandle`; CPAL 16 kHz mono f32; linear interpolation resample
- `audio/vad.rs` — `VadClassifier`: 10 ms frames, 500 ms silence flush, 30 s hard cap, 200 ms tail padding
- `audio/chunker.rs` — `Chunker`: accumulates frames, calls VadClassifier, returns `(wav_bytes, start_ms, end_ms)`
- `embed/mod.rs` — `get_model()` (OnceLock lazy init), `embed(text) -> Vec<f32>` (fastembed AllMiniLML6V2, 384-dim)
- `events/mod.rs` — `emit_segment_added`, `emit_new_speaker` helpers; `SegmentEvent`, `SpeakerEvent` payloads
- `commands/mod.rs` — `start_session`, `stop_session` Tauri commands; `run_pipeline` spawns OS thread for capture+VAD, async task for network+DB
- `commands/speakers.rs` — `get_speakers`, `rename_speaker`, `merge_speakers`, `delete_speaker`, `get_speaker_sample_path` Tauri commands

SQLite tables (migration 1): `sessions`, `segments` (speaker_id NOT NULL), `speaker_samples` (speaker_id NOT NULL), `speakers`
Migration 2: makes `speaker_id` nullable in `segments` and `speaker_samples` (via table-rebuild for SQLite compat)
Virtual table: `segment_embeddings` (vec0, FLOAT[384]) — created in `db::open`, not in migrations

`lib.rs` startup async block: health-check then full registry sync (`list_speakers` → `upsert_speaker` for each, propagate display_name, emit `new_speaker` for unlabeled speakers). `unix_ms()` helper defined in both `lib.rs` and `commands/mod.rs` (intentional duplication — commands is not public to lib).
