use super::vad::VadClassifier;

const FRAME_SAMPLES: usize = 160; // 10 ms at 16 kHz

/// Accepts raw f32 samples from the CPAL callback, feeds 10 ms frames to the
/// VAD, and emits WAV-encoded buffers when the VAD decides to flush.
pub struct Chunker {
    vad: VadClassifier,
    /// Partial frame accumulator — holds leftover samples between `push_samples`
    /// calls.
    frame_buf: Vec<f32>,
    /// Sample index at which the current voiced chunk began (used for timing).
    chunk_start_samples: u64,
    /// Total samples consumed so far in this session.
    total_samples: u64,
}

impl Chunker {
    pub fn new() -> Self {
        Self {
            vad: VadClassifier::new(),
            frame_buf: Vec::new(),
            chunk_start_samples: 0,
            total_samples: 0,
        }
    }

    /// Push a batch of samples received from CPAL.
    ///
    /// Returns `Some((wav_bytes, start_ms, end_ms))` when the VAD emits a
    /// chunk, `None` otherwise.
    ///
    /// Only one chunk per call is returned. In practice CPAL batches are small
    /// (~10 ms) so at most one VAD flush happens per call.
    pub fn push_samples(&mut self, samples: &[f32]) -> Option<(Vec<u8>, u64, u64)> {
        self.frame_buf.extend_from_slice(samples);

        while self.frame_buf.len() >= FRAME_SAMPLES {
            let frame: Vec<f32> = self.frame_buf.drain(..FRAME_SAMPLES).collect();

            // Record position before advancing the counter so we can mark where
            // voiced content started.
            let samples_before = self.total_samples;
            self.total_samples += FRAME_SAMPLES as u64;

            // If this is the very first voiced frame, anchor the start time.
            if self.vad.voiced_buf.is_empty() {
                self.chunk_start_samples = samples_before;
            }

            if let Some(chunk) = self.vad.push_frame(&frame) {
                let start_ms = samples_to_ms(self.chunk_start_samples);
                let end_ms = samples_to_ms(self.total_samples);
                self.chunk_start_samples = self.total_samples;
                return Some((encode_wav(&chunk), start_ms, end_ms));
            }
        }

        None
    }

    /// Flush remaining voiced content at session end.
    pub fn flush(&mut self) -> Option<(Vec<u8>, u64, u64)> {
        if let Some(chunk) = self.vad.flush() {
            let start_ms = samples_to_ms(self.chunk_start_samples);
            let end_ms = samples_to_ms(self.total_samples);
            return Some((encode_wav(&chunk), start_ms, end_ms));
        }
        None
    }
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new()
    }
}

fn samples_to_ms(samples: u64) -> u64 {
    samples * 1_000 / 16_000
}

/// Encode f32 samples as 16-bit PCM WAV at 16 kHz mono.
///
/// Uses `expect` on the hound writer operations because writing to an in-memory
/// Vec cannot fail for the operations performed here (no I/O, no disk full).
fn encode_wav(samples: &[f32]) -> Vec<u8> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::io::Cursor;

    let spec = WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut writer =
            WavWriter::new(cursor, spec).expect("WAV writer creation on Vec cannot fail");
        for &s in samples {
            let i = (s * 32_767.0).clamp(-32_768.0, 32_767.0) as i16;
            writer
                .write_sample(i)
                .expect("WAV write_sample to Vec cannot fail");
        }
        writer.finalize().expect("WAV finalize to Vec cannot fail");
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_wav_produces_valid_header() {
        let samples: Vec<f32> = vec![0.0; 160];
        let wav = encode_wav(&samples);
        // RIFF header starts with "RIFF"
        assert_eq!(&wav[0..4], b"RIFF");
        // WAV format marker at offset 8
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn samples_to_ms_converts_correctly() {
        assert_eq!(samples_to_ms(0), 0);
        assert_eq!(samples_to_ms(16_000), 1_000);
        assert_eq!(samples_to_ms(8_000), 500);
    }

    #[test]
    fn flush_on_empty_chunker_returns_none() {
        let mut chunker = Chunker::new();
        assert!(chunker.flush().is_none());
    }

    #[test]
    fn push_samples_accumulates_partial_frames() {
        let mut chunker = Chunker::new();
        // 80 samples — not enough for a full 160-sample frame.
        let result = chunker.push_samples(&vec![0.0f32; 80]);
        assert!(result.is_none());
        assert_eq!(chunker.frame_buf.len(), 80);
    }
}
