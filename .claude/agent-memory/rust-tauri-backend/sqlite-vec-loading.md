---
name: sqlite-vec loading pattern
description: How sqlite-vec extension is registered — auto_extension not load_extension
type: reference
---

sqlite-vec (v0.1.9) exposes a single bare FFI symbol `sqlite3_vec_init`. It does NOT provide a `load_extension_from_init_fn`-style Rust helper.

The correct registration pattern (from the crate's own test):

```rust
unsafe {
    rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
        sqlite3_vec_init as *const (),
    )));
}
```

This is a process-level registration — once called, every `Connection` opened in the process will have the vec0 virtual table module available automatically.

**Why not `load_extension`:** The `load_extension` path requires the `load_extension` feature flag on rusqlite and works by path to a `.dylib`. The `auto_extension` path is self-contained, works with the bundled SQLite, and is the pattern endorsed by the sqlite-vec crate itself.

**Idempotency:** Calling `sqlite3_auto_extension` with the same function pointer multiple times is safe (SQLite deduplicates).

**Read-only connections:** `db::open_readonly` does NOT need to re-register the extension. The `auto_extension` is process-global — it applies to all connections opened after registration, including read-only ones opened via `Connection::open_with_flags`. As long as `db::open()` (which registers the auto_extension) is called first at startup, all subsequent `open_readonly` calls get the extension automatically.
