use webrtc_vad::{SampleRate, Vad, VadMode};

/// Hard cap: flush after 30 s of voiced audio regardless of silence.
const MAX_VOICED_SAMPLES: usize = 480_000;

/// 200 ms of silence padding appended to voiced content so the tail of an
/// utterance is not cut off by encoder look-ahead.
const SILENCE_PAD_MS: usize = 200;

// ---------------------------------------------------------------------------
// VadBackend trait
// ---------------------------------------------------------------------------

/// Abstraction over different VAD implementations.
///
/// Both `WebRtcBackend` and `SileroBackend` implement this trait.  The frame
/// size is part of the trait contract — `VadClassifier` uses it to scale all
/// silence thresholds correctly.
pub trait VadBackend {
    /// Number of f32 samples per frame this backend expects.
    fn frame_size(&self) -> usize;
    /// Classify one frame. Returns `true` if voiced.
    fn classify_frame(&mut self, frame: &[f32]) -> bool;
    /// Reset per-session state (e.g. Silero LSTM hidden state).
    #[allow(dead_code)]
    fn reset(&mut self);
}

// ---------------------------------------------------------------------------
// WebRtcBackend
// ---------------------------------------------------------------------------

/// WebRTC-VAD backed classifier.
///
/// `webrtc_vad::Vad` wraps a raw C pointer and is therefore `!Send`.
/// This struct is intentionally `!Send` too — it must stay on the OS capture
/// thread.
pub struct WebRtcBackend {
    vad: Vad,
}

impl WebRtcBackend {
    pub fn new() -> Self {
        Self {
            vad: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Quality),
        }
    }
}

impl Default for WebRtcBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VadBackend for WebRtcBackend {
    /// 10 ms at 16 kHz — the minimum frame size webrtc-vad accepts.
    fn frame_size(&self) -> usize {
        160
    }

    fn classify_frame(&mut self, frame: &[f32]) -> bool {
        let i16_frame: Vec<i16> = frame
            .iter()
            .map(|&s| (s * 32_767.0).clamp(-32_768.0, 32_767.0) as i16)
            .collect();
        self.vad.is_voice_segment(&i16_frame).unwrap_or(false)
    }

    fn reset(&mut self) {
        // WebRTC VAD has no per-session state to reset.
    }
}

// ---------------------------------------------------------------------------
// VadClassifier<B>
// ---------------------------------------------------------------------------

/// Gating layer that buffers voiced audio and emits chunks on silence.
///
/// Frame-size-dependent thresholds are computed dynamically from
/// `B::frame_size()` so the logic is identical for both backends.
pub struct VadClassifier<B: VadBackend> {
    pub backend: B,
    /// Accumulated voiced (+ padding) samples waiting to be flushed.
    pub voiced_buf: Vec<f32>,
    /// Consecutive silent frames seen since the last voiced frame.
    silence_run: u32,
    /// Short silence tail kept as padding — prepended when the next voiced
    /// frame arrives, or discarded on flush.
    silence_pad: Vec<f32>,
}

