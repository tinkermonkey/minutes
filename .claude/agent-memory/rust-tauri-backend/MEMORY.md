# Rust/Tauri Backend Agent Memory

- [Module structure](module-structure.md) — source tree layout and responsibility map for the Rust backend
- [sqlite-vec loading pattern](sqlite-vec-loading.md) — how sqlite-vec extension is registered (auto_extension, not load_extension)
- [rusqlite migration pattern](rusqlite-migration-pattern.md) — vec0 virtual table created post-migration, not inside M::up
- [Tauri trait imports](tauri-trait-imports.md) — non-obvious trait imports required for Tauri APIs
- [tauri.conf.json macOS fields](tauri-conf-macos-fields.md) — correct field names for macOS bundle config
