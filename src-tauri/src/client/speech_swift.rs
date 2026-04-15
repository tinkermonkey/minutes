/// Segment returned by the audio-server for a single speaker turn.
///
/// `start` and `end` are in **seconds** (float). Convert to ms with
/// `(start * 1000.0) as i64` at the call site.
#[derive(Debug, serde::Deserialize)]
pub struct SegmentResponse {
    pub speaker_id:    Option<i64>,
    pub speaker_label: Option<String>,
    /// Segment start time in seconds from the beginning of the submitted audio.
    pub start:         f64,
    /// Segment end time in seconds.
    pub end:           f64,
    #[allow(dead_code)]
    pub duration:      f64,
    pub transcript:    Option<String>,
}

/// Top-level response from `POST /registry/sessions`.
#[derive(Debug, serde::Deserialize)]
pub struct SessionResponse {
    #[allow(dead_code)]
    pub num_speakers: u32,
    pub segments:     Vec<SegmentResponse>,
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
    let form = reqwest::multipart::Form::new().part("file", part);

    let body = client
        .post(format!("{}/registry/sessions", base_url))
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    serde_json::from_str::<SessionResponse>(&body).map_err(|e| {
        let preview: String = body.chars().take(1000).collect();
        anyhow::anyhow!("transcribe parse error: {e}\nbody: {preview}")
    })
}

/// A speaker record returned by `GET /registry/speakers`.
#[derive(Debug, serde::Deserialize)]
pub struct SpeakerRecord {
    pub id:           i64,
    pub display_name: Option<String>,
    #[allow(dead_code)]
    pub notes:        Option<String>,
}

/// Fetch the full speaker registry from the audio-server.
pub async fn list_speakers(base_url: &str) -> anyhow::Result<Vec<SpeakerRecord>> {
    let body = reqwest::get(format!("{}/registry/speakers", base_url))
        .await?
        .error_for_status()?
        .text()
        .await?;

    #[derive(serde::Deserialize)]
    struct Wrapper { speakers: Vec<SpeakerRecord> }

    serde_json::from_str::<Wrapper>(&body)
        .map(|w| w.speakers)
        .map_err(|e| {
            let preview: String = body.chars().take(1000).collect();
            anyhow::anyhow!("list_speakers parse error: {e}\nbody: {preview}")
        })
}

/// Set the display name for a speaker in the audio-server registry.
pub async fn rename_speaker(base_url: &str, speech_swift_id: i64, name: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    client
        .patch(format!("{}/registry/speakers/{}", base_url, speech_swift_id))
        .json(&serde_json::json!({ "displayName": name }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Merge `src_id` into `dst_id` in the audio-server registry.
///
/// After this call, speech-swift treats all occurrences of `src` as `dst`.
pub async fn merge_speakers(base_url: &str, src_id: i64, dst_id: i64) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    client
        .post(format!("{}/registry/speakers/merge", base_url))
        .json(&serde_json::json!({ "src": src_id, "dst": dst_id }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Delete a speaker from the audio-server registry.
pub async fn delete_speaker(base_url: &str, speech_swift_id: i64) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    client
        .delete(format!("{}/registry/speakers/{}", base_url, speech_swift_id))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Wipe all speakers and centroids from the audio-server registry.
///
/// Calls `DELETE /registry/speakers` which resets the registry to empty.
pub async fn reset_registry(base_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    client
        .delete(format!("{}/registry/speakers", base_url))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
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
