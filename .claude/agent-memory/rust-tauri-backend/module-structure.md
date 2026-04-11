---
name: Module structure
description: Source tree layout and responsibility map for the Rust backend
type: project
---

`src-tauri/src/` layout after Stage 0:

- `main.rs` — binary entry point, calls `lib::run()`
- `lib.rs` — Tauri app wiring: plugin registration, setup hook, managed state, invoke_handler
- `state.rs` — `AppState` struct (`db: Mutex<Connection>`, `speech_swift: Mutex<SpeechSwiftStatus>`, `speech_swift_url: String`)
- `db/mod.rs` — `open(path) -> anyhow::Result<Connection>`: registers sqlite-vec auto-extension, runs migrations, creates vec0 virtual table
- `db/migrations.rs` — `migrations() -> Migrations<'static>`: all rusqlite_migration `M::up` definitions
- `client/mod.rs` — re-exports client submodules
- `client/speech_swift.rs` — `health_check(base_url) -> bool` and future audio-server HTTP calls

SQLite tables (migration 1): `sessions`, `segments`, `speaker_samples`, `speakers`
Virtual table: `segment_embeddings` (vec0, FLOAT[384]) — created in `db::open`, not in migrations
