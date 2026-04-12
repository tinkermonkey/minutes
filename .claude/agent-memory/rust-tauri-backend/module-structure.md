---
name: Module structure
description: Source tree layout and responsibility map for the Rust backend
type: project
---

`src-tauri/src/` layout after Stage 6 (MVP Polish):

- `main.rs` — binary entry point, calls `lib::run()`
- `lib.rs` — Tauri app wiring: plugin registration, setup hook, managed state, invoke_handler; spawns REST API server
- `state.rs` — `AppState` struct: `db: Mutex<Connection>`, `speech_swift: Mutex<SpeechSwiftStatus>`, `speech_swift_url: String`, `pipelines: Mutex<HashMap<i64, oneshot::Sender<()>>>`, `preferred_device: Mutex<Option<String>>`
- `db/mod.rs` — `open(path)`: registers sqlite-vec auto-extension, runs migrations, creates vec0 virtual table; `open_readonly(path)`: read-only connection for axum (auto_extension applies globally so no extra extension loading needed)
- `db/migrations.rs` — 3 migrations: (1) initial schema, (2) nullable speaker_id, (3) settings table
- `db/settings.rs` — `get(conn, key) -> Option<String>`, `set(conn, key, value)` — simple KV settings table
- `db/segments.rs` — `insert_segment`, `insert_segment_embedding`; `get_segments_with_speakers` → `Vec<SegmentWithSpeaker>`
- `db/sessions.rs` — `list_sessions(filter) -> SessionsPage` (paginated); `get_session_by_id` → `Option<SessionRow>`
- `db/speakers.rs` — `upsert_speaker`, `list_with_stats`, `merge_speaker_local`, `delete_speaker_local`, `get_sample_path`
- `db/samples.rs` — `insert_speaker_sample`
- `db/search.rs` — `search_segments(conn, embedding, filters) -> Vec<SearchResult>` (vec0 KNN)
- `client/mod.rs` — re-exports client submodules
- `client/speech_swift.rs` — `health_check`, `transcribe_chunk`, `list_speakers`, `rename_speaker`, `merge_speakers`, `delete_speaker`
- `audio/mod.rs` — re-exports audio submodules
- `audio/capture.rs` — `start_capture(preferred) -> CaptureHandle`; CPAL 16 kHz mono f32; linear interpolation resample
- `audio/vad.rs` — `VadClassifier`: 10 ms frames, 500 ms silence flush, 30 s hard cap, 200 ms tail padding
- `audio/chunker.rs` — `Chunker`: accumulates frames, calls VadClassifier, returns `(wav_bytes, start_ms, end_ms)`
- `embed/mod.rs` — `get_model()` (OnceLock lazy init), `embed(text) -> Vec<f32>` (fastembed AllMiniLML6V2, 384-dim)
- `events/mod.rs` — `emit_segment_added`, `emit_new_speaker` helpers; `SegmentEvent`, `SpeakerEvent` payloads
- `commands/mod.rs` — `start_session`, `stop_session` Tauri commands; `run_pipeline` (OS thread for capture+VAD, async task for network+DB)
- `commands/speakers.rs` — `get_speakers`, `rename_speaker`, `merge_speakers`, `delete_speaker`, `get_speaker_sample_path`
- `commands/sessions.rs` — `get_sessions`, `get_session`, `get_segments`
- `commands/search.rs` — `search_segments`
- `commands/devices.rs` — `get_audio_devices() -> Vec<AudioDevice>`, `set_audio_device(device_name)` (persists to settings, updates preferred_device in AppState)
- `commands/health.rs` — `retry_health_check()` (re-probes, emits event, updates AppState), `set_speech_swift_port()` (persists to settings; restart required to take effect)
- `api/mod.rs` — `router(ApiState) -> Router`, `serve(db_path)` — axum REST API on 127.0.0.1:8765; `ApiState { db_path: PathBuf }`
- `api/sessions.rs` — GET /sessions (paginated list), GET /sessions/{id}/segments
- `api/speakers.rs` — GET /speakers
- `api/search.rs` — GET /search?q=... (embeds query, KNN search)

SQLite tables: `sessions`, `segments`, `speaker_samples`, `speakers` (migrations 1+2), `settings` (migration 3)
Virtual table: `segment_embeddings` (vec0, FLOAT[384]) — created in `db::open`, not in migrations

Plugins: `tauri-plugin-opener`, `tauri-plugin-dialog`, `tauri-plugin-window-state`
Capabilities: `core:default`, `core:window:allow-start-dragging`, `core:window:allow-set-title`, `opener:default`, `dialog:default`, `window-state:allow-restore-state`, `window-state:allow-save-window-state`

`lib.rs` startup: (1) open DB, (2) load preferred_device from settings, (3) manage AppState, (4) async health-check + registry sync, (5) spawn REST API server.
`unix_ms()` defined in both `lib.rs` and `commands/mod.rs` (intentional duplication — commands is not public to lib).
