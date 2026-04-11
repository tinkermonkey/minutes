use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot;

/// Holds live connection-reachability state for the speech-swift audio-server.
pub struct SpeechSwiftStatus {
    pub reachable: bool,
}

/// Top-level Tauri managed state.
pub struct AppState {
    /// SQLite connection. Guarded by a mutex because rusqlite `Connection` is
    /// not `Send + Sync` on its own. All command handlers must lock this before
    /// issuing queries and must release the lock before any `.await` point.
    pub db: Mutex<Connection>,
    pub speech_swift: Mutex<SpeechSwiftStatus>,
    /// Base URL for the speech-swift audio-server (e.g. "http://localhost:8080").
    pub speech_swift_url: String,
    /// Active recording pipelines keyed by session_id.
    /// Sending on the channel stops the corresponding pipeline task.
    pub pipelines: Mutex<HashMap<i64, oneshot::Sender<()>>>,
}
