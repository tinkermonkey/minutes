//! Silero VAD v5 backend using `ort` (OnnxRuntime Rust bindings).
//!
//! Silero v5 specifics:
//! - Frame: 512 samples at 16 kHz (32 ms)
//! - Inputs:  `input` f32[1,512], `h` f32[2,1,64], `c` f32[2,1,64]  (sr removed in v5)
//! - Outputs: `output` f32[1,1] (voiced probability), `hn` f32[2,1,64], `cn` f32[2,1,64]
//! - Voiced if output[0] > 0.5
//!
//! Why ort instead of tract-onnx: tract cannot handle the ONNX `If` operator that
//! Silero v5 uses for sample-rate branching. ort v2 supports the full ONNX spec
//! and its `Session` is `Send + Sync`.
//!
//! ## ort 2.0.0-rc.9 API notes (version locked by fastembed dependency)
//!
//! - `Session` is at `ort::session::Session`, not re-exported at the crate root.
//! - `inputs!` macro returns `Result<...>` — must propagate with `?` or match.
//! - In the named form, each value goes through `TryInto::<DynValue>` — ndarray
//!   `Array` and `ArrayView` types implement this conversion directly.
//! - `try_extract_tensor::<T>()` returns `Result<ArrayViewD<T>>` (no shape tuple).

use std::path::Path;
use std::sync::Arc;

use ndarray::{Array2, Array3, ArrayD};
use ort::session::Session;

use super::vad::VadBackend;

/// Number of samples per Silero v5 frame at 16 kHz.
const FRAME_SAMPLES: usize = 512;

/// Voiced probability threshold.
const SPEECH_THRESHOLD: f32 = 0.5;

/// Hidden-state shape [2, 1, 64] as a constant tuple for `Array3::zeros`.
const H_DIM: (usize, usize, usize) = (2, 1, 64);

pub struct SileroBackend {
    /// OnnxRuntime session. `Session` is `Send + Sync` via `SharedSessionInner`
    /// unsafe impls in ort; `Arc` lets us clone the backend cheaply if needed.
    session: Arc<Session>,
    /// LSTM hidden state h, shape [2, 1, 64].
    h: ArrayD<f32>,
    /// LSTM cell state c, shape [2, 1, 64].
    c: ArrayD<f32>,
}

impl SileroBackend {
    /// Load the Silero v5 ONNX model from `model_path`.
    pub fn new(model_path: &Path) -> anyhow::Result<Self> {
        let session = Session::builder()?.commit_from_file(model_path)?;
        Ok(Self {
            session: Arc::new(session),
            h: Self::zero_state(),
            c: Self::zero_state(),
        })
    }

    /// Returns a zeroed LSTM state array of shape [2, 1, 64].
    fn zero_state() -> ArrayD<f32> {
        Array3::<f32>::zeros(H_DIM).into_dyn()
    }
}

impl VadBackend for SileroBackend {
    fn frame_size(&self) -> usize {
        FRAME_SAMPLES
    }

    fn classify_frame(&mut self, frame: &[f32]) -> bool {
        debug_assert_eq!(frame.len(), FRAME_SAMPLES);

        // Build owned input array [1, 512].
        let input_arr =
            match Array2::<f32>::from_shape_vec((1, FRAME_SAMPLES), frame.to_vec()) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("silero: bad frame shape: {e}");
                    return false;
                }
            };

        // Build the session inputs.
        //
        // In ort 2.0.0-rc.9 the named inputs! macro returns Result<Vec<...>>,
        // where each value is converted via TryInto::<DynValue>.
        //
        // - `input_arr`   : Array2<f32>          — IntoValueTensor impl (owned, may copy if non-contiguous)
        // - `self.h.view()`: ArrayViewD<f32>      — TryFrom<ArrayView> for DynValue impl (always copies)
        // - `self.c.view()`: ArrayViewD<f32>      — same
        //
        // Both arrays were created from Array3::zeros or assigned from to_owned()
        // so they are guaranteed contiguous; the copy is cheap (2*1*64 = 128 f32).
        let run_inputs = match ort::inputs![
            "input" => input_arr,
            "h"     => self.h.view(),
            "c"     => self.c.view(),
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

        // Extract voiced probability.
        // try_extract_tensor in rc.9 returns Result<ArrayViewD<T>>.
        let prob = outputs["output"]
            .try_extract_tensor::<f32>()
            .ok()
            .as_ref()
            .and_then(|view| view.iter().next().copied())
            .unwrap_or(0.0);

        // Update LSTM hidden states.  Errors here are non-fatal — stale state
        // degrades VAD accuracy for the current utterance but does not crash
        // the pipeline.
        if let Ok(hn_view) = outputs["hn"].try_extract_tensor::<f32>() {
            self.h = hn_view.to_owned();
        } else {
            eprintln!("silero: failed to extract hn state");
        }
        if let Ok(cn_view) = outputs["cn"].try_extract_tensor::<f32>() {
            self.c = cn_view.to_owned();
        } else {
            eprintln!("silero: failed to extract cn state");
        }

        prob > SPEECH_THRESHOLD
    }

    fn reset(&mut self) {
        self.h = Self::zero_state();
        self.c = Self::zero_state();
    }
}

// Verify at compile time that SileroBackend is Send.
// `Session` is Send + Sync (via unsafe impls on SharedSessionInner in ort).
// `Arc<Session>` is Send. `ArrayD<f32>` is Send.
fn _assert_send()
where
    SileroBackend: Send,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the zero-state array has the correct shape.
    #[test]
    fn zero_state_shape() {
        let s = SileroBackend::zero_state();
        assert_eq!(s.shape(), &[2usize, 1, 64]);
    }

    /// Verify the frame size constant is 512.
    #[test]
    fn frame_size_constant() {
        assert_eq!(FRAME_SAMPLES, 512);
    }

    /// Verify zero_state produces all-zero values.
    #[test]
    fn zero_state_values() {
        let s = SileroBackend::zero_state();
        assert!(s.iter().all(|&v| v == 0.0_f32));
    }

    /// Verify that two successive zero states are identical.
    #[test]
    fn zero_state_is_idempotent() {
        let a = SileroBackend::zero_state();
        let b = SileroBackend::zero_state();
        assert_eq!(a.shape(), b.shape());
        assert!(a.iter().zip(b.iter()).all(|(x, y)| x == y));
    }
}
