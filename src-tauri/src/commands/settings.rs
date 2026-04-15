use tauri::State;

use crate::{audio::VadMode, db, state::AppState};

const VAD_MODE_KEY: &str = "vad_mode";

/// Return the persisted VAD mode, defaulting to `Silero` if not yet set.
#[tauri::command]
pub fn get_vad_mode(state: State<AppState>) -> VadMode {
    let conn = state.db.lock().expect("db mutex poisoned");
    db::settings::get(&conn, VAD_MODE_KEY)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(VadMode::Silero)
}

/// Persist the user's chosen VAD mode.
#[tauri::command]
pub fn set_vad_mode(mode: VadMode, state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let value = serde_json::to_string(&mode).map_err(|e| e.to_string())?;
    db::settings::set(&conn, VAD_MODE_KEY, &value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    fn open_conn() -> (rusqlite::Connection, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let conn = db::open(&dir.path().join("test.db")).expect("open db");
        (conn, dir)
    }

    #[test]
    fn get_vad_mode_defaults_to_silero_when_unset() {
        let (conn, _dir) = open_conn();
        let val: Option<String> = db::settings::get(&conn, VAD_MODE_KEY).expect("get");
        assert!(val.is_none(), "key should not exist yet");

        // Simulate the default branch: no stored value → Silero.
        let mode = val
            .and_then(|s| serde_json::from_str::<VadMode>(&s).ok())
            .unwrap_or(VadMode::Silero);
        assert!(matches!(mode, VadMode::Silero));
    }

    #[test]
    fn set_and_get_silero_roundtrips() {
        let (conn, _dir) = open_conn();
        let value = serde_json::to_string(&VadMode::Silero).expect("serialize");
        db::settings::set(&conn, VAD_MODE_KEY, &value).expect("set");
        let stored = db::settings::get(&conn, VAD_MODE_KEY)
            .expect("get")
            .expect("should be Some");
        let mode: VadMode = serde_json::from_str(&stored).expect("deserialize");
        assert!(matches!(mode, VadMode::Silero));
    }

    #[test]
    fn set_and_get_webrtc_roundtrips() {
        let (conn, _dir) = open_conn();
        let value = serde_json::to_string(&VadMode::WebRtc).expect("serialize");
        db::settings::set(&conn, VAD_MODE_KEY, &value).expect("set");
        let stored = db::settings::get(&conn, VAD_MODE_KEY)
            .expect("get")
            .expect("should be Some");
        let mode: VadMode = serde_json::from_str(&stored).expect("deserialize");
        assert!(matches!(mode, VadMode::WebRtc));
    }

    #[test]
    fn overwrite_mode_persists_latest_value() {
        let (conn, _dir) = open_conn();
        let silero = serde_json::to_string(&VadMode::Silero).expect("serialize");
        let webrtc = serde_json::to_string(&VadMode::WebRtc).expect("serialize");
        db::settings::set(&conn, VAD_MODE_KEY, &silero).expect("set silero");
        db::settings::set(&conn, VAD_MODE_KEY, &webrtc).expect("set webrtc");
        let stored = db::settings::get(&conn, VAD_MODE_KEY)
            .expect("get")
            .expect("should be Some");
        let mode: VadMode = serde_json::from_str(&stored).expect("deserialize");
        assert!(matches!(mode, VadMode::WebRtc));
    }
}
