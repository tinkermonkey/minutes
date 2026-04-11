use rusqlite::{Connection, OptionalExtension};

/// Read a settings value by key. Returns `None` if the key does not exist.
pub fn get(conn: &Connection, key: &str) -> anyhow::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |r| r.get(0),
    )
    .optional()
    .map_err(Into::into)
}

/// Insert or update a settings value.
pub fn set(conn: &Connection, key: &str, value: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    fn setup() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let conn = db::open(&dir.path().join("test.db")).expect("open");
        (conn, dir)
    }

    #[test]
    fn get_missing_key_returns_none() {
        let (conn, _dir) = setup();
        let val = get(&conn, "nonexistent").expect("get");
        assert!(val.is_none());
    }

    #[test]
    fn set_then_get_roundtrips() {
        let (conn, _dir) = setup();
        set(&conn, "foo", "bar").expect("set");
        let val = get(&conn, "foo").expect("get");
        assert_eq!(val.as_deref(), Some("bar"));
    }

    #[test]
    fn set_overwrites_existing_value() {
        let (conn, _dir) = setup();
        set(&conn, "key", "first").expect("set first");
        set(&conn, "key", "second").expect("set second");
        let val = get(&conn, "key").expect("get");
        assert_eq!(val.as_deref(), Some("second"));
    }
}
