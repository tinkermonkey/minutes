---
name: dual-stream audio architecture
description: The dual-stream (fast-path + slow-path) speaker recognition architecture added to the audio pipeline
type: project
---

The pipeline uses two parallel paths for audio processing:

**Fast path**: short VAD chunks (~2–5 s) are sent to speech-swift for ASR only. speech-swift may skip speaker recognition for short clips (returns null `speaker_id`/`speaker_label`). Segments are inserted with `status = 'pending'` and `chunk_start`/`chunk_end` set to session-relative seconds for later overlap matching.

**Slow path**: `SpeechAccumulator` in `src/audio/accumulator.rs` collects the same silence-stripped speech frames. Once ≥ 30 s of speech accumulates, a long clip is submitted to speech-swift with `min_duration=0` to force speaker recognition. Results are mapped back to pending segments via overlap of `[chunk_start, chunk_end]` and segments are updated to `status = 'confirmed'` with the resolved `speaker_id`. A `speaker_resolved` Tauri event is emitted per resolved segment.

**Key invariant**: The DB mutex is always acquired, all DB work done, and the lock released before any `.await` — this pattern must be preserved when modifying `handle_chunk` or `handle_long_clip`.

**Why:** Short VAD chunks (2–5 s) are insufficient for speech-swift's speaker recognition minimum duration. Long clips give the ML model enough context for high-confidence speaker ID.

**How to apply:** When adding new audio pipeline logic, maintain the fast/slow path separation. Never move speaker resolution entirely to the fast path — short clips will produce unreliable speaker IDs.
