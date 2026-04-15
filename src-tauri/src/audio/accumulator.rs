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
    /// Mapping from each appended chunk to its frame offsets in `frames` and
    /// the segment IDs saved from that chunk. Used during slow-path processing
    /// to match long-clip segments back to pending DB rows.
    chunks: Vec<AppendedChunk>,
}

/// Records where a single fast-path chunk lands in the accumulator's frame
/// buffer, so slow-path results can be matched back to DB segment IDs.
pub struct AppendedChunk {
    /// Index into `frames` where this chunk's samples begin (inclusive).
    /// Available for future frame-extraction use; slow-path currently matches by time range.
    #[allow(dead_code)]
    pub frame_start: usize,
    /// Exclusive end index into `frames`.
    #[allow(dead_code)]
    pub frame_end: usize,
    /// Session-relative start time for this chunk (ms).
    #[allow(dead_code)]
    pub session_start_ms: u64,
    /// Session-relative end time for this chunk (ms).
    #[allow(dead_code)]
    pub session_end_ms: u64,
    /// The DB segment IDs saved from the fast-path pass for this chunk.
    pub segment_ids: Vec<i64>,
}

/// Trigger a slow-path clip after this many seconds of accumulated speech.
pub const SPEECH_TRIGGER_SECS: f64 = 10.0;

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
    /// in the session timeline. `segment_ids` are the DB row IDs of the
    /// fast-path segments saved from this chunk (may be empty if the fast path
    /// produced no transcript).
    pub fn append(
        &mut self,
        speech_only: Vec<f32>,
        chunk_start_ms: u64,
        chunk_end_ms: u64,
        segment_ids: Vec<i64>,
    ) {
        let frame_start = self.frames.len();
        if self.clip_start_ms.is_none() {
            self.clip_start_ms = Some(chunk_start_ms);
        }
        self.clip_end_ms = Some(chunk_end_ms);
        self.speech_secs += speech_only.len() as f64 / 16_000.0;
        self.frames.extend(speech_only);
        let frame_end = self.frames.len();
        self.last_append_at = Some(std::time::Instant::now());

        self.chunks.push(AppendedChunk {
            frame_start,
            frame_end,
            session_start_ms: chunk_start_ms,
            session_end_ms: chunk_end_ms,
            segment_ids,
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
    /// Chunk-to-frame mapping for retroactive speaker resolution.
    pub chunks: Vec<AppendedChunk>,
    /// Session-relative start of this clip in milliseconds.
    #[allow(dead_code)]
    pub clip_start_ms: u64,
    /// Session-relative end of this clip in milliseconds.
    #[allow(dead_code)]
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
        acc.append(make_frames(160), 0, 10, vec![]);
        assert_eq!(acc.clip_start_ms, Some(0));
        assert_eq!(acc.clip_end_ms, Some(10));

        acc.append(make_frames(160), 10, 20, vec![]);
        // clip_start stays at first append
        assert_eq!(acc.clip_start_ms, Some(0));
        assert_eq!(acc.clip_end_ms, Some(20));
    }

    #[test]
    fn append_records_chunk_frame_offsets() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160), 0, 10, vec![1]);
        acc.append(make_frames(320), 10, 30, vec![2, 3]);

        assert_eq!(acc.chunks.len(), 2);
        assert_eq!(acc.chunks[0].frame_start, 0);
        assert_eq!(acc.chunks[0].frame_end, 160);
        assert_eq!(acc.chunks[1].frame_start, 160);
        assert_eq!(acc.chunks[1].frame_end, 480);
    }

    #[test]
    fn drain_resets_accumulator() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160), 0, 10, vec![]);
        let clip = acc.drain().expect("should have clip");
        assert_eq!(clip.clip_start_ms, 0);
        assert_eq!(clip.clip_end_ms, 10);
        assert!(acc.is_empty());
        assert!(acc.clip_start_ms.is_none());
        assert!(acc.chunks.is_empty());
    }

    #[test]
    fn drain_includes_chunks() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160), 0, 10, vec![42]);
        let clip = acc.drain().expect("should have clip");
        assert_eq!(clip.chunks.len(), 1);
        assert_eq!(clip.chunks[0].segment_ids, vec![42]);
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
        acc.append(make_frames(480_000), 0, 30_000, vec![]);
        assert!(acc.should_trigger());
    }

    #[test]
    fn should_not_trigger_below_threshold() {
        let mut acc = SpeechAccumulator::new();
        acc.append(make_frames(160_000), 0, 10_000, vec![]);
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
        acc.append(make_frames(160), 0, 10, vec![]);
        // Immediately after append; elapsed is ~0, well below 10s threshold.
        assert!(!acc.should_flush_for_inactivity(std::time::Duration::from_secs(10)));
    }
}
