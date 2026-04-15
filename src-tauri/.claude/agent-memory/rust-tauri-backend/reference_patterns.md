---
name: established patterns
description: Recurring patterns for Tauri commands, DB mutex, events, and error handling
type: reference
---

## DB mutex pattern (CRITICAL invariant)
Always acquire the lock, do ALL DB work inside the block, drop before any `.await`:
```rust
let results: Vec<Foo> = {
    let db = state.db.lock().expect("db mutex poisoned");
    // ... all inserts/queries ...
    computed_vec
    // MutexGuard dropped here
};
// Now safe to .await
emit_events(results).await;
```

## Tauri event emission pattern
Collect events into a `Vec` inside the DB lock scope, emit after releasing:
- Fast-path segment: `emit_segment_added` + optional `emit_new_speaker`
- Slow-path resolution: `emit_speaker_resolved` + optional `emit_new_speaker`

## NewSegment fields (as of migration 4)
`session_id`, `speaker_id: Option<i64>`, `start_ms`, `end_ms`, `transcript_text`, `status: String` ("pending"|"confirmed"), `chunk_start: Option<f64>`, `chunk_end: Option<f64>`

## SegmentEvent fields (nullable speaker)
`speaker_id: Option<i64>`, `speaker_label: Option<String>`, `status: String` — fast-path segments arrive with no speaker (pending), slow-path resolution fires `speaker_resolved` event instead of updating the segment event.

## speaker_count with Option<i64>
Use `.filter_map(|s| s.speaker_id)` not `.map(|s| s.speaker_id)` when building the HashSet for `chunk_processed` event.

## Error handling
- `thiserror` for domain errors
- `anyhow::Result` in DB/client functions
- `.expect("...")` only for invariants (mutex poisoned, system time)
- Never `unwrap()` in production paths

## Slow-path concurrency model
The pipeline loop must never `await` slow-path diarization mid-session — that blocks fast-path ASR for 60+ seconds.
Three-function split in `commands/mod.rs`:
- `process_long_clip(session_id, AccumulatorClip, app: AppHandle)` — free async fn, owns the clip
- `handle_long_clip(session_id, &mut accumulator, &app)` — thin wrapper that `drain()`s then `await`s `process_long_clip`. Used ONLY at session end (shutdown context, blocking is OK).
- `spawn_long_clip(session_id, &mut accumulator, &app)` — `drain()`s and fires `tauri::async_runtime::spawn(...)`. No `await`. Used for all mid-session triggers (threshold and inactivity).

## Silero VAD / ort API (ort = 2.0.0-rc.9, ndarray 0.16)
- Session created via `Session::builder()?.commit_from_memory(&[u8])?`
- Inputs via `ort::inputs! { "name" => ndarray_array, ... }?` (returns `Result<SessionInputs>`)
- `session.run(inputs)?` returns `SessionOutputs` indexable by `&str`
- Extract f32 tensor: `outputs["name"].try_extract_tensor::<f32>()?` → `ArrayViewD<f32>`
- Reshape ndarray: `Array1::from_vec(v).into_shape_with_order((1, N))?`
- Build `Array3` from flat vec: `Array3::from_shape_vec((2, 1, 64), vec)?`
