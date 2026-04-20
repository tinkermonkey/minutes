use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, serde::Serialize)]
pub struct SessionParticipant {
    pub speech_swift_id: i64,
    pub display_name:    Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SessionRow {
    pub id:           i64,
    pub created_at:   i64,
    pub label:        Option<String>,
    pub duration_ms:  Option<i64>,
    pub source:       String,
    pub participants: Vec<SessionParticipant>,
}

#[derive(Debug, serde::Serialize)]
pub struct SessionsPage {
    pub sessions:    Vec<SessionRow>,
    pub total_count: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct SessionFilter {
    pub start_date: Option<i64>,
    pub end_date:   Option<i64>,
    pub sort_by:    SortBy,
    pub sort_dir:   SortDir,
    pub page:       u32,
    pub page_size:  u32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy { Date, Duration }

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDir { Asc, Desc }

fn get_participants(conn: &Connection, session_id: i64) -> anyhow::Result<Vec<SessionParticipant>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT sg.speaker_id, sp.display_name
         FROM segments sg
         LEFT JOIN speakers sp ON sp.speech_swift_id = sg.speaker_id
         WHERE sg.session_id = ?1 AND sg.speaker_id IS NOT NULL",
    )?;
    let rows = stmt
        .query_map([session_id], |row| {
            Ok(SessionParticipant {
                speech_swift_id: row.get(0)?,
                display_name:    row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_sessions(
    conn: &Connection,
    filter: &SessionFilter,
) -> anyhow::Result<SessionsPage> {
    let order = match (&filter.sort_by, &filter.sort_dir) {
        (SortBy::Date,     SortDir::Desc) => "created_at DESC",
        (SortBy::Date,     SortDir::Asc)  => "created_at ASC",
        (SortBy::Duration, SortDir::Desc) => "duration_ms DESC",
        (SortBy::Duration, SortDir::Asc)  => "duration_ms ASC",
    };
    let offset = (filter.page.saturating_sub(1) as i64) * (filter.page_size as i64);

    // Nullable-param trick: always pass both date params; SQLite short-circuits
    // NULL checks so the filter only applies when the value is non-NULL.
    let start_date = filter.start_date;
    let end_date   = filter.end_date;

    let count_sql = format!(
        "SELECT COUNT(*) FROM sessions
         WHERE (?1 IS NULL OR created_at >= ?1)
           AND (?2 IS NULL OR created_at <= ?2)"
    );
    let total_count: i64 = conn.query_row(
        &count_sql,
        rusqlite::params![start_date, end_date],
        |r| r.get(0),
    )?;

    let data_sql = format!(
        "SELECT id, created_at, label, duration_ms, source
         FROM sessions
         WHERE (?1 IS NULL OR created_at >= ?1)
           AND (?2 IS NULL OR created_at <= ?2)
         ORDER BY {order}
         LIMIT ?3 OFFSET ?4"
    );

    let mut stmt = conn.prepare(&data_sql)?;
    let sessions_raw = stmt
        .query_map(
            rusqlite::params![start_date, end_date, filter.page_size as i64, offset],
            |row| {
                Ok(SessionRow {
                    id:           row.get(0)?,
                    created_at:   row.get(1)?,
                    label:        row.get(2)?,
                    duration_ms:  row.get(3)?,
                    source:       row.get(4)?,
                    participants: Vec::new(),
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let mut sessions = sessions_raw;
    for session in &mut sessions {
        session.participants = get_participants(conn, session.id)?;
    }

    Ok(SessionsPage { sessions, total_count })
}

/// Update the label for an existing session.
pub fn update_session_label(
    conn: &Connection,
    session_id: i64,
    label: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE sessions SET label = ?1 WHERE id = ?2",
        rusqlite::params![label, session_id],
    )?;
    Ok(())
}

/// Delete all sessions and their associated data from the database.
///
/// Deletes in dependency order: speaker_samples → segments → sessions.
/// Speaker identities in the `speakers` table are intentionally left intact —
/// the user can reset those separately via reset_speaker_registry.
pub fn delete_all(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "DELETE FROM speaker_samples;
         DELETE FROM segments;
         DELETE FROM sessions;",
    )?;
    Ok(())
}

pub fn get_session_by_id(
    conn: &Connection,
    session_id: i64,
) -> anyhow::Result<Option<SessionRow>> {
    let result = conn.query_row(
        "SELECT id, created_at, label, duration_ms, source FROM sessions WHERE id = ?1",
        [session_id],
        |row| {
            Ok(SessionRow {
                id:           row.get(0)?,
                created_at:   row.get(1)?,
                label:        row.get(2)?,
                duration_ms:  row.get(3)?,
                source:       row.get(4)?,
                participants: Vec::new(),
            })
        },
    )
    .optional()?;

    if let Some(mut session) = result {
        session.participants = get_participants(conn, session.id)?;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    fn open_test_db() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let conn = db::open(&dir.path().join("test.db")).expect("open");
        (conn, dir)
    }

    fn insert_session(conn: &Connection, created_at: i64, duration_ms: Option<i64>) -> i64 {
        conn.execute(
            "INSERT INTO sessions (created_at, source, duration_ms) VALUES (?1, 'mic', ?2)",
            rusqlite::params![created_at, duration_ms],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn default_filter() -> SessionFilter {
        SessionFilter {
            start_date: None,
            end_date:   None,
            sort_by:    SortBy::Date,
            sort_dir:   SortDir::Desc,
            page:       1,
            page_size:  20,
        }
    }

    #[test]
    fn list_sessions_empty() {
        let (conn, _dir) = open_test_db();
        let page = list_sessions(&conn, &default_filter()).expect("list");
        assert_eq!(page.total_count, 0);
        assert!(page.sessions.is_empty());
    }

    #[test]
    fn list_sessions_returns_rows() {
        let (conn, _dir) = open_test_db();
        insert_session(&conn, 1000, Some(5000));
        insert_session(&conn, 2000, Some(3000));

        let page = list_sessions(&conn, &default_filter()).expect("list");
        assert_eq!(page.total_count, 2);
        assert_eq!(page.sessions.len(), 2);
        // Default sort is date DESC — most recent first.
        assert_eq!(page.sessions[0].created_at, 2000);
    }

    #[test]
    fn list_sessions_date_filter() {
        let (conn, _dir) = open_test_db();
        insert_session(&conn, 1000, None);
        insert_session(&conn, 5000, None);

        let filter = SessionFilter {
            start_date: Some(2000),
            end_date:   None,
            ..default_filter()
        };
        let page = list_sessions(&conn, &filter).expect("list");
        assert_eq!(page.total_count, 1);
        assert_eq!(page.sessions[0].created_at, 5000);
    }

    #[test]
    fn list_sessions_pagination() {
        let (conn, _dir) = open_test_db();
        for i in 0..5 {
            insert_session(&conn, i * 1000, None);
        }

        let filter = SessionFilter {
            page:      1,
            page_size: 2,
            sort_by:   SortBy::Date,
            sort_dir:  SortDir::Asc,
            ..default_filter()
        };
        let page = list_sessions(&conn, &filter).expect("list");
        assert_eq!(page.total_count, 5);
        assert_eq!(page.sessions.len(), 2);
        assert_eq!(page.sessions[0].created_at, 0);

        let page2 = list_sessions(
            &conn,
            &SessionFilter { page: 2, ..filter },
        )
        .expect("list page 2");
        assert_eq!(page2.sessions.len(), 2);
        assert_eq!(page2.sessions[0].created_at, 2000);
    }

    #[test]
    fn get_session_by_id_not_found() {
        let (conn, _dir) = open_test_db();
        let result = get_session_by_id(&conn, 999).expect("query");
        assert!(result.is_none());
    }

    #[test]
    fn get_session_by_id_found() {
        let (conn, _dir) = open_test_db();
        let id = insert_session(&conn, 3000, Some(10000));
        let session = get_session_by_id(&conn, id)
            .expect("query")
            .expect("should exist");
        assert_eq!(session.id, id);
        assert_eq!(session.created_at, 3000);
        assert_eq!(session.duration_ms, Some(10000));
        assert_eq!(session.source, "mic");
    }

    #[test]
    fn get_session_participants_populated() {
        let (conn, _dir) = open_test_db();
        let session_id = insert_session(&conn, 1000, None);
        conn.execute(
            "INSERT INTO segments (session_id, speaker_id, start_ms, end_ms, transcript_text)
             VALUES (?1, 7, 0, 500, 'hello')",
            [session_id],
        )
        .unwrap();

        let session = get_session_by_id(&conn, session_id)
            .expect("query")
            .expect("should exist");
        assert_eq!(session.participants.len(), 1);
        assert_eq!(session.participants[0].speech_swift_id, 7);
    }
}
