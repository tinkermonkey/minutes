//! Silero VAD backend using `ort` (OnnxRuntime Rust bindings).
//!
//! Model introspection (resources/silero_vad.onnx):
//!   Inputs:  `input` f32[None,None], `state` f32[2,None,128], `sr` i64[]
//!   Outputs: `output` f32[None,1], `stateN` f32[None,None,None]
//!
//! Usage:
//!   - input shape: [1, 512] at 16 kHz (512 samples = 32 ms per frame)
//!   - state shape: [2, 1, 128] (single-batch LSTM state)
//!   - sr: scalar 16000
//!   - Voiced if output[0] > 0.5
//!
//! Why ort instead of tract-onnx: tract cannot handle the ONNX `If` operator.
//! ort v2 supports the full ONNX spec and its `Session` is `Send + Sync`.
//!
//! ## ort 2.0.0-rc.9 API notes (version locked by fastembed dependency)
//!
//! - `Session` is at `ort::session::Session`.
//! - `inputs!` macro returns `Result<...>` — must propagate with `?`.
//! - `try_extract_tensor::<T>()` returns `Result<ArrayViewD<T>>`.

use std::path::Path;
use std::sync::Arc;

use ndarray::{Array1, Array2, Array3, ArrayD};
use ort::session::Session;

use super::vad::VadBackend;

/// Number of samples per frame at 16 kHz (32 ms).
const FRAME_SAMPLES: usize = 512;

/// Sample rate sent to the model.
const SAMPLE_RATE: i64 = 16000;

/// Voiced probability threshold.
const SPEECH_THRESHOLD: f32 = 0.4;

/// LSTM state shape [2, 1, 128].
const STATE_DIM: (usize, usize, usize) = (2, 1, 128);

pub struct SileroBackend {
    /// OnnxRuntime session. `Session` is `Send + Sync` via `SharedSessionInner`
    /// unsafe impls in ort; `Arc` lets us clone the backend cheaply if needed.
    session: Arc<Session>,
    /// Combined LSTM state, shape [2, 1, 128].
    state: ArrayD<f32>,
}

impl SileroBackend {
    /// Load the Silero ONNX model from `model_path`.
    pub fn new(model_path: &Path) -> anyhow::Result<Self> {
        let session = Session::builder()?.commit_from_file(model_path)?;
        Ok(Self {
            session: Arc::new(session),
            state: Self::zero_state(),
        })
    }

    /// Returns a zeroed LSTM state array of shape [2, 1, 128].
    fn zero_state() -> ArrayD<f32> {
        Array3::<f32>::zeros(STATE_DIM).into_dyn()
    }
}

impl VadBackend for SileroBackend {
    fn frame_size(&self) -> usize {
        FRAME_SAMPLES
    }

    fn classify_frame(&mut self, frame: &[f32]) -> bool {
        debug_assert_eq!(frame.len(), FRAME_SAMPLES);

        // Build input [1, 512].
        let input_arr =
            match Array2::<f32>::from_shape_vec((1, FRAME_SAMPLES), frame.to_vec()) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("silero: bad frame shape: {e}");
                    return false;
                }
            };

        // Scalar sample rate as a 1-D array with one element (shape [1]).
        // ort's ndarray feature converts Array1 → DynValue; a true scalar
        // Array0 hits a different code path that may not be supported.
        let sr_arr = Array1::<i64>::from_vec(vec![SAMPLE_RATE]);

        let run_inputs = match ort::inputs![
            "input" => input_arr,
            "state" => self.state.view(),
            "sr"    => sr_arr,
        ] {
            Ok(i) => i,
            Err(e) => {
                eprintln!("silero: failed to build inputs: {e}");
                return false;
            }
        };

        let outputs = match self.session.run(run_inputs) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("silero: inference error: {e}");
                return false;
            }
        };

        // Extract voiced probability (first element of output [None, 1]).
        let prob = outputs["output"]
            .try_extract_tensor::<f32>()
            .ok()
            .as_ref()
            .and_then(|view| view.iter().next().copied())
            .unwrap_or(0.0);

        // Update LSTM state from stateN.
        if let Ok(state_view) = outputs["stateN"].try_extract_tensor::<f32>() {
            self.state = state_view.to_owned();
        } else {
            eprintln!("silero: failed to extract stateN");
        }

        prob > SPEECH_THRESHOLD
    }

    fn reset(&mut self) {
        self.state = Self::zero_state();
    }
}

// Compile-time Send check.
fn _assert_send()
where
    SileroBackend: Send,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_state_shape() {
        let s = SileroBackend::zero_state();
        assert_eq!(s.shape(), &[2usize, 1, 128]);
    }

    #[test]
    fn frame_size_constant() {
        assert_eq!(FRAME_SAMPLES, 512);
    }

    #[test]
    fn zero_state_values() {
        let s = SileroBackend::zero_state();
        assert!(s.iter().all(|&v| v == 0.0_f32));
    }
}
