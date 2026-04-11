use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Speaker {
    pub id:              i64,
    pub speech_swift_id: i64,
    pub display_name:    Option<String>,
    pub notes:           Option<String>,
    pub first_seen_at:   i64,
    pub last_seen_at:    i64,
}

fn row_to_speaker(row: &rusqlite::Row<'_>) -> rusqlite::Result<Speaker> {
    Ok(Speaker {
        id:              row.get(0)?,
        speech_swift_id: row.get(1)?,
        display_name:    row.get(2)?,
        notes:           row.get(3)?,
        first_seen_at:   row.get(4)?,
        last_seen_at:    row.get(5)?,
    })
}

/// Upsert a speaker by their speech-swift registry id.
///
/// - If the speaker does not yet exist, inserts a new row.
/// - If the speaker exists, bumps `last_seen_at`.
///
/// Returns `(speaker, is_new)`.
pub fn upsert_speaker(
    conn: &Connection,
    speech_swift_id: i64,
    now_ms: i64,
) -> anyhow::Result<(Speaker, bool)> {
    let existing = conn
        .query_row(
            "SELECT id, speech_swift_id, display_name, notes, first_seen_at, last_seen_at
             FROM speakers WHERE speech_swift_id = ?1",
            [speech_swift_id],
            row_to_speaker,
        )
        .optional()?;

    if let Some(speaker) = existing {
        conn.execute(
            "UPDATE speakers SET last_seen_at = ?1 WHERE speech_swift_id = ?2",
            [now_ms, speech_swift_id],
        )?;
        Ok((speaker, false))
    } else {
        conn.execute(
            "INSERT INTO speakers (speech_swift_id, first_seen_at, last_seen_at)
             VALUES (?1, ?2, ?3)",
            [speech_swift_id, now_ms, now_ms],
        )?;
        let speaker = conn.query_row(
            "SELECT id, speech_swift_id, display_name, notes, first_seen_at, last_seen_at
             FROM speakers WHERE speech_swift_id = ?1",
            [speech_swift_id],
            row_to_speaker,
        )?;
        Ok((speaker, true))
    }
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
    fn upsert_new_speaker_sets_is_new() {
        let (conn, _dir) = open_test_db();
        let (speaker, is_new) = upsert_speaker(&conn, 99, 1_000).expect("upsert");
        assert!(is_new);
        assert_eq!(speaker.speech_swift_id, 99);
        assert_eq!(speaker.first_seen_at, 1_000);
        assert_eq!(speaker.last_seen_at, 1_000);
        assert!(speaker.display_name.is_none());
    }

    #[test]
    fn upsert_existing_speaker_not_new() {
        let (conn, _dir) = open_test_db();
        let (s1, is_new1) = upsert_speaker(&conn, 7, 1_000).expect("first upsert");
        assert!(is_new1);

        let (s2, is_new2) = upsert_speaker(&conn, 7, 2_000).expect("second upsert");
        assert!(!is_new2);
        assert_eq!(s1.id, s2.id);
        // The returned speaker still has the snapshot from before the UPDATE,
        // which is acceptable — the caller uses it for event emission.
        assert_eq!(s2.first_seen_at, 1_000);
    }

    #[test]
    fn upsert_different_ids_are_independent() {
        let (conn, _dir) = open_test_db();
        let (a, _) = upsert_speaker(&conn, 1, 1_000).unwrap();
        let (b, _) = upsert_speaker(&conn, 2, 1_000).unwrap();
        assert_ne!(a.id, b.id);
    }
}
