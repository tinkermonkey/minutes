use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::{api::ApiState, db};

pub async fn list_speakers(
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let conn = match db::open_readonly(&state.db_path) {
        Ok(c)  => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    match db::speakers::list_with_stats(&conn) {
        Ok(s)  => Json(s).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
