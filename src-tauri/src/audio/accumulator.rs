/// Accumulates speech frames from VAD-gated chunks for the slow path.
///
/// The dual-stream design sends short VAD chunks to speech-swift immediately
/// (fast path) for low-latency segment events. In parallel, this accumulator
/// collects the raw speech frames so that a longer clip can be sent periodically
/// (slow path) for accurate diarization with cross-speaker context.
///
/// `speech_secs` counts only frames classified as speech (no silence-pad),
/// avoiding over-counting caused by the encoder look-ahead padding.
pub struct SpeechAccumulator {
    /// Raw speech-only f32 samples (16 kHz mono).
    frames: Vec<f32>,
    /// Total seconds of speech accumulated (no padding included).
    pub speech_secs: f64,
    /// Wall-clock time of the last `append` call; used for inactivity detection.
    last_append_at: Option<std::time::Instant>,
    /// Offset in milliseconds from the session start for the first frame in
    /// `frames`. Set on the first `append` after a flush.
    pub clip_start_ms: Option<u64>,
    /// Offset in milliseconds for the end of the last appended chunk.
    pub clip_end_ms: Option<u64>,
    /// Mapping from each appended chunk to its audio-relative frame offsets and
    /// session-relative wall time. Used during slow-path processing to convert
    /// speech-swift's audio-clip-relative timestamps back to session time.
    chunks: Vec<AppendedChunk>,
}

/// Records where a single fast-path chunk lands in the session timeline so
/// slow-path results can map audio-clip-relative timestamps back to session time.
pub struct AppendedChunk {
    /// Session-relative start time for this chunk (ms).
    pub session_start_ms: u64,
    /// Offset (ms) of this chunk's first frame within the concatenated clip
    /// audio — i.e. how many ms of speech frames preceded this chunk in
    /// `SpeechAccumulator::frames`. speech-swift timestamps are relative to
    /// this same audio origin, so overlap checks must use these fields rather
    /// than the session-relative fields.
    pub audio_start_ms: u64,
    /// Offset (ms) of this chunk's last frame within the concatenated clip.
    pub audio_end_ms: u64,
}

/// Trigger a slow-path clip after this many seconds of accumulated speech.
pub const SPEECH_TRIGGER_SECS: f64 = 10.0;

/// Minimum speech duration before sending a fast-path clip to speech-swift.
/// speech-swift requires at least 2 s of audio to run speaker recognition.
pub const FAST_SPEECH_TRIGGER_SECS: f64 = 2.0;

impl SpeechAccumulator {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            speech_secs: 0.0,
            last_append_at: None,
            clip_start_ms: None,
            clip_end_ms: None,
            chunks: Vec::new(),
        }
    }

    /// Append speech-only frames from a VAD chunk.
    ///
    /// `speech_only` must be the no-pad samples from `VadClassifier`.
    /// `chunk_start_ms` / `chunk_end_ms` are the wall positions of the chunk
    /// in the session timeline.
    pub fn append(
        &mut self,
        speech_only: Vec<f32>,
        chunk_start_ms: u64,
        chunk_end_ms: u64,
    ) {
        if self.clip_start_ms.is_none() {
            self.clip_start_ms = Some(chunk_start_ms);
        }
        self.clip_end_ms = Some(chunk_end_ms);
        self.speech_secs += speech_only.len() as f64 / 16_000.0;

        // Measure audio-relative offsets from the frame buffer length before
        // and after the extend. speech-swift timestamps are in this same
        // audio-only coordinate space (silence stripped), so overlap checks in
        // run_slow_path must compare against these — not session wall-clock ms.
        let audio_start_ms = (self.frames.len() as u64 * 1000) / 16_000;
        self.frames.extend(speech_only);
        let audio_end_ms = (self.frames.len() as u64 * 1000) / 16_000;

        self.last_append_at = Some(std::time::Instant::now());

        self.chunks.push(AppendedChunk {
            session_start_ms: chunk_start_ms,
            audio_start_ms,
            audio_end_ms,
        });
    }

    /// Returns `true` when enough speech has accumulated to justify sending a
    /// slow-path clip to speech-swift.
    pub fn should_trigger(&self) -> bool {
        self.speech_secs >= SPEECH_TRIGGER_SECS
    }

    /// Returns `true` if no new audio has arrived for at least `timeout` and
    /// the accumulator is non-empty. Used to flush mid-session silence gaps.
    pub fn should_flush_for_inactivity(&self, timeout: std::time::Duration) -> bool {
        if self.is_empty() {
            return false;
        }
        self.last_append_at
            .map(|t| t.elapsed() >= timeout)
            .unwrap_or(false)
    }

    /// Returns `true` if no frames have been accumulated since the last drain.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Take all accumulated frames and timing metadata, resetting the accumulator.
    ///
    /// Returns `None` if the accumulator is empty.
    pub fn drain(&mut self) -> Option<AccumulatorClip> {
        if self.frames.is_empty() {
            return None;
        }
        let frames = std::mem::take(&mut self.frames);
        let chunks = std::mem::take(&mut self.chunks);
        let clip_start_ms = self.clip_start_ms.take();
        let clip_end_ms = self.clip_end_ms.take();
        self.speech_secs = 0.0;
        self.last_append_at = None;
        Some(AccumulatorClip {
            frames,
            chunks,
            clip_start_ms: clip_start_ms
                .expect("clip_start_ms is always set when frames is non-empty"),
            clip_end_ms: clip_end_ms
                .expect("clip_end_ms is always set when frames is non-empty"),
        })
    }
}

