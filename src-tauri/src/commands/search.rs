use crate::{db, embed, state::AppState};

/// Search transcript segments by semantic similarity to `query`.
///
/// Embeds `query` on a blocking thread (fastembed / ONNX is CPU-bound), then
/// runs an ANN search against the `segment_embeddings` vec0 virtual table.
/// Returns at most `filters.limit` results (default 50, hard cap 100), ordered
/// by descending similarity score.
///
/// Returns an empty list immediately when `query` is blank rather than
/// embedding an empty string, which would return arbitrary nearest-neighbour
/// results.
#[tauri::command]
pub async fn search_segments(
    query:   String,
    filters: db::search::SearchFilters,
    state:   tauri::State<'_, AppState>,
) -> Result<Vec<db::search::SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Embed on a blocking thread — fastembed is CPU-bound and must not run on
    // the async executor.
    let embedding = tokio::task::spawn_blocking(move || embed::embed(&query))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    let db = state.db.lock().expect("db mutex poisoned");
    db::search::search_segments(&db, &embedding, &filters).map_err(|e| e.to_string())
}
