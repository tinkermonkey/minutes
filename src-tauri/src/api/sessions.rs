use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::collections::HashMap;
use crate::{api::ApiState, db};

pub async fn list_sessions(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let conn = match db::open_readonly(&state.db_path) {
        Ok(c)  => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let filter = db::sessions::SessionFilter {
        start_date: params.get("start_date").and_then(|v| v.parse().ok()),
        end_date:   params.get("end_date").and_then(|v| v.parse().ok()),
        sort_by:    db::sessions::SortBy::Date,
        sort_dir:   db::sessions::SortDir::Desc,
        page:       params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1),
        page_size:  params.get("page_size").and_then(|v| v.parse().ok()).unwrap_or(20),
    };
    match db::sessions::list_sessions(&conn, &filter) {
        Ok(page) => Json(page).into_response(),
        Err(e)   => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_segments(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let conn = match db::open_readonly(&state.db_path) {
        Ok(c)  => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    match db::segments::get_segments_with_speakers(&conn, id) {
        Ok(segs) => Json(segs).into_response(),
        Err(e)   => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
