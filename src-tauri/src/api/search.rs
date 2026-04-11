use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::collections::HashMap;
use crate::{api::ApiState, db, embed};

pub async fn search(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let q = match params.get("q") {
        Some(q) if !q.is_empty() => q.clone(),
        _ => return (StatusCode::BAD_REQUEST, "missing q parameter").into_response(),
    };

    let filters = db::search::SearchFilters {
        speaker_id: params.get("speaker_id").and_then(|v| v.parse().ok()),
        start_date: params.get("start").and_then(|v| v.parse().ok()),
        end_date:   params.get("end").and_then(|v| v.parse().ok()),
        limit:      None,
    };

    let embedding = match embed::embed(&q) {
        Ok(e)  => e,
        Err(e) => return (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response(),
    };

    let conn = match db::open_readonly(&state.db_path) {
        Ok(c)  => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match db::search::search_segments(&conn, &embedding, &filters) {
        Ok(results) => Json(results).into_response(),
        Err(e)      => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
