use rusqlite::Connection;
use std::sync::Mutex;

/// Holds live connection-reachability state for the speech-swift audio-server.
pub struct SpeechSwiftStatus {
    pub reachable: bool,
}

/// Top-level Tauri managed state.
pub struct AppState {
    /// SQLite connection. Guarded by a mutex because rusqlite `Connection` is
    /// not `Send + Sync` on its own. All command handlers must lock this before
    /// issuing queries.
    pub db: Mutex<Connection>,
    pub speech_swift: Mutex<SpeechSwiftStatus>,
    /// Base URL for the speech-swift audio-server (e.g. "http://localhost:8080").
    pub speech_swift_url: String,
}
