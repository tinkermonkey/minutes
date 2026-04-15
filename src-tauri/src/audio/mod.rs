pub mod capture;
pub mod chunker;
pub mod silero;
pub mod vad;

use std::path::Path;

use chunker::Chunker;
use silero::SileroBackend;
use vad::WebRtcBackend;

// ---------------------------------------------------------------------------
// VadMode — persisted setting (serializable for Tauri state / settings store)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VadMode {
    /// Silero VAD v5 via tract-onnx (pure Rust, default).
    #[default]
    Silero,
    /// WebRTC VAD (C library, !Send — kept on OS capture thread).
    WebRtc,
}

// ---------------------------------------------------------------------------
// DynChunker — type-erased wrapper so commands/mod.rs has one concrete type
// ---------------------------------------------------------------------------

/// Unifies the two chunker variants behind a single concrete enum so the
/// capture OS thread does not need to be generic.
pub enum DynChunker {
    WebRtc(Chunker<WebRtcBackend>),
    Silero(Chunker<SileroBackend>),
}

impl DynChunker {
    /// Build a `DynChunker` according to `mode`.
    ///
    /// `model_path` is only consulted for `VadMode::Silero`.  If the Silero
    /// model file is missing or fails to load, the function falls back to
    /// `WebRtc` with a warning log.
    pub fn new(mode: VadMode, model_path: &Path) -> Self {
        match mode {
            VadMode::Silero => match SileroBackend::new(model_path) {
                Ok(backend) => {
                    eprintln!("VAD: using Silero (tract-onnx)");
                    DynChunker::Silero(Chunker::new(backend))
                }
                Err(e) => {
                    eprintln!(
                        "VAD: Silero model failed to load ({e}), falling back to WebRTC VAD"
                    );
                    DynChunker::WebRtc(Chunker::new(WebRtcBackend::new()))
                }
            },
            VadMode::WebRtc => {
                eprintln!("VAD: using WebRTC VAD");
                DynChunker::WebRtc(Chunker::new(WebRtcBackend::new()))
            }
        }
    }

    pub fn push_samples(&mut self, samples: &[f32]) -> Option<(Vec<u8>, u64, u64)> {
        match self {
            DynChunker::WebRtc(c) => c.push_samples(samples),
            DynChunker::Silero(c) => c.push_samples(samples),
        }
    }

    pub fn flush(&mut self) -> Option<(Vec<u8>, u64, u64)> {
        match self {
            DynChunker::WebRtc(c) => c.flush(),
            DynChunker::Silero(c) => c.flush(),
        }
    }

    pub fn reset(&mut self) {
        match self {
            DynChunker::WebRtc(c) => c.reset(),
            DynChunker::Silero(c) => c.reset(),
        }
    }
}
