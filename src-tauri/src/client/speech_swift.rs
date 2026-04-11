/// Returns `true` if the speech-swift audio-server is reachable and healthy.
///
/// A GET to `/health` returning any 2xx status is considered success. Any
/// network error or non-2xx status returns `false` — the caller decides how
/// to surface this to the UI.
pub async fn health_check(base_url: &str) -> bool {
    let url = format!("{}/health", base_url);
    reqwest::get(&url)
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_returns_false_for_unreachable_host() {
        // Port 19999 is almost certainly not bound locally; this exercises the
        // error path without needing a real server.
        let result = health_check("http://127.0.0.1:19999").await;
        assert!(!result);
    }
}