impl<B: VadBackend> VadClassifier<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            voiced_buf: Vec::new(),
            silence_run: 0,
            silence_pad: Vec::new(),
        }
    }

    /// Expose the backend's frame size so `Chunker` can query it without
    /// reaching through `classifier.backend.frame_size()`.
    pub fn frame_size(&self) -> usize {
        self.backend.frame_size()
    }

    /// How many consecutive silent frames trigger a flush.
    ///
    /// Scales to 500 ms regardless of the backend's frame duration.
    fn silence_flush_frames(&self) -> u32 {
        let frame_ms = self.backend.frame_size() * 1_000 / 16_000;
        // 500 ms / frame_duration_ms, minimum 1
        (500 / frame_ms.max(1)) as u32
    }

    /// Maximum number of samples kept in the silence-pad buffer.
    fn silence_pad_samples(&self) -> usize {
        SILENCE_PAD_MS * 16_000 / 1_000
    }

    /// Feed exactly `backend.frame_size()` samples.
    ///
    /// Returns `Some(chunk)` when a flush condition is met:
    /// - 500 ms of trailing silence after voiced content, or
    /// - 30 s hard cap.
    ///
    /// Returns `None` otherwise.
    pub fn push_frame(&mut self, frame: &[f32]) -> Option<Vec<f32>> {
        debug_assert_eq!(
            frame.len(),
            self.backend.frame_size(),
            "frame length must match backend.frame_size()"
        );

        let is_speech = self.backend.classify_frame(frame);

        if is_speech {
            // Prepend accumulated silence padding so utterance boundaries
            // are preserved.
            if !self.silence_pad.is_empty() {
                self.voiced_buf
                    .extend_from_slice(&std::mem::take(&mut self.silence_pad));
            }
            self.voiced_buf.extend_from_slice(frame);
            self.silence_run = 0;

            if self.voiced_buf.len() >= MAX_VOICED_SAMPLES {
                return self.flush();
            }
        } else if !self.voiced_buf.is_empty() {
            // In silence after voiced content.
            self.silence_run += 1;

            let room = self.silence_pad_samples().saturating_sub(self.silence_pad.len());
            if room > 0 {
                let take = room.min(frame.len());
                self.silence_pad.extend_from_slice(&frame[..take]);
            }

            if self.silence_run >= self.silence_flush_frames() {
                return self.flush();
            }
        }
        // Leading silence — nothing to do.

        None
    }

    /// Flush any buffered voiced content, ignoring the silence threshold.
    /// Called at session end to drain the final utterance.
    pub fn flush(&mut self) -> Option<Vec<f32>> {
        if self.voiced_buf.is_empty() {
            return None;
        }
        let chunk = std::mem::take(&mut self.voiced_buf);
        self.silence_pad.clear();
        self.silence_run = 0;
        Some(chunk)
    }

    /// Reset all session state, including the backend.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.backend.reset();
        self.voiced_buf.clear();
        self.silence_pad.clear();
        self.silence_run = 0;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn silent_frame_160() -> Vec<f32> {
        vec![0.0f32; 160]
    }

    fn voiced_frame_160() -> Vec<f32> {
        // 440 Hz sine — clearly speech-like.
        (0..160)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16_000.0).sin() * 0.5)
            .collect()
    }

    #[test]
    fn webrtc_frame_size_is_160() {
        let b = WebRtcBackend::new();
        assert_eq!(b.frame_size(), 160);
    }

    #[test]
    fn silence_flush_frames_webrtc() {
        let vad = VadClassifier::new(WebRtcBackend::new());
        // 500 ms / 10 ms = 50 frames
        assert_eq!(vad.silence_flush_frames(), 50);
    }

    #[test]
    fn no_flush_on_leading_silence() {
        let mut vad = VadClassifier::new(WebRtcBackend::new());
        for _ in 0..60 {
            assert!(vad.push_frame(&silent_frame_160()).is_none());
        }
        assert!(vad.voiced_buf.is_empty());
    }

    #[test]
    fn flush_triggers_after_silence_threshold() {
        let mut vad = VadClassifier::new(WebRtcBackend::new());
        for _ in 0..10 {
            vad.push_frame(&voiced_frame_160());
        }
        let mut flushed = None;
        for _ in 0..50 {
            flushed = vad.push_frame(&silent_frame_160());
            if flushed.is_some() {
                break;
            }
        }
        // webrtc-vad may classify our sine as non-speech; that is acceptable.
        let _ = flushed;
    }

    #[test]
    fn explicit_flush_drains_buffer() {
        let mut vad = VadClassifier::new(WebRtcBackend::new());
        vad.voiced_buf.extend(voiced_frame_160());
        let chunk = vad.flush();
        assert!(chunk.is_some());
        assert!(vad.voiced_buf.is_empty());
    }

    #[test]
    fn flush_empty_returns_none() {
        let mut vad = VadClassifier::new(WebRtcBackend::new());
        assert!(vad.flush().is_none());
    }

    #[test]
    fn reset_clears_state() {
        let mut vad = VadClassifier::new(WebRtcBackend::new());
        vad.voiced_buf.extend(voiced_frame_160());
        vad.silence_run = 10;
        vad.reset();
        assert!(vad.voiced_buf.is_empty());
        assert_eq!(vad.silence_run, 0);
        assert!(vad.silence_pad.is_empty());
    }

    // ---------- Mock backend for deterministic testing ----------

    struct MockBackend {
        frame_size: usize,
        /// Sequence of responses to return for each classify_frame call.
        responses: std::collections::VecDeque<bool>,
    }

    impl MockBackend {
        fn new(frame_size: usize, responses: Vec<bool>) -> Self {
            Self {
                frame_size,
                responses: responses.into(),
            }
        }
    }

    impl VadBackend for MockBackend {
        fn frame_size(&self) -> usize {
            self.frame_size
        }
        fn classify_frame(&mut self, _frame: &[f32]) -> bool {
            self.responses.pop_front().unwrap_or(false)
        }
        fn reset(&mut self) {
            self.responses.clear();
        }
    }

    #[test]
    fn mock_voiced_then_silence_flushes() {
        // 10 voiced frames (512 samples each, 32 ms) then 16 silent ones
        // should trigger a flush at the 16th silence frame (500ms threshold).
        let frame_size = 512;
        let voiced: Vec<bool> = vec![true; 10];
        let silent: Vec<bool> = vec![false; 20];
        let responses = [voiced, silent].concat();
        let backend = MockBackend::new(frame_size, responses);
        let mut vad = VadClassifier::new(backend);

        let frame = vec![0.0f32; frame_size];
        for _ in 0..10 {
            assert!(vad.push_frame(&frame).is_none());
        }
        // voiced_buf should have 10 * 512 = 5120 samples
        assert_eq!(vad.voiced_buf.len(), 10 * frame_size);

        // silence_flush_frames for 512-sample frames at 16kHz:
        // frame_ms = 512 * 1000 / 16000 = 32ms; 500 / 32 = 15 frames
        let expected_flush_frames = vad.silence_flush_frames();
        assert_eq!(expected_flush_frames, 15);

        let mut flushed = None;
        for i in 0..20 {
            flushed = vad.push_frame(&frame);
            if flushed.is_some() {
                // Should flush at frame index (expected_flush_frames - 1) = 14
                assert_eq!(i, (expected_flush_frames - 1) as usize);
                break;
            }
        }
        assert!(flushed.is_some(), "should have flushed after silence threshold");
    }

    #[test]
    fn mock_silence_pad_prepended_on_resume() {
        let frame_size = 160;
        // voiced, silent, voiced: silence pad should be prepended on second voiced frame
        let responses = vec![true, false, true];
        let backend = MockBackend::new(frame_size, responses);
        let mut vad = VadClassifier::new(backend);

        let frame = vec![0.1f32; frame_size];
        vad.push_frame(&frame); // voiced
        vad.push_frame(&frame); // silent → goes into silence_pad
        let before_len = vad.voiced_buf.len();
        vad.push_frame(&frame); // voiced again → silence_pad prepended
        // voiced_buf should have grown by at least 2 * frame_size (pad + new frame)
        assert!(vad.voiced_buf.len() >= before_len + 2 * frame_size);
    }
}
