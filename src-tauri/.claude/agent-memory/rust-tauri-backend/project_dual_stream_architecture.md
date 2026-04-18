---
name: dual-stream audio architecture
description: The dual-stream (fast-path + slow-path) speaker recognition architecture — replace model where slow path deletes fast segments and inserts its own fresh
type: project
---

The pipeline uses two parallel paths for audio processing:

**Fast path**: short VAD chunks (~2 s) are sent to speech-swift for ASR. Segments are inserted with `status = 'pending'`. Returned segment IDs are accumulated into `pending_fast_segment_ids: Vec<i64>` in `run_pipeline`.

**Slow path**: `SpeechAccumulator` collects the same silence-stripped frames. Once ≥ 10 s of speech accumulates (or on session stop/inactivity), a long clip is sent to speech-swift. The slow path then:
1. Calls `db::segments::delete_segments(&db, &fast_segment_ids)` — deletes all fast-path segments and their embeddings.
2. Inserts its own fresh `confirmed` segments with interpolated session timestamps.
3. Emits a `segments_replaced` Tauri event: `{ removed_ids: Vec<i64>, added: Vec<SegmentEvent> }`.

The frontend handles `segments_replaced` by removing rows with `removed_ids` and inserting the `added` rows. There is no in-place update — old pending rows are gone, new confirmed rows replace them.

**Timestamp mapping**: speech-swift returns timestamps in audio-clip-relative ms (silence stripped). `AccumulatorClip.chunks` maps each chunk's `audio_start_ms`/`audio_end_ms` to `session_start_ms`/`session_end_ms`. `audio_to_session_ms` interpolates: find the chunk where `audio_start_ms <= t < audio_end_ms`, then `session_start_ms + (t - audio_start_ms)`. Falls back to `clip_start_ms + t` if no chunk found.

**`AppendedChunk`** fields (in `src/audio/accumulator.rs`):
- `session_start_ms`, `session_end_ms` — wall-clock session time of this chunk
- `audio_start_ms`, `audio_end_ms` — position in the concatenated speech-only frame buffer

`AppendedChunk` no longer has `segment_ids`. `extend_segment_ids_in_range` has been removed.

**`SpeechAccumulator::append`** signature: `(speech_only: Vec<f32>, chunk_start_ms: u64, chunk_end_ms: u64)` — no `segment_ids` arg.

**Key invariant**: The DB mutex is always acquired, all DB work done (delete + insert), and the lock released before any `.await` or event emission.

**Why**: The previous in-place update model (slow path finding overlapping fast-segment IDs and calling `update_segment_speaker`) was fragile — segment IDs had to be threaded through the accumulator chunks. Replace model is simpler: fast segments are throwaway, slow segments are authoritative.

**How to apply**: When adding new pipeline logic, continue collecting fast-path IDs into `pending_fast_segment_ids`. Always pass `std::mem::take(&mut pending_fast_segment_ids)` to `run_slow_path` so the Vec is always empty after each slow drain.
