use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, serde::Serialize)]
pub struct SegmentWithSpeaker {
    pub id:              i64,
    pub session_id:      i64,
    pub speaker_id:      Option<i64>,
    pub start_ms:        i64,
    pub end_ms:          i64,
    pub transcript_text: String,
    pub display_name:    Option<String>,
    pub status:          String,
}

pub struct NewSegment {
    pub session_id:       i64,
    /// `None` when the fast path produced no confident speaker ID (pending).
    pub speaker_id:       Option<i64>,
    pub start_ms:         i64,
    pub end_ms:           i64,
    pub transcript_text:  String,
    /// Session-relative start of the enclosing VAD chunk (seconds).
    pub chunk_start_secs: Option<f64>,
    /// Session-relative end of the enclosing VAD chunk (seconds).
    pub chunk_end_secs:   Option<f64>,
}

/// Insert a transcript segment and return its new row id.
///
/// Sets `status = 'pending'` when `speaker_id` is None, `'confirmed'`
/// otherwise. `chunk_start` and `chunk_end` store the enclosing VAD chunk's
/// session-relative position so the slow path can match long-clip segments back
/// to fast-path rows.
pub fn insert_segment(conn: &Connection, seg: &NewSegment) -> anyhow::Result<i64> {
    // Guard against duplicate segments (speech-swift occasionally returns the
    // same time-range segment twice in one response). Return the existing row's
    // ID so the caller can still wire up speaker resolution correctly.
    let existing_id: Option<i64> = conn.query_row(
        "SELECT id FROM segments WHERE session_id = ?1 AND start_ms = ?2 AND end_ms = ?3",
        rusqlite::params![seg.session_id, seg.start_ms, seg.end_ms],
        |r| r.get(0),
    ).optional()?;

    if let Some(id) = existing_id {
        return Ok(id);
    }

    let status = if seg.speaker_id.is_some() { "confirmed" } else { "pending" };
    conn.execute(
        "INSERT INTO segments
             (session_id, speaker_id, start_ms, end_ms, transcript_text, status, chunk_start, chunk_end)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            seg.session_id,
            seg.speaker_id,
            seg.start_ms,
            seg.end_ms,
            &seg.transcript_text,
            status,
            seg.chunk_start_secs,
            seg.chunk_end_secs,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Set `speaker_id` and mark `status = 'confirmed'` on a segment.
///
/// Called by the slow path after a long-clip diarization resolves a speaker ID
/// for a previously pending segment.
pub fn update_segment_speaker(
    conn: &Connection,
    segment_id: i64,
    speaker_id: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE segments SET speaker_id = ?1, status = 'confirmed' WHERE id = ?2",
        rusqlite::params![speaker_id, segment_id],
    )?;
    Ok(())
}

/// Return all segments for a session with the speaker's display name joined in.
/// Ordered by start_ms ascending so the caller gets chronological order.
pub fn get_segments_with_speakers(
    conn: &Connection,
    session_id: i64,
) -> anyhow::Result<Vec<SegmentWithSpeaker>> {
    let mut stmt = conn.prepare(
        "SELECT sg.id, sg.session_id, sg.speaker_id, sg.start_ms, sg.end_ms,
                sg.transcript_text, sp.display_name, sg.status
         FROM segments sg
         LEFT JOIN speakers sp ON sp.speech_swift_id = sg.speaker_id
         WHERE sg.session_id = ?1
         ORDER BY sg.start_ms ASC",
    )?;
    let rows = stmt
        .query_map([session_id], |row| {
            Ok(SegmentWithSpeaker {
                id:              row.get(0)?,
                session_id:      row.get(1)?,
                speaker_id:      row.get(2)?,
                start_ms:        row.get(3)?,
                end_ms:          row.get(4)?,
                transcript_text: row.get(5)?,
                display_name:    row.get(6)?,
                status:          row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Insert a 384-dim embedding for a segment into the sqlite-vec virtual table.
///
/// The embedding is stored as raw little-endian f32 bytes, which is the format
/// sqlite-vec expects for FLOAT[] columns.
pub fn insert_segment_embedding(
    conn: &Connection,
    segment_id: i64,
    embedding: &[f32],
) -> anyhow::Result<()> {
    let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
    conn.execute(
        "INSERT INTO segment_embeddings (segment_id, embedding) VALUES (?1, ?2)",
        rusqlite::params![segment_id, blob],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    /// Returns `(Connection, TempDir)` — caller must keep both alive.
    /// TempDir must outlive the Connection; dropping it deletes the directory
    /// and triggers SQLITE_READONLY_DBMOVED on any subsequent write.
    fn open_test_db() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let conn = db::open(&dir.path().join("test.db")).expect("open");
        (conn, dir)
    }

    #[test]
    fn insert_segment_returns_row_id() {
        let (conn, _dir) = open_test_db();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        let seg = NewSegment {
            session_id,
            speaker_id: Some(42),
            start_ms: 0,
            end_ms: 1000,
            transcript_text: "hello world".into(),
            chunk_start_secs: Some(0.0),
            chunk_end_secs: Some(1.0),
        };
        let id = insert_segment(&conn, &seg).expect("insert");
        assert!(id > 0);
    }

    #[test]
    fn insert_segment_pending_when_no_speaker() {
        let (conn, _dir) = open_test_db();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        let seg = NewSegment {
            session_id,
            speaker_id: None,
            start_ms: 0,
            end_ms: 1000,
            transcript_text: "hello world".into(),
            chunk_start_secs: Some(0.0),
            chunk_end_secs: Some(1.0),
        };
        let id = insert_segment(&conn, &seg).expect("insert");

        let status: String = conn
            .query_row(
                "SELECT status FROM segments WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "pending");
    }

    #[test]
    fn update_segment_speaker_sets_confirmed() {
        let (conn, _dir) = open_test_db();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        let id = insert_segment(
            &conn,
            &NewSegment {
                session_id,
                speaker_id: None,
                start_ms: 0,
                end_ms: 500,
                transcript_text: "test".into(),
                chunk_start_secs: None,
                chunk_end_secs: None,
            },
        )
        .unwrap();

        update_segment_speaker(&conn, id, 7).expect("update");

        let (spk, status): (Option<i64>, String) = conn
            .query_row(
                "SELECT speaker_id, status FROM segments WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(spk, Some(7));
        assert_eq!(status, "confirmed");
    }

    #[test]
    fn insert_segment_deduplicates_same_time_range() {
        let (conn, _dir) = open_test_db();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        let seg = NewSegment {
            session_id,
            speaker_id: Some(1),
            start_ms: 0,
            end_ms: 1000,
            transcript_text: "hello".into(),
            chunk_start_secs: Some(0.0),
            chunk_end_secs: Some(1.0),
        };

        let id1 = insert_segment(&conn, &seg).expect("first insert");
        let id2 = insert_segment(&conn, &seg).expect("second insert (duplicate)");

        // Both calls must return the same row ID — no duplicate row created.
        assert_eq!(id1, id2);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM segments WHERE session_id = ?1",
                [session_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn insert_segment_embedding_stores_blob() {
        let (conn, _dir) = open_test_db();
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        let seg_id = insert_segment(
            &conn,
            &NewSegment {
                session_id,
                speaker_id: Some(1),
                start_ms: 0,
                end_ms: 500,
                transcript_text: "test".into(),
                chunk_start_secs: None,
                chunk_end_secs: None,
            },
        )
        .unwrap();

        // 384-dim embedding of all zeros.
        let embedding = vec![0.0f32; 384];
        insert_segment_embedding(&conn, seg_id, &embedding).expect("embed insert");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM segment_embeddings WHERE segment_id = ?1",
                [seg_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
