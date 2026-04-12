# Rust/Tauri Backend Agent Memory

- [Module structure](module-structure.md) — source tree layout and responsibility map for the Rust backend
- [sqlite-vec loading pattern](sqlite-vec-loading.md) — how sqlite-vec extension is registered (auto_extension, not load_extension)
- [rusqlite migration pattern](rusqlite-migration-pattern.md) — vec0 virtual table created post-migration, not inside M::up
- [Tauri trait imports](tauri-trait-imports.md) — non-obvious trait imports required for Tauri APIs
- [tauri.conf.json macOS fields](tauri-conf-macos-fields.md) — correct field names for macOS bundle config
- [cpal 0.17 API](cpal-017-api.md) — SampleRate is u32 alias, try_with_sample_rate returns Option, name() is deprecated
- [VAD Send constraint](vad-send-constraint.md) — webrtc_vad::Vad is !Send; capture+VAD runs on OS thread, chunks flow to async consumer via mpsc
- [TempDir SQLite test pattern](tempdir-sqlite-test-pattern.md) — must return (Connection, TempDir) from test helpers to avoid SQLITE_READONLY_DBMOVED
- [rusqlite query_map lifetime](rusqlite-query-map-lifetime.md) — collect into a let binding before stmt goes out of scope; tail-expression chain hits E0597
- [sqlite-vec KNN query syntax](sqlite-vec-knn-syntax.md) — MATCH + ORDER BY distance + LIMIT required; query vector is LE f32 bytes; CTE join pattern for post-filter
- [axum 0.8 patterns](axum-08-patterns.md) — {param} not :param syntax; tower::ServiceExt::oneshot for tests; per-request open_readonly connections for ApiState
