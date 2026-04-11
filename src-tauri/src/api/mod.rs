use axum::{routing::get, Router};
use std::path::PathBuf;

pub mod search;
pub mod sessions;
pub mod speakers;

/// State shared across all axum handlers. Uses a `PathBuf` rather than an open
/// connection because each handler opens its own short-lived read-only
/// connection. This avoids SQLite write-read contention with the main
/// `Mutex<Connection>` in `AppState` and is safe because WAL mode allows
/// concurrent readers alongside the single writer.
#[derive(Clone)]
pub struct ApiState {
    pub db_path: PathBuf,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/sessions",               get(sessions::list_sessions))
        .route("/sessions/{id}/segments", get(sessions::get_segments))
        .route("/speakers",               get(speakers::list_speakers))
        .route("/search",                 get(search::search))
        .with_state(state)
}

/// Bind on `127.0.0.1:8765` and serve the read-only REST API.
///
/// Logs to stderr on bind failure so the app continues even if the port is
/// already in use (e.g. a second instance). Never panics.
pub async fn serve(db_path: PathBuf) {
    let state = ApiState { db_path };
    let app = router(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8765").await {
        Ok(l)  => l,
        Err(e) => {
            eprintln!("REST API failed to bind port 8765: {e}");
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("REST API server error: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    use tempfile::tempdir;
    use crate::db;

    fn make_router() -> (Router, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        db::open(&db_path).expect("open db");
        let state = ApiState { db_path };
        (router(state), dir)
    }

    #[tokio::test]
    async fn sessions_returns_ok() {
        let (app, _dir) = make_router();
        let resp = app
            .oneshot(Request::builder().uri("/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn speakers_returns_ok() {
        let (app, _dir) = make_router();
        let resp = app
            .oneshot(Request::builder().uri("/speakers").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn search_missing_q_returns_400() {
        let (app, _dir) = make_router();
        let resp = app
            .oneshot(Request::builder().uri("/search").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
