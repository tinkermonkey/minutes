---
name: rusqlite migration pattern
description: vec0 virtual table must be created after migrations run, not inside M::up
type: reference
---

The `segment_embeddings` vec0 virtual table is created in `db::open()` after `migrations().to_latest()` runs:

```rust
conn.execute_batch(r#"
    CREATE VIRTUAL TABLE IF NOT EXISTS segment_embeddings USING vec0(
        segment_id INTEGER PRIMARY KEY,
        embedding  FLOAT[384]
    );
"#)?;
```

**Why not inside M::up:** The vec0 module is only available after `sqlite3_auto_extension` is called. rusqlite_migration runs migrations before the extension registration could be guaranteed in all code paths. Keeping the virtual table creation in `db::open()` makes the dependency explicit and avoids migration failures if the extension is ever not loaded.

Standard schema tables (sessions, segments, speaker_samples, speakers) live in migration 1 in `db/migrations.rs`.
