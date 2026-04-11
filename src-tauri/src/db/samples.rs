use rusqlite::Connection;

/// Record that a segment of audio was saved to `audio_path` and is associated
/// with `speaker_id`. Used to build a speaker sample library for future
/// identification quality improvements.
pub fn insert_speaker_sample(
    conn: &Connection,
    speaker_id: i64,
    session_id: i64,
    start_ms:   i64,
    end_ms:     i64,
    audio_path: &str,
) -> anyhow::Result<i64> {
    conn.execute(
        "INSERT INTO speaker_samples (speaker_id, session_id, start_ms, end_ms, audio_path)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![speaker_id, session_id, start_ms, end_ms, audio_path],
    )?;
    Ok(conn.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    /// Returns `(Connection, TempDir)` — caller must keep both alive.
    fn open_test_db() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let conn = db::open(&dir.path().join("test.db")).expect("open");
        (conn, dir)
    }

    #[test]
    fn insert_speaker_sample_returns_row_id() {
        let (conn, _dir) = open_test_db();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO speakers (speech_swift_id, first_seen_at, last_seen_at) VALUES (1, 1000, 1000)",
            [],
        )
        .unwrap();
        let speaker_id = conn.last_insert_rowid();

        let id = insert_speaker_sample(&conn, speaker_id, session_id, 0, 1000, "/tmp/chunk.wav")
            .expect("insert");
        assert!(id > 0);
    }
}
