use rusqlite::Connection;

#[derive(Debug, serde::Serialize)]
pub struct SegmentWithSpeaker {
    pub id:              i64,
    pub session_id:      i64,
    pub speaker_id:      Option<i64>,
    pub start_ms:        i64,
    pub end_ms:          i64,
    pub transcript_text: String,
    pub display_name:    Option<String>,
}

pub struct NewSegment {
    pub session_id:      i64,
    pub speaker_id:      i64,
    pub start_ms:        i64,
    pub end_ms:          i64,
    pub transcript_text: String,
}

/// Insert a transcript segment and return its new row id.
pub fn insert_segment(conn: &Connection, seg: &NewSegment) -> anyhow::Result<i64> {
    conn.execute(
        "INSERT INTO segments (session_id, speaker_id, start_ms, end_ms, transcript_text)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            seg.session_id,
            seg.speaker_id,
            seg.start_ms,
            seg.end_ms,
            &seg.transcript_text,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Return all segments for a session with the speaker's display name joined in.
/// Ordered by start_ms ascending so the caller gets chronological order.
pub fn get_segments_with_speakers(
    conn: &Connection,
    session_id: i64,
) -> anyhow::Result<Vec<SegmentWithSpeaker>> {
    let mut stmt = conn.prepare(
        "SELECT sg.id, sg.session_id, sg.speaker_id, sg.start_ms, sg.end_ms,
                sg.transcript_text, sp.display_name
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
            speaker_id: 42,
            start_ms: 0,
            end_ms: 1000,
            transcript_text: "hello world".into(),
        };
        let id = insert_segment(&conn, &seg).expect("insert");
        assert!(id > 0);
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
                speaker_id: 1,
                start_ms: 0,
                end_ms: 500,
                transcript_text: "test".into(),
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
