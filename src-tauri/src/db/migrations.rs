use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            r#"
                CREATE TABLE sessions (
                    id          INTEGER PRIMARY KEY,
                    created_at  INTEGER NOT NULL,
                    label       TEXT,
                    duration_ms INTEGER,
                    source      TEXT NOT NULL
                );

                CREATE TABLE segments (
                    id              INTEGER PRIMARY KEY,
                    session_id      INTEGER NOT NULL REFERENCES sessions(id),
                    speaker_id      INTEGER NOT NULL,
                    start_ms        INTEGER NOT NULL,
                    end_ms          INTEGER NOT NULL,
                    transcript_text TEXT NOT NULL
                );

                CREATE TABLE speaker_samples (
                    id          INTEGER PRIMARY KEY,
                    speaker_id  INTEGER NOT NULL,
                    session_id  INTEGER NOT NULL REFERENCES sessions(id),
                    start_ms    INTEGER NOT NULL,
                    end_ms      INTEGER NOT NULL,
                    audio_path  TEXT NOT NULL
                );

                CREATE TABLE speakers (
                    id              INTEGER PRIMARY KEY,
                    speech_swift_id INTEGER NOT NULL UNIQUE,
                    display_name    TEXT,
                    notes           TEXT,
                    first_seen_at   INTEGER NOT NULL,
                    last_seen_at    INTEGER NOT NULL
                );
            "#,
        ),
        M::up(
            r#"
                -- Recreate segments with nullable speaker_id (for speaker deletion support)
                CREATE TABLE segments_new (
                    id              INTEGER PRIMARY KEY,
                    session_id      INTEGER NOT NULL REFERENCES sessions(id),
                    speaker_id      INTEGER,
                    start_ms        INTEGER NOT NULL,
                    end_ms          INTEGER NOT NULL,
                    transcript_text TEXT NOT NULL
                );
                INSERT INTO segments_new SELECT * FROM segments;
                DROP TABLE segments;
                ALTER TABLE segments_new RENAME TO segments;

                -- Recreate speaker_samples with nullable speaker_id
                CREATE TABLE speaker_samples_new (
                    id          INTEGER PRIMARY KEY,
                    speaker_id  INTEGER,
                    session_id  INTEGER NOT NULL REFERENCES sessions(id),
                    start_ms    INTEGER NOT NULL,
                    end_ms      INTEGER NOT NULL,
                    audio_path  TEXT NOT NULL
                );
                INSERT INTO speaker_samples_new SELECT * FROM speaker_samples;
                DROP TABLE speaker_samples;
                ALTER TABLE speaker_samples_new RENAME TO speaker_samples;
            "#,
        ),
        M::up(
            r#"
                CREATE TABLE IF NOT EXISTS settings (
                    key   TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
            "#,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn migrations_run_to_latest() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        migrations()
            .to_latest(&mut conn)
            .expect("migrations should succeed");

        // Verify all four tables exist.
        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
                )
                .expect("prepare");
            stmt.query_map([], |row| row.get(0))
                .expect("query")
                .map(|r| r.expect("row"))
                .collect()
        };
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"segments".to_string()));
        assert!(tables.contains(&"speaker_samples".to_string()));
        assert!(tables.contains(&"speakers".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }
}
