//! Silero VAD v5 backend using `tract-onnx` (pure-Rust ONNX runtime).
//!
//! Silero v5 specifics:
//! - Frame: 512 samples at 16 kHz (32 ms)
//! - Inputs:  `input` f32[1,512], `sr` i64 scalar, `h` f32[2,1,64], `c` f32[2,1,64]
//! - Outputs: `output` f32[1,1] (voiced probability), `hn` f32[2,1,64], `cn` f32[2,1,64]
//! - Voiced if output[0] > 0.5

use std::path::Path;
use std::sync::Arc;

use tract_onnx::prelude::*;

use super::vad::VadBackend;

/// Number of samples per Silero v5 frame at 16 kHz.
const FRAME_SAMPLES: usize = 512;

/// Voiced probability threshold.
const SPEECH_THRESHOLD: f32 = 0.5;

/// Hidden state dimensions: [2, 1, 64].
const H_SHAPE: [usize; 3] = [2, 1, 64];

type SileroPlan = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct SileroBackend {
    plan: Arc<SileroPlan>,
    /// LSTM hidden state h, shape [2, 1, 64].
    h: Tensor,
    /// LSTM cell state c, shape [2, 1, 64].
    c: Tensor,
}

impl SileroBackend {
    /// Load the Silero ONNX model from `model_path`.
    pub fn new(model_path: &Path) -> anyhow::Result<Self> {
        // Declare concrete input shapes so tract's ToTypedTranslator can
        // perform static analysis during optimization. Without these hints,
        // tract fails to infer shapes from the ONNX model's dynamic dims.
        let model = tract_onnx::onnx()
            .model_for_path(model_path)?
            .with_input_fact(0, f32::fact([1usize, FRAME_SAMPLES]).into())?   // input  f32[1, 512]
            .with_input_fact(1, i64::fact([] as [usize; 0]).into())?          // sr     i64 scalar
            .with_input_fact(2, f32::fact(H_SHAPE).into())?                   // h      f32[2, 1, 64]
            .with_input_fact(3, f32::fact(H_SHAPE).into())?                   // c      f32[2, 1, 64]
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            plan: Arc::new(model),
            h: Self::zero_state(),
            c: Self::zero_state(),
        })
    }

    fn zero_state() -> Tensor {
        tract_ndarray::Array3::<f32>::zeros((H_SHAPE[0], H_SHAPE[1], H_SHAPE[2])).into()
    }
}

impl VadBackend for SileroBackend {
    fn frame_size(&self) -> usize {
        FRAME_SAMPLES
    }

    fn classify_frame(&mut self, frame: &[f32]) -> bool {
        debug_assert_eq!(frame.len(), FRAME_SAMPLES);

        // Build input tensor: shape [1, 512]
        let input_arr =
            tract_ndarray::Array2::<f32>::from_shape_vec((1, FRAME_SAMPLES), frame.to_vec());
        let input_arr = match input_arr {
            Ok(a) => a,
            Err(e) => {
                eprintln!("silero: failed to build input tensor: {e}");
                return false;
            }
        };
        let input: Tensor = input_arr.into();

        // Sample rate: int64 scalar
        let sr: Tensor = tract_ndarray::arr0::<i64>(16_000).into();

        let inputs = tvec![
            input.into(),
            sr.into(),
            self.h.clone().into(),
            self.c.clone().into(),
        ];

        let mut outputs = match self.plan.run(inputs) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("silero: inference error: {e}");
                return false;
            }
        };

        // outputs[0] = voiced probability [1, 1]
        // outputs[1] = hn [2, 1, 64]
        // outputs[2] = cn [2, 1, 64]
        let prob = outputs[0]
            .as_slice::<f32>()
            .ok()
            .and_then(|s| s.first())
            .copied()
            .unwrap_or(0.0);

        // Update hidden state — consume by removing (high index first to keep indices stable).
        let cn = outputs.remove(2).into_tensor();
        let hn = outputs.remove(1).into_tensor();
        self.h = hn;
        self.c = cn;

        prob > SPEECH_THRESHOLD
    }

    fn reset(&mut self) {
        self.h = Self::zero_state();
        self.c = Self::zero_state();
    }
}

// SileroBackend is Send + Sync because tract's SimplePlan (wrapped in Arc) is
// Send + Sync and the Tensor type is also Send.  No raw pointers involved.
// SAFETY: Verify at compile time — if tract's types are not Send this will
// fail to compile.
fn _assert_send_sync()
where
    SileroBackend: Send,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the zero-state tensor has the right shape.
    #[test]
    fn zero_state_shape() {
        let t = SileroBackend::zero_state();
        assert_eq!(t.shape(), &[2usize, 1, 64]);
    }

    /// Verify frame_size is 512.
    #[test]
    fn silero_frame_size() {
        // We cannot load the real model in unit tests (no resource dir),
        // so we test via a helper that constructs the backend without a plan.
        // Instead, assert the constant directly.
        assert_eq!(FRAME_SAMPLES, 512);
    }
}
