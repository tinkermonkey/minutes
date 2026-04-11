/// Segment returned by the audio-server for a single speaker turn.
#[derive(Debug, serde::Deserialize)]
pub struct SegmentResponse {
    pub speaker_id:    i64,
    pub speaker_label: String,
    pub start_ms:      i64,
    pub end_ms:        i64,
    pub transcript:    String,
}

/// Top-level response from `POST /registry/sessions`.
#[derive(Debug, serde::Deserialize)]
pub struct SessionResponse {
    pub segments: Vec<SegmentResponse>,
}

/// Submit a WAV chunk to the audio-server for diarization + transcription.
///
/// The bytes are posted as a multipart form field named `audio` with MIME type
/// `audio/wav`. Returns the parsed segment list on success.
pub async fn transcribe_chunk(
    base_url: &str,
    wav_bytes: Vec<u8>,
) -> anyhow::Result<SessionResponse> {
    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(wav_bytes)
        .file_name("chunk.wav")
        .mime_str("audio/wav")?;
    let form = reqwest::multipart::Form::new().part("audio", part);

    let resp = client
        .post(format!("{}/registry/sessions", base_url))
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .json::<SessionResponse>()
        .await?;

    Ok(resp)
}

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
