use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::OnceLock;

static MODEL: OnceLock<TextEmbedding> = OnceLock::new();

/// Return a reference to the lazily-initialized embedding model.
///
/// The first call downloads and loads the model; subsequent calls return
/// immediately. This is intentionally blocking — call it from
/// `tokio::task::spawn_blocking` when warmup matters.
pub fn get_model() -> anyhow::Result<&'static TextEmbedding> {
    if let Some(m) = MODEL.get() {
        return Ok(m);
    }
    let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2))?;
    // OnceLock::get_or_try_init is nightly-only; use set + get instead.
    // A race on first init is harmless: the losing thread discards its model,
    // and both threads see the same winner via get().
    let _ = MODEL.set(model);
    Ok(MODEL.get().expect("OnceLock must be populated after set"))
}

/// Embed a single string using all-MiniLM-L6-v2 (384 dimensions).
///
/// Blocks the calling thread while the ONNX session runs. Callers on the async
/// executor should wrap this in `tokio::task::spawn_blocking`.
pub fn embed(text: &str) -> anyhow::Result<Vec<f32>> {
    let model = get_model()?;
    let mut results = model.embed(vec![text], None)?;
    if results.is_empty() {
        anyhow::bail!("fastembed returned empty embedding vector");
    }
    Ok(results.remove(0))
}
