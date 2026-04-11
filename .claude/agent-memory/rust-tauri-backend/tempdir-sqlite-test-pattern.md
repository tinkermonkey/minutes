---
name: TempDir must outlive Connection in SQLite tests
description: Dropping TempDir while Connection is open causes SQLITE_READONLY_DBMOVED (error code 1032); return both from helper
type: feedback
---

When using `tempfile::TempDir` with rusqlite in tests, the `TempDir` must remain alive for the entire test — dropping it deletes the directory, which triggers SQLite error code 1032 `SQLITE_READONLY_DBMOVED` on any subsequent write.

**Rule**: test helpers that open a DB must return `(Connection, TempDir)` as a tuple, not just the `Connection`.

**Why:** SQLite detects that the file it has open is no longer present (unlinked by TempDir drop) and marks the connection read-only.

**How to apply**: in every `#[cfg(test)]` helper that calls `db::open()`, return `(Connection, tempfile::TempDir)` and bind both in the test body as `let (conn, _dir) = open_test_db();`. The `_dir` binding keeps the TempDir alive until end-of-scope.
