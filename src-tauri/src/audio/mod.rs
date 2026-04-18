pub mod accumulator;
pub mod capture;
pub mod chunker;
pub mod silero;
pub mod vad;

pub use accumulator::SpeechAccumulator;

use std::path::Path;

use chunker::{Chunker, ChunkerOutput};
use silero::SileroBackend;
use vad::WebRtcBackend;

// ---------------------------------------------------------------------------
// VadMode — persisted setting (serializable for Tauri state / settings store)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum VadMode {
    /// Silero VAD v5 via tract-onnx (pure Rust).
    Silero,
    /// WebRTC VAD (C library, !Send — kept on OS capture thread).
    #[default]
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
                    eprintln!("VAD: using Silero (ort/OnnxRuntime)");
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

    pub fn push_samples(&mut self, samples: &[f32]) -> Option<ChunkerOutput> {
        match self {
            DynChunker::WebRtc(c) => c.push_samples(samples),
            DynChunker::Silero(c) => c.push_samples(samples),
        }
    }

    pub fn flush(&mut self) -> Option<ChunkerOutput> {
        match self {
            DynChunker::WebRtc(c) => c.flush(),
            DynChunker::Silero(c) => c.flush(),
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        match self {
            DynChunker::WebRtc(c) => c.reset(),
            DynChunker::Silero(c) => c.reset(),
        }
    }

    /// Whether the most recently classified frame was speech.
    ///
    /// Used by the capture OS thread to detect VAD state transitions and emit
    /// `vad_state` events to the frontend.
    pub fn is_speech(&self) -> bool {
        match self {
            DynChunker::WebRtc(c) => c.vad.last_frame_was_speech,
            DynChunker::Silero(c) => c.vad.last_frame_was_speech,
        }
    }
}
