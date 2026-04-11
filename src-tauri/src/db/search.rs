use rusqlite::Connection;

#[derive(Debug, serde::Deserialize)]
pub struct SearchFilters {
    pub speaker_id:  Option<i64>,
    pub start_date:  Option<i64>,
    pub end_date:    Option<i64>,
    pub limit:       Option<u32>,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub segment_id:         i64,
    pub transcript_text:    String,
    pub start_ms:           i64,
    pub end_ms:             i64,
    pub speaker_id:         Option<i64>,
    pub display_name:       Option<String>,
    pub session_id:         i64,
    pub session_label:      Option<String>,
    pub session_created_at: i64,
    pub score:              f32,
}

/// ANN search over `segment_embeddings`.
///
/// The sqlite-vec 0.1.9 KNN syntax requires a `MATCH` constraint on the vector
/// column plus either a `LIMIT` or `k = ?` to bound the result set. We use a
/// CTE to run the inner KNN query with a bare `LIMIT` (which sqlite-vec
/// recognises as the k parameter), then join the segment and session rows in
/// the outer query and apply the optional caller-supplied filters there.
///
/// Optional filters (`speaker_id`, `start_date`, `end_date`) are passed as
/// nullable SQL params — `?N IS NULL OR <column> = ?N` — so a single prepared
/// statement handles all filter combinations without string interpolation.
pub fn search_segments(
    conn:      &Connection,
    embedding: &[f32],
    filters:   &SearchFilters,
) -> anyhow::Result<Vec<SearchResult>> {
    let limit = filters.limit.unwrap_or(50).min(100) as i64;

    // Serialize embedding to LE bytes — the format sqlite-vec expects for
    // FLOAT[] columns (same as insert_segment_embedding in db/segments.rs).
    let blob: Vec<u8> = embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    let sql = r#"
        WITH matches AS (
            SELECT segment_id, distance
            FROM   segment_embeddings
            WHERE  embedding MATCH ?1
            ORDER  BY distance
            LIMIT  ?2
        )
        SELECT
            sg.id,
            sg.transcript_text,
            sg.start_ms,
            sg.end_ms,
            sg.speaker_id,
            sp.display_name,
            ss.id,
            ss.label,
            ss.created_at,
            m.distance
        FROM  matches m
        JOIN  segments sg ON sg.id    = m.segment_id
        JOIN  sessions ss ON ss.id    = sg.session_id
        LEFT  JOIN speakers sp ON sp.speech_swift_id = sg.speaker_id
        WHERE (?3 IS NULL OR sg.speaker_id  = ?3)
          AND (?4 IS NULL OR ss.created_at >= ?4)
          AND (?5 IS NULL OR ss.created_at <= ?5)
        ORDER BY m.distance ASC
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(
            rusqlite::params![
                blob,
                limit,
                filters.speaker_id,
                filters.start_date,
                filters.end_date,
            ],
            |row| {
                let distance: f64 = row.get(9)?;
                Ok(SearchResult {
                    segment_id:         row.get(0)?,
                    transcript_text:    row.get(1)?,
                    start_ms:           row.get(2)?,
                    end_ms:             row.get(3)?,
                    speaker_id:         row.get(4)?,
                    display_name:       row.get(5)?,
                    session_id:         row.get(6)?,
                    session_label:      row.get(7)?,
                    session_created_at: row.get(8)?,
                    score: (1.0_f32 - distance as f32).max(0.0),
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::segments::{insert_segment, insert_segment_embedding, NewSegment};
    use tempfile::tempdir;

    fn open_test_db() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let conn = db::open(&dir.path().join("test.db")).expect("open");
        (conn, dir)
    }

    fn seed_session(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO sessions (created_at, source) VALUES (1000, 'mic')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_segment(conn: &Connection, session_id: i64, text: &str) -> i64 {
        insert_segment(
            conn,
            &NewSegment {
                session_id,
                speaker_id:      1,
                start_ms:        0,
                end_ms:          1000,
                transcript_text: text.into(),
            },
        )
        .unwrap()
    }

    /// Build a 384-dim query vector that is identical to the stored vector so
    /// distance == 0 and score == 1.0.
    fn unit_vec() -> Vec<f32> {
        let mut v = vec![0.0f32; 384];
        v[0] = 1.0;
        // Normalise to unit length so cosine distance behaves predictably.
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= norm);
        v
    }

    #[test]
    fn returns_matching_segment() {
        let (conn, _dir) = open_test_db();
        let session_id = seed_session(&conn);
        let seg_id = seed_segment(&conn, session_id, "hello world");

        let vec = unit_vec();
        insert_segment_embedding(&conn, seg_id, &vec).unwrap();

        let filters = SearchFilters {
            speaker_id: None,
            start_date: None,
            end_date:   None,
            limit:      Some(10),
        };
        let results = search_segments(&conn, &vec, &filters).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].segment_id, seg_id);
        assert_eq!(results[0].transcript_text, "hello world");
        assert!(results[0].score >= 0.0);
    }

    #[test]
    fn respects_limit() {
        let (conn, _dir) = open_test_db();
        let session_id = seed_session(&conn);

        let vec = unit_vec();
        for i in 0..5 {
            let seg_id = seed_segment(&conn, session_id, &format!("segment {i}"));
            insert_segment_embedding(&conn, seg_id, &vec).unwrap();
        }

        let filters = SearchFilters {
            speaker_id: None,
            start_date: None,
            end_date:   None,
            limit:      Some(2),
        };
        let results = search_segments(&conn, &vec, &filters).expect("search");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn filters_by_speaker_id() {
        let (conn, _dir) = open_test_db();
        let session_id = seed_session(&conn);

        // Insert two segments with different speaker_ids via raw SQL so we can
        // control the speaker_id directly.
        for (speaker_id, text) in [(1i64, "speaker one"), (2i64, "speaker two")] {
            conn.execute(
                "INSERT INTO segments (session_id, speaker_id, start_ms, end_ms, transcript_text)
                 VALUES (?1, ?2, 0, 1000, ?3)",
                rusqlite::params![session_id, speaker_id, text],
            )
            .unwrap();
            let seg_id = conn.last_insert_rowid();
            let vec = unit_vec();
            insert_segment_embedding(&conn, seg_id, &vec).unwrap();
        }

        let vec = unit_vec();
        let filters = SearchFilters {
            speaker_id: Some(1),
            start_date: None,
            end_date:   None,
            limit:      Some(10),
        };
        let results = search_segments(&conn, &vec, &filters).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].speaker_id, Some(1));
    }

    #[test]
    fn empty_when_no_embeddings_stored() {
        let (conn, _dir) = open_test_db();
        let session_id = seed_session(&conn);
        seed_segment(&conn, session_id, "no embedding for this one");

        let vec = unit_vec();
        let filters = SearchFilters {
            speaker_id: None,
            start_date: None,
            end_date:   None,
            limit:      Some(10),
        };
        let results = search_segments(&conn, &vec, &filters).expect("search");
        assert!(results.is_empty());
    }
}
