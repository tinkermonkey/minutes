---
name: dual-stream pipeline wiring
description: How the dual-stream audio pipeline is wired — fast path saves pending segments, slow path resolves speaker IDs retroactively
type: project
---

The pipeline runs two parallel streams from the same VAD-gated audio:

**Fast path** (`handle_chunk`): sends each VAD chunk to speech-swift immediately, saves segments to SQLite with `status='pending'` when `speaker_id` is None, `status='confirmed'` when a speaker ID is returned. Emits `segment_added` Tauri event with `speaker_id=0` for pending segments. Returns `Option<(Vec<f32>, u64, u64, Vec<i64>)>` — speech frames, timing, and saved segment IDs — for feeding the accumulator.

**Slow path** (`handle_slow_path`): called when `SpeechAccumulator` triggers (30s of speech, 10s inactivity, or session end). Encodes accumulated frames to WAV via `encode_wav` (now `pub(crate)` in `chunker.rs`), calls `speech_swift::transcribe_chunk`, maps long-clip segment frame ranges back to `AppendedChunk` entries, calls `db::segments::update_segment_speaker`, emits `segment_speaker_resolved` events.

**Key types:**
- `AppendedChunk` in `accumulator.rs`: records `frame_start`, `frame_end` (indices into `SpeechAccumulator::frames`), timing, and `segment_ids` from the fast path
- `AccumulatorClip.chunks`: populated by `drain()`, consumed by `handle_slow_path`
- `NewSegment.speaker_id`: now `Option<i64>` — None means pending resolution

**Trigger logic** is a `macro_rules! process_chunk!` inside `run_pipeline` that checks `should_trigger()` then `should_flush_for_inactivity(10s)` after every chunk.

**Why:** The speech-swift registry needs ≥30s of cumulative speech for reliable centroid enrollment. Sending 2–5s VAD chunks causes an exploding registry with no matches. The dual stream gives low-latency transcription AND enrollment-quality speaker IDs.
