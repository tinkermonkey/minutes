pub mod migrations;
pub mod samples;
pub mod search;
pub mod segments;
pub mod sessions;
pub mod settings;
pub mod speakers;

use rusqlite::Connection;
use sqlite_vec::sqlite3_vec_init;
use std::path::Path;

/// Open a SQLite connection at `db_path`, register the sqlite-vec extension
/// as an auto-extension (so every connection gets it automatically), run all
/// pending migrations, and create the vec0 virtual table if it does not exist.
///
/// # Safety
/// `sqlite3_auto_extension` is an unsafe FFI call. We transmute the bare
/// function pointer to the signature SQLite expects for auto-extensions. This
/// is the canonical pattern from the sqlite-vec crate itself.
pub fn open(db_path: &Path) -> anyhow::Result<Connection> {
    // Register sqlite-vec as a permanent auto-extension so the virtual table
    // module is available on every connection opened in this process. Calling
    // this multiple times for the same function pointer is idempotent.
    unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite3_vec_init as *const (),
        )));
    }

    let mut conn = Connection::open(db_path)?;

    migrations::migrations().to_latest(&mut conn)?;

    // segment_embeddings stores 384-dim float vectors (all-MiniLM-L6-v2 size).
    // Created here rather than in a migration because vec0 requires the
    // sqlite-vec extension to be loaded, which is process-level state and
    // not guaranteed to be present when rusqlite_migration runs.
    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS segment_embeddings USING vec0(
            segment_id INTEGER PRIMARY KEY,
            embedding  FLOAT[384]
        );
    "#,
    )?;

    Ok(conn)
}

/// Open a read-only SQLite connection to an existing database.
///
/// Used by the axum REST API server for concurrent reads without interfering
/// with the single write connection in `AppState`. The sqlite-vec extension is
/// available on this connection automatically because `sqlite3_auto_extension`
/// is registered globally by `open()` at app startup — it applies to every new
/// connection opened in the process, including read-only ones.
pub fn open_readonly(db_path: &Path) -> anyhow::Result<Connection> {
    let conn = Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_creates_schema() {
        // Use a temporary file so open() path is exercised end-to-end.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        let conn = open(&path).expect("open should succeed");

        // Verify vec_version() is reachable (proves sqlite-vec is loaded).
        let ver: String = conn
            .query_row("SELECT vec_version()", [], |r| r.get(0))
            .expect("vec_version");
        assert!(ver.starts_with('v'), "expected version string, got {ver}");

        // Verify the virtual table was created.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='segment_embeddings'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 1);
    }
}
