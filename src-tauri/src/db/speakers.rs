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

/// Speaker row joined with aggregate statistics derived from `segments`.
#[derive(Debug, serde::Serialize, Clone)]
pub struct SpeakerWithStats {
    pub id:              i64,
    pub speech_swift_id: i64,
    pub display_name:    Option<String>,
    pub notes:           Option<String>,
    pub first_seen_at:   i64,
    pub last_seen_at:    i64,
    pub session_count:   i64,
}

/// List all speakers ordered by most-recently seen, with a session count.
pub fn list_with_stats(conn: &Connection) -> anyhow::Result<Vec<SpeakerWithStats>> {
    let mut stmt = conn.prepare(
        r#"
            SELECT
                s.id, s.speech_swift_id, s.display_name, s.notes,
                s.first_seen_at, s.last_seen_at,
                COUNT(DISTINCT sg.session_id) AS session_count
            FROM speakers s
            LEFT JOIN segments sg ON sg.speaker_id = s.speech_swift_id
            GROUP BY s.id
            ORDER BY s.last_seen_at DESC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SpeakerWithStats {
            id:              row.get(0)?,
            speech_swift_id: row.get(1)?,
            display_name:    row.get(2)?,
            notes:           row.get(3)?,
            first_seen_at:   row.get(4)?,
            last_seen_at:    row.get(5)?,
            session_count:   row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Re-point all segments and samples from `src` to `dst`, then delete the src
/// speaker row. Called after the audio-server merge succeeds.
pub fn merge_speaker_local(
    conn: &Connection,
    src_speech_swift_id: i64,
    dst_speech_swift_id: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE segments SET speaker_id = ?1 WHERE speaker_id = ?2",
        [dst_speech_swift_id, src_speech_swift_id],
    )?;
    conn.execute(
        "UPDATE speaker_samples SET speaker_id = ?1 WHERE speaker_id = ?2",
        [dst_speech_swift_id, src_speech_swift_id],
    )?;
    conn.execute(
        "DELETE FROM speakers WHERE speech_swift_id = ?1",
        [src_speech_swift_id],
    )?;
    Ok(())
}

/// NULL-out all segment/sample references to this speaker, then delete the
/// speaker row. Called after the audio-server delete succeeds.
pub fn delete_speaker_local(conn: &Connection, speech_swift_id: i64) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE segments SET speaker_id = NULL WHERE speaker_id = ?1",
        [speech_swift_id],
    )?;
    conn.execute(
        "UPDATE speaker_samples SET speaker_id = NULL WHERE speaker_id = ?1",
        [speech_swift_id],
    )?;
    conn.execute(
        "DELETE FROM speakers WHERE speech_swift_id = ?1",
        [speech_swift_id],
    )?;
    Ok(())
}

/// Remove all speaker data from the local database.
///
/// Called after `DELETE /registry/speakers` to keep local state in sync.
/// - Deletes all rows from `speakers` and `speaker_samples`.
/// - Sets `speaker_id = NULL` and `status = 'pending'` on all segments
///   so the transcript view shows them as unresolved.
pub fn reset_all(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "DELETE FROM speaker_samples;
         DELETE FROM speakers;
         UPDATE segments SET speaker_id = NULL, status = 'pending';",
    )?;
    Ok(())
}

/// Return the audio path of the most-recent sample for this speaker, if any.
pub fn get_sample_path(conn: &Connection, speech_swift_id: i64) -> anyhow::Result<Option<String>> {
    conn.query_row(
        "SELECT audio_path FROM speaker_samples
         WHERE speaker_id = ?1
         ORDER BY end_ms DESC LIMIT 1",
        [speech_swift_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
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

    #[test]
    fn list_with_stats_returns_all_speakers() {
        let (conn, _dir) = open_test_db();
        upsert_speaker(&conn, 10, 1_000).unwrap();
        upsert_speaker(&conn, 20, 2_000).unwrap();
        let rows = list_with_stats(&conn).expect("list");
        assert_eq!(rows.len(), 2);
        // Most-recently seen first.
        assert_eq!(rows[0].speech_swift_id, 20);
        assert_eq!(rows[1].speech_swift_id, 10);
        // No segments, so session_count is zero.
        assert_eq!(rows[0].session_count, 0);
    }

    #[test]
    fn merge_speaker_local_reassigns_and_removes_src() {
        let (conn, _dir) = open_test_db();
        upsert_speaker(&conn, 1, 1_000).unwrap();
        upsert_speaker(&conn, 2, 1_000).unwrap();

        // Insert a segment referencing speaker 1.
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO segments (session_id, speaker_id, start_ms, end_ms, transcript_text)
             VALUES (1, 1, 0, 1000, 'hello')",
            [],
        )
        .unwrap();

        merge_speaker_local(&conn, 1, 2).expect("merge");

        // speaker 1 row should be gone.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM speakers WHERE speech_swift_id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        // Segment should now point to speaker 2.
        let sid: i64 = conn
            .query_row(
                "SELECT speaker_id FROM segments WHERE session_id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sid, 2);
    }

    #[test]
    fn delete_speaker_local_nulls_references_and_removes_row() {
        let (conn, _dir) = open_test_db();
        upsert_speaker(&conn, 5, 1_000).unwrap();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO segments (session_id, speaker_id, start_ms, end_ms, transcript_text)
             VALUES (1, 5, 0, 1000, 'test')",
            [],
        )
        .unwrap();

        delete_speaker_local(&conn, 5).expect("delete");

        // Speaker row gone.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM speakers WHERE speech_swift_id = 5",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        // Segment speaker_id is now NULL.
        let sid: Option<i64> = conn
            .query_row(
                "SELECT speaker_id FROM segments WHERE session_id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(sid.is_none());
    }

    #[test]
    fn get_sample_path_returns_none_when_no_samples() {
        let (conn, _dir) = open_test_db();
        upsert_speaker(&conn, 7, 1_000).unwrap();
        let path = get_sample_path(&conn, 7).expect("query");
        assert!(path.is_none());
    }
}
