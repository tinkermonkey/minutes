use super::vad::{VadBackend, VadClassifier};

/// Accepts raw f32 samples from the CPAL callback, feeds frames to the VAD,
/// and emits WAV-encoded buffers when the VAD decides to flush.
///
/// The generic parameter `B` is the VAD backend.  The frame size is queried
/// from the backend at runtime so `Chunker<WebRtcBackend>` and
/// `Chunker<SileroBackend>` both work without change.
pub struct Chunker<B: VadBackend> {
    vad: VadClassifier<B>,
    /// Partial frame accumulator — holds leftover samples between
    /// `push_samples` calls.
    frame_buf: Vec<f32>,
    /// Sample index at which the current voiced chunk began (used for timing).
    chunk_start_samples: u64,
    /// Total samples consumed so far in this session.
    total_samples: u64,
}

impl<B: VadBackend> Chunker<B> {
    pub fn new(backend: B) -> Self {
        Self {
            vad: VadClassifier::new(backend),
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
    /// Only one chunk per call is returned.  In practice CPAL batches are
    /// small (~10 ms) so at most one VAD flush happens per call.
    pub fn push_samples(&mut self, samples: &[f32]) -> Option<(Vec<u8>, u64, u64)> {
        let frame_size = self.vad.frame_size();
        self.frame_buf.extend_from_slice(samples);

        while self.frame_buf.len() >= frame_size {
            let frame: Vec<f32> = self.frame_buf.drain(..frame_size).collect();

            let samples_before = self.total_samples;
            self.total_samples += frame_size as u64;

            // Anchor the start time at the first voiced frame.
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

    /// Reset all per-session state: VAD backend hidden state, frame buffer, and
    /// sample counters.  Called between sessions so a new recording starts clean.
    pub fn reset(&mut self) {
        self.vad.reset();
        self.frame_buf.clear();
        self.chunk_start_samples = 0;
        self.total_samples = 0;
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

fn samples_to_ms(samples: u64) -> u64 {
    samples * 1_000 / 16_000
}

/// Encode f32 samples as 16-bit PCM WAV at 16 kHz mono.
///
/// Uses `expect` on hound operations because writing to an in-memory Vec
/// cannot fail for the operations performed here (no I/O, no disk full).
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
    use crate::audio::vad::WebRtcBackend;

    // Helper: Chunker backed by WebRTC VAD (same as before the refactor).
    fn webrtc_chunker() -> Chunker<WebRtcBackend> {
        Chunker::new(WebRtcBackend::new())
    }

    #[test]
    fn encode_wav_produces_valid_header() {
        let samples: Vec<f32> = vec![0.0; 160];
        let wav = encode_wav(&samples);
        assert_eq!(&wav[0..4], b"RIFF");
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
        assert!(webrtc_chunker().flush().is_none());
    }

    #[test]
    fn push_samples_accumulates_partial_frames() {
        let mut chunker = webrtc_chunker();
        // 80 samples — not enough for a full 160-sample frame.
        let result = chunker.push_samples(&vec![0.0f32; 80]);
        assert!(result.is_none());
        assert_eq!(chunker.frame_buf.len(), 80);
    }

}