impl Default for SpeechAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

/// A drained clip ready for slow-path processing.
pub struct AccumulatorClip {
    /// Raw f32 speech-only samples at 16 kHz mono.
    pub frames: Vec<f32>,
    /// Per-chunk timing map: used by `run_slow_path` to convert speech-swift's
    /// audio-clip-relative timestamps back to session wall-clock time.
    pub chunks: Vec<AppendedChunk>,
    /// Session-relative start of this clip in milliseconds.
    pub clip_start_ms: u64,
    /// Session-relative end of this clip in milliseconds.
    pub clip_end_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frames(n: usize) -> Vec<f32> {
        vec![0.0f32; n]
    }

    #[test]
    fn new_accumulator_is_empty() {
        let acc = SpeechAccumulator::new();
        assert!(acc.is_empty());
        assert!(!acc.should_trigger());
    }

    #[test]
    fn append_sets_clip_bounds() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160), 0, 10);
        assert_eq!(acc.clip_start_ms, Some(0));
        assert_eq!(acc.clip_end_ms, Some(10));

        acc.append(make_frames(160), 10, 20);
        // clip_start stays at first append
        assert_eq!(acc.clip_start_ms, Some(0));
        assert_eq!(acc.clip_end_ms, Some(20));
    }

    #[test]
    fn drain_resets_accumulator() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160), 0, 10);
        let clip = acc.drain().expect("should have clip");
        assert_eq!(clip.clip_start_ms, 0);
        assert_eq!(clip.clip_end_ms, 10);
        assert!(acc.is_empty());
        assert!(acc.clip_start_ms.is_none());
        assert!(acc.chunks.is_empty());
    }

    #[test]
    fn drain_includes_chunks_with_audio_offsets() {
        let mut acc = SpeechAccumulator::new();
        // 160 frames = 10 ms at 16 kHz
        acc.append(make_frames(160), 0, 10);
        acc.append(make_frames(160), 10, 20);
        let clip = acc.drain().expect("should have clip");
        assert_eq!(clip.chunks.len(), 2);
        assert_eq!(clip.chunks[0].audio_start_ms, 0);
        assert_eq!(clip.chunks[0].audio_end_ms, 10);
        assert_eq!(clip.chunks[1].audio_start_ms, 10);
        assert_eq!(clip.chunks[1].audio_end_ms, 20);
    }

    #[test]
    fn drain_on_empty_returns_none() {
        let mut acc = SpeechAccumulator::new();
        assert!(acc.drain().is_none());
    }

    #[test]
    fn should_trigger_when_enough_speech() {
        let mut acc = SpeechAccumulator::new();
        // 30 s of speech at 16 kHz = 480_000 samples
        acc.append(make_frames(480_000), 0, 30_000);
        assert!(acc.should_trigger());
    }

    #[test]
    fn should_not_trigger_below_threshold() {
        let mut acc = SpeechAccumulator::new();
        // 9 s of speech at 16 kHz = 144_000 samples — just below the 10 s trigger.
        acc.append(make_frames(144_000), 0, 9_000);
        assert!(!acc.should_trigger());
    }

    #[test]
    fn inactivity_flush_false_when_empty() {
        let acc = SpeechAccumulator::new();
        assert!(!acc.should_flush_for_inactivity(std::time::Duration::from_secs(10)));
    }

    #[test]
    fn inactivity_flush_false_before_timeout() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160), 0, 10);
        // Immediately after append; elapsed is ~0, well below 10s threshold.
        assert!(!acc.should_flush_for_inactivity(std::time::Duration::from_secs(10)));
    }
}
