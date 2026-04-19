pub mod devices;
pub mod health;
pub mod search;
pub mod sessions;
pub mod settings;
pub mod speakers;

use tauri::Manager;
use tokio::sync::oneshot;

use crate::{
    audio::{capture::start_capture, DynChunker, VadMode},
    client::speech_swift,
    db::{self, segments::NewSegment},
    embed,
    events::{self, SegmentEvent, SegmentsReplacedEvent, SpeakerEvent},
    state::AppState,
};

/// Compute the root mean square of a sample slice.
///
/// Returns `0.0` for an empty slice rather than NaN.
fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Current time as milliseconds since the Unix epoch.
///
/// Uses `expect` because `SystemTime::now()` returning before the epoch is an
/// invariant that cannot occur on any supported platform.
fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time must be after Unix epoch")
        .as_millis() as i64
}

/// Persist `bytes` to `<app_data>/audio_chunks/<session_id>/<start_ms>.wav`
/// and return the full path as a String.
///
/// Uses `expect` because these are invariants guaranteed by Tauri's path API
/// and `fs::create_dir_all` — if either fails we cannot proceed anyway.
fn save_wav_chunk(app: &tauri::AppHandle, session_id: i64, start_ms: u64, bytes: &[u8]) -> String {
    let dir = app
        .path()
        .app_data_dir()
        .expect("Tauri app_data_dir must be set")
        .join("audio_chunks")
        .join(session_id.to_string());
    std::fs::create_dir_all(&dir).expect("failed to create audio_chunks directory");
    let path = dir.join(format!("{start_ms}.wav"));
    std::fs::write(&path, bytes).expect("failed to write WAV chunk to disk");
    path.to_string_lossy().into_owned()
}

/// A chunk produced by the capture thread, ready for transcription.
struct AudioChunk {
    wav_bytes:     Vec<u8>,
    /// Raw f32 speech-only samples (16 kHz mono) — fed to the slow-path accumulator.
    speech_frames: Vec<f32>,
    start_ms:      u64,
    end_ms:        u64,
}

/// Send one WAV chunk to the audio-server, persist results to SQLite, compute
/// embeddings, and fire Tauri events.
///
/// Returns the segment IDs inserted so the caller can add them to
/// `pending_fast_segment_ids` for replacement when the slow path fires.
///
/// The DB mutex is acquired, all inserts are done synchronously, and the lock
/// is released before any `.await`. This is the critical invariant that keeps
/// `Mutex<Connection>` safe on the async executor.
async fn handle_chunk(
    session_id:  i64,
    chunk:       AudioChunk,
    app:         &tauri::AppHandle,
    embed_queue: &mut Vec<(i64, String)>,
    language:    &str,
) -> Vec<i64> {
    let audio_path = save_wav_chunk(app, session_id, chunk.start_ms, &chunk.wav_bytes);

    let state = app.state::<AppState>();
    let base_url = state.speech_swift_url.clone();

    let chunk_start_secs = chunk.start_ms as f64 / 1000.0;
    let chunk_end_secs   = chunk.end_ms   as f64 / 1000.0;

    events::emit_chunk_sent(app, events::ChunkSentEvent {
        start_ms:   chunk.start_ms,
        end_ms:     chunk.end_ms,
        sent_at_ms: unix_ms() as u64,
    });

    let t0 = std::time::Instant::now();
    let response = match speech_swift::transcribe_chunk(&base_url, chunk.wav_bytes, language).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("speech-swift transcribe error: {e}");
            events::emit_speech_swift_unreachable(app);
            return Vec::new();
        }
    };
    let response_ms = t0.elapsed().as_millis() as u64;

    // --- DB work: acquire lock, do all inserts, release before any await. ---
    let now_ms = unix_ms();
    let (events_to_emit, segment_ids): (Vec<(SegmentEvent, Option<SpeakerEvent>)>, Vec<i64>) = {
        let db = state.db.lock().expect("db mutex poisoned");

        let mut events = Vec::with_capacity(response.segments.len());
        let mut ids    = Vec::with_capacity(response.segments.len());

        for seg in &response.segments {
            // Upsert the speaker only when speech-swift assigned a speaker ID.
            // A None speaker_id means the segment is pending — skip upsert.
            let speaker_opt: Option<(crate::db::speakers::Speaker, bool)> =
                if let Some(sid) = seg.speaker_id {
                    match db::speakers::upsert_speaker(&db, sid, now_ms) {
                        Ok(r) => Some(r),
                        Err(e) => {
                            eprintln!("upsert speaker error: {e}");
                            continue;
                        }
                    }
                } else {
                    None
                };

            let transcript_text = seg.transcript.clone().unwrap_or_default();

            let segment_id = match db::segments::insert_segment(
                &db,
                &NewSegment {
                    session_id,
                    speaker_id:       seg.speaker_id,
                    start_ms:         chunk.start_ms as i64 + (seg.start * 1000.0) as i64,
                    end_ms:           chunk.start_ms as i64 + (seg.end   * 1000.0) as i64,
                    transcript_text:  transcript_text.clone(),
                    chunk_start_secs: Some(chunk_start_secs),
                    chunk_end_secs:   Some(chunk_end_secs),
                },
            ) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("insert segment error: {e}");
                    continue;
                }
            };

            ids.push(segment_id);

            if let Some((ref speaker, _)) = speaker_opt {
                let _ = db::samples::insert_speaker_sample(
                    &db,
                    speaker.id,
                    session_id,
                    chunk.start_ms as i64 + (seg.start * 1000.0) as i64,
                    chunk.start_ms as i64 + (seg.end   * 1000.0) as i64,
                    &audio_path,
                );
            }

            // Try embedding synchronously; on failure, defer to the drain queue.
            match embed::embed(&transcript_text) {
                Ok(vec) => {
                    let _ = db::segments::insert_segment_embedding(&db, segment_id, &vec);
                }
                Err(_) => {
                    embed_queue.push((segment_id, transcript_text.clone()));
                }
            }

            let (display_name, speaker_event) = match speaker_opt {
                Some((ref speaker, is_new)) => {
                    let ev = if is_new || speaker.display_name.is_none() {
                        Some(SpeakerEvent {
                            id:              speaker.id,
                            speech_swift_id: speaker.speech_swift_id,
                            display_name:    speaker.display_name.clone(),
                        })
                    } else {
                        None
                    };
                    (speaker.display_name.clone(), ev)
                }
                None => (None, None),
            };

            events.push((
                SegmentEvent {
                    id:              segment_id,
                    session_id,
                    speaker_id:      seg.speaker_id,
                    speaker_label:   seg.speaker_label.clone(),
                    display_name,
                    status:          if seg.speaker_id.is_some() { "confirmed".to_string() } else { "pending".to_string() },
                    start_ms:        chunk.start_ms as i64 + (seg.start * 1000.0) as i64,
                    end_ms:          chunk.start_ms as i64 + (seg.end   * 1000.0) as i64,
                    transcript_text: transcript_text.clone(),
                },
                speaker_event,
            ));
        }

        (events, ids)
        // `db` MutexGuard dropped here — lock released before any await.
    };

    // Emit chunk_processed now that we have response stats and DB work is done.
    {
        use std::collections::HashSet;
        let word_count: u32 = response.segments.iter()
            .map(|s| s.transcript.as_deref().unwrap_or("").split_whitespace().count() as u32)
            .sum();
        let speaker_count: u32 = response.segments.iter()
            .map(|s| s.speaker_id)
            .collect::<HashSet<_>>()
            .len() as u32;
        let best_score: Option<f32> = response.segments.iter()
            .filter_map(|s| s.best_score)
            .reduce(f32::min);
        events::emit_chunk_processed(app, events::ChunkProcessedEvent {
            start_ms: chunk.start_ms,
            response_ms,
            word_count,
            speaker_count,
            best_score,
        });
    }

    // Fire segment/speaker events after releasing the DB lock.
    for (seg_event, speaker_event) in events_to_emit {
        events::emit_segment_added(app, seg_event);
        if let Some(ev) = speaker_event {
            events::emit_new_speaker(app, ev);
        }
    }

    segment_ids
}

/// Retry all segments that failed to embed during the hot path.
/// Embeds each text first (CPU-bound, no lock held), then acquires the DB
/// lock only for the insert.
async fn drain_embed_queue(queue: Vec<(i64, String)>, app: &tauri::AppHandle) {
    if queue.is_empty() {
        return;
    }
    let state = app.state::<AppState>();
    for (segment_id, text) in queue {
        match embed::embed(&text) {
            Ok(vec) => {
                let db = state.db.lock().expect("db mutex poisoned");
                let _ = db::segments::insert_segment_embedding(&db, segment_id, &vec);
            }
            Err(e) => eprintln!("embed drain error for segment {segment_id}: {e}"),
        }
    }
}

/// Process an accumulated speech clip through speech-swift (slow path).
///
/// Drains the accumulator, encodes the frames as WAV, POSTs to speech-swift,
/// then deletes the fast-path segments (`fast_segment_ids`) from SQLite and
/// inserts the slow-path results as fresh `confirmed` segments. Fires a
/// `segments_replaced` event so the frontend can swap the pending rows for the
/// authoritative slow-path results.
async fn run_slow_path(
    session_id:       i64,
    base_url:         &str,
    app:              &tauri::AppHandle,
    accumulator:      &mut crate::audio::SpeechAccumulator,
    fast_segment_ids: Vec<i64>,
    language:         &str,
    embed_queue:      &mut Vec<(i64, String)>,
) {
    let Some(clip) = accumulator.drain() else { return };

    // Emit zeroed accumulator state after drain.
    events::emit_accumulator_updated(app, events::AccumulatorUpdatedEvent {
        speech_secs:  0.0,
        trigger_secs: crate::audio::accumulator::SPEECH_TRIGGER_SECS,
    });

    let wav_bytes = crate::audio::chunker::encode_wav(&clip.frames);
    let audio_path = save_wav_chunk(app, session_id, clip.clip_start_ms, &wav_bytes);
    let speech_secs = clip.frames.len() as f64 / 16_000.0;

    events::emit_slow_path_sent(app, events::SlowPathSentEvent {
        start_ms:         clip.clip_start_ms,
        end_ms:           clip.clip_end_ms,
        clip_speech_secs: speech_secs,
        sent_at_ms:       unix_ms() as u64,
    });

    let t0 = std::time::Instant::now();
    let response = match speech_swift::transcribe_chunk(base_url, wav_bytes, language).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("slow-path speech-swift error: {e}");
            events::emit_speech_swift_unreachable(app);
            return;
        }
    };
    let response_ms = t0.elapsed().as_millis() as u64;

    let best_score: Option<f32> = response.segments.iter()
        .filter_map(|s| s.best_score)
        .reduce(f32::min);

    events::emit_slow_path_done(app, events::SlowPathDoneEvent {
        start_ms:      clip.clip_start_ms,
        response_ms,
        segment_count: response.segments.len() as u32,
        best_score,
    });

    // --- DB work: acquire lock, delete fast segments, insert slow segments. ---
    let state = app.state::<AppState>();
    let now_ms = unix_ms();

    // Resolve session-time for a speech-swift audio-clip-relative timestamp.
    // `seg_audio_ms` is milliseconds from the clip's audio origin (silence
    // stripped). Walk the chunk list to find which raw chunk contributed those
    // frames, then interpolate the session wall-clock time.
    //
    // The fallback handles two edge cases:
    //   1. `seg_audio_ms` lands exactly on the last chunk's `audio_end_ms`
    //      (the loop's `< audio_end_ms` guard misses it).
    //   2. The chunk list is empty (should never happen but is safe).
    // In both cases we extrapolate from the last chunk rather than adding
    // a raw audio offset to `clip_start_ms`, which would be incorrect.
    let audio_to_session_ms = |seg_audio_ms: u64| -> u64 {
        for ch in &clip.chunks {
            if ch.audio_start_ms <= seg_audio_ms && seg_audio_ms < ch.audio_end_ms {
                return ch.session_start_ms + (seg_audio_ms - ch.audio_start_ms);
            }
        }
        // Extrapolate from the last chunk (handles end-of-clip boundary).
        if let Some(last) = clip.chunks.last() {
            last.session_start_ms + seg_audio_ms.saturating_sub(last.audio_start_ms)
        } else {
            clip.clip_start_ms.saturating_add(seg_audio_ms)
        }
    };

    let (new_segment_events, new_speaker_events): (Vec<SegmentEvent>, Vec<SpeakerEvent>) = {
        let db = state.db.lock().expect("db mutex poisoned");

        // Delete all fast-path segments (and their embeddings) for this clip.
        if let Err(e) = db::segments::delete_segments(&db, &fast_segment_ids) {
            eprintln!("slow-path delete_segments error: {e}");
        }

        let mut seg_events    = Vec::new();
        let mut speaker_events = Vec::new();
        let mut sampled_speakers = std::collections::HashSet::<i64>::new();

        for seg in &response.segments {
            // Skip segments where speech-swift gave no speaker — we have nothing
            // to confirm without a speaker identity.
            let Some(speaker_id) = seg.speaker_id else { continue };

            let (speaker, is_new) = match db::speakers::upsert_speaker(&db, speaker_id, now_ms) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("slow-path upsert speaker error: {e}");
                    continue;
                }
            };

            let seg_start_audio_ms = (seg.start * 1000.0) as u64;
            let seg_end_audio_ms   = (seg.end   * 1000.0) as u64;

            let start_ms = audio_to_session_ms(seg_start_audio_ms) as i64;
            let end_ms   = audio_to_session_ms(seg_end_audio_ms)   as i64;

            let transcript_text = seg.transcript.clone().unwrap_or_default();

            let segment_id = match db::segments::insert_segment(
                &db,
                &NewSegment {
                    session_id,
                    speaker_id:       Some(speaker_id),
                    start_ms,
                    end_ms,
                    transcript_text:  transcript_text.clone(),
                    chunk_start_secs: Some(clip.clip_start_ms as f64 / 1000.0),
                    chunk_end_secs:   Some(clip.clip_end_ms   as f64 / 1000.0),
                },
            ) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("slow-path insert_segment error: {e}");
                    continue;
                }
            };

            // Store one voice sample per speaker per slow-path clip.
            if sampled_speakers.insert(speaker.id) {
                let _ = db::samples::insert_speaker_sample(
                    &db,
                    speaker.id,
                    session_id,
                    start_ms,
                    end_ms,
                    &audio_path,
                );
            }

            // Try embedding synchronously; on failure defer to drain queue.
            match embed::embed(&transcript_text) {
                Ok(vec) => {
                    let _ = db::segments::insert_segment_embedding(&db, segment_id, &vec);
                }
                Err(_) => {
                    embed_queue.push((segment_id, transcript_text.clone()));
                }
            }

            seg_events.push(SegmentEvent {
                id:              segment_id,
                session_id,
                speaker_id:      Some(speaker_id),
                speaker_label:   seg.speaker_label.clone(),
                display_name:    speaker.display_name.clone(),
                status:          "confirmed".to_string(),
                start_ms,
                end_ms,
                transcript_text,
            });

            if is_new || speaker.display_name.is_none() {
                speaker_events.push(SpeakerEvent {
                    id:              speaker.id,
                    speech_swift_id: speaker.speech_swift_id,
                    display_name:    speaker.display_name.clone(),
                });
            }
        }

        (seg_events, speaker_events)
        // `db` MutexGuard dropped here — lock released before any emits.
    };

    // Emit the replacement event: frontend removes fast-path rows and inserts
    // the authoritative slow-path segments.
    events::emit_segments_replaced(app, SegmentsReplacedEvent {
        removed_ids: fast_segment_ids,
        added:       new_segment_events,
    });

    // Emit new_speaker for genuinely new speakers.
    for ev in new_speaker_events {
        events::emit_new_speaker(app, ev);
    }
}

/// Drain the fast-path accumulator, encode the accumulated frames as WAV, send
/// to speech-swift, and persist the results.
///
/// Returns the inserted segment IDs so the caller can add them to
/// `pending_fast_segment_ids`. Returns `None` if the accumulator is empty.
async fn run_fast_path(
    session_id:       i64,
    app:              &tauri::AppHandle,
    embed_queue:      &mut Vec<(i64, String)>,
    fast_accumulator: &mut crate::audio::SpeechAccumulator,
    language:         &str,
) -> Option<Vec<i64>> {
    let clip = fast_accumulator.drain()?;
    let wav_bytes = crate::audio::chunker::encode_wav(&clip.frames);
    let audio_chunk = AudioChunk {
        wav_bytes,
        speech_frames: Vec::new(), // not consumed by handle_chunk
        start_ms: clip.clip_start_ms,
        end_ms:   clip.clip_end_ms,
    };
    let ids = handle_chunk(session_id, audio_chunk, app, embed_queue, language).await;
    // Signal that the fast-path accumulator has been drained.
    events::emit_fast_accumulator_updated(app, events::FastAccumulatorUpdatedEvent {
        speech_secs:  0.0,
        trigger_secs: crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS,
    });
    Some(ids)
}

/// How long after the fast accumulator reaches its threshold before the fast
/// path actually fires. Gives the slow path a window to cancel the fast send
/// via clear-and-cover if both would fire for the same audio.
const FAST_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(500);

/// Core pipeline task.
///
/// Architecture: `webrtc_vad::Vad` contains a raw C pointer (`*mut Fvad`) and
/// is therefore `!Send`. Rather than wrapping it in an unsafe `Send` impl, we
/// isolate all `!Send` work on a dedicated OS thread. That thread owns CPAL
/// capture + the VAD + the Chunker, and sends completed WAV chunks through a
/// bounded `std::sync::mpsc` channel. The async task on the tokio executor
/// reads from a `tokio::sync::mpsc` receiver and handles the network + DB
/// work, which is legitimately async.
async fn run_pipeline(
    session_id: i64,
    app_handle: tauri::AppHandle,
    stop_rx:    oneshot::Receiver<()>,
    language:   String,
) {
    // Warm the embedding model in the background so the first segment does not
    // stall while ONNX loads.
    tokio::task::spawn_blocking(|| {
        let _ = embed::get_model();
    });

    // Bridge: capture thread -> async consumer.
    let (chunk_tx, mut chunk_rx) = tokio::sync::mpsc::channel::<AudioChunk>(4);
    // Stop signal forwarded to the capture thread.
    let (thread_stop_tx, thread_stop_rx) = std::sync::mpsc::channel::<()>();

    // Read the preferred device before spawning — the closure takes ownership.
    let preferred_device = app_handle
        .state::<AppState>()
        .preferred_device
        .lock()
        .expect("preferred_device mutex poisoned")
        .clone();

    // Read the persisted VAD mode (default Silero) before spawning.
    let vad_mode = {
        let state = app_handle.state::<AppState>();
        let conn = state.db.lock().expect("db mutex poisoned");
        db::settings::get(&conn, "vad_mode")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<VadMode>(&s).ok())
            .unwrap_or(VadMode::WebRtc)
    };

    // Resolve the Silero model path from the Tauri resource directory.
    // Falls back to WebRTC VAD if the path cannot be determined.
    let model_path = app_handle
        .path()
        .resource_dir()
        .ok()
        .map(|d| d.join("resources").join("silero_vad.onnx"));

    let level_app = app_handle.clone();
    // Clone app_handle for VAD state events emitted from the OS thread.
    let vad_app = app_handle.clone();

    // Spawn a plain OS thread that owns the `!Send` types (Vad, CPAL stream).
    std::thread::spawn(move || {
        let capture = match start_capture(preferred_device.as_deref()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("CPAL capture start error: {e}");
                return;
            }
        };

        // Keep _stream alive for the duration of the thread.
        let crate::audio::capture::CaptureHandle { rx: mut sample_rx, _stream } = capture;
        events::emit_mic_active(&vad_app, true);

        // Build the DynChunker using the persisted VAD mode.
        // Falls back to WebRTC when resource_dir is unavailable regardless of setting.
        let mut chunker = match model_path {
            Some(ref p) => DynChunker::new(vad_mode, p),
            None => {
                eprintln!("VAD: resource_dir unavailable, falling back to WebRTC VAD");
                DynChunker::new(VadMode::WebRtc, std::path::Path::new(""))
            }
        };
        let mut last_level_emit = std::time::Instant::now();
        let mut last_vad_active = false;

        loop {
            // Poll the stop channel — non-blocking.
            if thread_stop_rx.try_recv().is_ok() {
                break;
            }

            // Drain all available sample batches without blocking.
            // Accumulate into a local vec so RMS reflects the full tick window.
            let mut tick_samples: Vec<f32> = Vec::new();
            while let Ok(samples) = sample_rx.try_recv() {
                tick_samples.extend_from_slice(&samples);
                if let Some(output) = chunker.push_samples(&samples) {
                    let _ = chunk_tx.blocking_send(AudioChunk {
                        wav_bytes:     output.wav_bytes,
                        speech_frames: output.speech_frames,
                        start_ms:      output.start_ms,
                        end_ms:        output.end_ms,
                    });
                }
                // Note: only the final frame's VAD state is observable per CPAL batch.
                // Sub-batch speech↔silence transitions within a single batch are not emitted.
                // In practice CPAL batches are short (~10ms) so this is rarely consequential.
                let currently_speech = chunker.is_speech();
                if currently_speech != last_vad_active {
                    last_vad_active = currently_speech;
                    events::emit_vad_state(&vad_app, currently_speech);
                }
            }

            if tick_samples.is_empty() {
                // Avoid busy-spin when the mic buffer is empty.
                std::thread::sleep(std::time::Duration::from_millis(5));
            } else if last_level_emit.elapsed() >= std::time::Duration::from_millis(50) {
                events::emit_audio_level(&level_app, compute_rms(&tick_samples));
                last_level_emit = std::time::Instant::now();
            }
        }

        // Flush remaining voiced content.
        if let Some(output) = chunker.flush() {
            let _ = chunk_tx.blocking_send(AudioChunk {
                wav_bytes:     output.wav_bytes,
                speech_frames: output.speech_frames,
                start_ms:      output.start_ms,
                end_ms:        output.end_ms,
            });
        }

        events::emit_mic_active(&vad_app, false);
        // Reset the meter on the frontend when capture ends.
        events::emit_audio_level(&level_app, 0.0);
        // chunk_tx dropped here, closing the channel and signalling the async
        // consumer to finish.
    });

    // Async consumer: handles network + DB work.
    let mut embed_queue: Vec<(i64, String)> = Vec::new();
    let mut accumulator = crate::audio::SpeechAccumulator::new();
    // Fast-path accumulator: batches VAD chunks until >= 2 s before sending to
    // speech-swift. speech-swift requires at least 2 s for speaker recognition.
    let mut fast_accumulator = crate::audio::SpeechAccumulator::new();
    // All fast-path segment IDs produced since the last slow-path drain. When
    // the slow path fires it takes ownership of this list, deletes those fast
    // segments from the DB, and inserts its own authoritative rows instead.
    let mut pending_fast_segment_ids: Vec<i64> = Vec::new();
    // When the fast accumulator crosses its threshold, this is set to
    // `now + FAST_DEBOUNCE`. The select loop's debounce arm fires the fast path
    // after the deadline passes. Slow-path triggers clear it (cancel-and-cover).
    let mut fast_debounce_deadline: Option<tokio::time::Instant> = None;
    let base_url = app_handle.state::<AppState>().speech_swift_url.clone();

    /// Appends the raw VAD chunk to both accumulators immediately (same frames,
    /// same timing), arms the fast-path debounce when enough speech has built
    /// up, and fires the slow path when its threshold is met.
    ///
    /// The fast path is NOT fired directly here. Instead, `fast_debounce_deadline`
    /// is set to `now + FAST_DEBOUNCE` the first time the fast accumulator
    /// crosses its threshold. The select loop's debounce arm fires fast after
    /// the deadline passes. If slow triggers before the deadline, it clears the
    /// deadline (cancel) and applies clear-and-cover.
    ///
    /// Fast-path segment IDs are collected into `pending_fast_segment_ids`. When
    /// the slow path fires it takes ownership of those IDs, deletes the fast
    /// segments from the DB, and inserts its own authoritative segments instead.
    async fn maybe_run_fast_path(
        session_id:               i64,
        app:                      &tauri::AppHandle,
        embed_queue:              &mut Vec<(i64, String)>,
        fast_accumulator:         &mut crate::audio::SpeechAccumulator,
        slow_accumulator:         &mut crate::audio::SpeechAccumulator,
        pending_fast_segment_ids: &mut Vec<i64>,
        fast_debounce_deadline:   &mut Option<tokio::time::Instant>,
        base_url:                 &str,
        speech_frames:            Vec<f32>,
        start_ms:                 u64,
        end_ms:                   u64,
        language:                 &str,
    ) {
        // Both accumulators receive the same raw speech frames immediately so
        // neither is ever starved when the other hasn't reached its threshold.
        fast_accumulator.append(speech_frames.clone(), start_ms, end_ms);
        slow_accumulator.append(speech_frames, start_ms, end_ms);

        events::emit_fast_accumulator_updated(app, events::FastAccumulatorUpdatedEvent {
            speech_secs:  fast_accumulator.speech_secs,
            trigger_secs: crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS,
        });
        events::emit_accumulator_updated(app, events::AccumulatorUpdatedEvent {
            speech_secs:  slow_accumulator.speech_secs,
            trigger_secs: crate::audio::accumulator::SPEECH_TRIGGER_SECS,
        });

        // Arm the debounce when the fast accumulator first crosses its threshold.
        // The actual send happens in the select loop's debounce arm after 500 ms.
        if fast_accumulator.speech_secs >= crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS
            && fast_debounce_deadline.is_none()
        {
            *fast_debounce_deadline =
                Some(tokio::time::Instant::now() + FAST_DEBOUNCE);
        }

        if slow_accumulator.should_trigger() {
            // Clear-and-cover: slow fires — cancel the pending fast debounce and
            // discard the fast accumulator. The slow clip already contains those
            // frames; sending fast here would duplicate the API call.
            *fast_debounce_deadline = None;
            fast_accumulator.drain();
            events::emit_fast_accumulator_updated(app, events::FastAccumulatorUpdatedEvent {
                speech_secs:  0.0,
                trigger_secs: crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS,
            });
            run_slow_path(
                session_id, base_url, app, slow_accumulator,
                std::mem::take(pending_fast_segment_ids),
                language, embed_queue,
            ).await;
        }
    }

    tokio::select! {
        _ = stop_rx => {
            // Tell the capture thread to flush and exit.
            let _ = thread_stop_tx.send(());
            // Drain any remaining chunks the thread flushed before exiting.
            while let Some(chunk) = chunk_rx.recv().await {
                let speech_frames = chunk.speech_frames;
                let start_ms = chunk.start_ms;
                let end_ms   = chunk.end_ms;
                maybe_run_fast_path(
                    session_id, &app_handle, &mut embed_queue,
                    &mut fast_accumulator, &mut accumulator,
                    &mut pending_fast_segment_ids,
                    &mut fast_debounce_deadline,
                    &base_url, speech_frames, start_ms, end_ms, &language,
                ).await;
            }
            // Clear-and-cover at session end: slow covers the fast accumulator,
            // so discard fast to avoid sending identical audio twice. Only fire
            // fast alone when slow has nothing (short session below 10 s threshold).
            if !accumulator.is_empty() {
                fast_accumulator.drain();
                events::emit_fast_accumulator_updated(&app_handle, events::FastAccumulatorUpdatedEvent {
                    speech_secs:  0.0,
                    trigger_secs: crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS,
                });
                run_slow_path(
                    session_id, &base_url, &app_handle, &mut accumulator,
                    std::mem::take(&mut pending_fast_segment_ids),
                    &language, &mut embed_queue,
                ).await;
            } else if !fast_accumulator.is_empty() {
                if let Some(ids) =
                    run_fast_path(session_id, &app_handle, &mut embed_queue, &mut fast_accumulator, &language).await
                {
                    pending_fast_segment_ids.extend(ids);
                }
            }
        }
        _ = async {
            let inactivity_timeout = std::time::Duration::from_secs(20);
            let mut inactivity_check = tokio::time::interval(std::time::Duration::from_secs(5));
            inactivity_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                // Snapshot the debounce deadline each iteration. Option<Instant> is
                // Copy, so this is cheap and avoids a borrow conflict with the mut
                // refs passed to maybe_run_fast_path in the chunk arm.
                let debounce_snapshot = fast_debounce_deadline;

                tokio::select! {
                    biased; // prefer chunk processing over timers
                    chunk = chunk_rx.recv() => {
                        match chunk {
                            Some(chunk) => {
                                let speech_frames = chunk.speech_frames;
                                let start_ms = chunk.start_ms;
                                let end_ms   = chunk.end_ms;
                                maybe_run_fast_path(
                                    session_id, &app_handle, &mut embed_queue,
                                    &mut fast_accumulator, &mut accumulator,
                                    &mut pending_fast_segment_ids,
                                    &mut fast_debounce_deadline,
                                    &base_url, speech_frames, start_ms, end_ms, &language,
                                ).await;
                            }
                            None => break,
                        }
                    }
                    // Debounce arm: fires 500 ms after the fast accumulator crossed
                    // its threshold (if slow hasn't cancelled it by then).
                    _ = async {
                        match debounce_snapshot {
                            Some(dl) => tokio::time::sleep_until(dl).await,
                            None     => std::future::pending::<()>().await,
                        }
                    } => {
                        if let Some(ids) =
                            run_fast_path(session_id, &app_handle, &mut embed_queue, &mut fast_accumulator, &language).await
                        {
                            pending_fast_segment_ids.extend(ids);
                        }
                        fast_debounce_deadline = None;
                    }
                    _ = inactivity_check.tick() => {
                        // Inactivity flushes require a minimum of 2 s of speech
                        // before firing. Below that, the clip is too short for
                        // speech-swift to diarize reliably, and the flush is
                        // likely a false alarm caused by buffered-but-unprocessed
                        // chunks that haven't yet updated `last_append_at`. Skipping
                        // lets those chunks arrive naturally and merge into the next
                        // accumulation window.
                        let min_speech = crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS;
                        if accumulator.should_flush_for_inactivity(inactivity_timeout)
                            && accumulator.speech_secs >= min_speech
                        {
                            // Clear-and-cover: slow takes priority. Discard the
                            // fast accumulator rather than sending duplicate audio.
                            fast_debounce_deadline = None;
                            fast_accumulator.drain();
                            events::emit_fast_accumulator_updated(&app_handle, events::FastAccumulatorUpdatedEvent {
                                speech_secs:  0.0,
                                trigger_secs: crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS,
                            });
                            run_slow_path(
                                session_id, &base_url, &app_handle, &mut accumulator,
                                std::mem::take(&mut pending_fast_segment_ids),
                                &language, &mut embed_queue,
                            ).await;
                        } else if fast_accumulator.should_flush_for_inactivity(inactivity_timeout)
                            && fast_accumulator.speech_secs >= min_speech
                        {
                            if let Some(ids) =
                                run_fast_path(session_id, &app_handle, &mut embed_queue, &mut fast_accumulator, &language).await
                            {
                                pending_fast_segment_ids.extend(ids);
                            }
                            fast_debounce_deadline = None;
                        }
                    }
                }
            }
        } => {
            // Capture thread exited on its own (e.g. device disconnected).
            // Same clear-and-cover rule: slow covers fast at exit.
            if !accumulator.is_empty() {
                fast_accumulator.drain();
                events::emit_fast_accumulator_updated(&app_handle, events::FastAccumulatorUpdatedEvent {
                    speech_secs:  0.0,
                    trigger_secs: crate::audio::accumulator::FAST_SPEECH_TRIGGER_SECS,
                });
                run_slow_path(
                    session_id, &base_url, &app_handle, &mut accumulator,
                    std::mem::take(&mut pending_fast_segment_ids),
                    &language, &mut embed_queue,
                ).await;
            } else if !fast_accumulator.is_empty() {
                if let Some(ids) =
                    run_fast_path(session_id, &app_handle, &mut embed_queue, &mut fast_accumulator, &language).await
                {
                    pending_fast_segment_ids.extend(ids);
                }
            }
        }
    }

    drain_embed_queue(embed_queue, &app_handle).await;
}

/// Start a new recording session.
///
/// Creates a session row in SQLite, spawns the capture pipeline task, and
/// returns the session id so the frontend can track and stop it later.
#[tauri::command]
pub async fn start_session(
    app:      tauri::AppHandle,
    state:    tauri::State<'_, AppState>,
    language: Option<String>,
) -> Result<i64, String> {
    let language = language.unwrap_or_else(|| "english".to_string());

    let now_ms = unix_ms();
    let session_id = {
        let db = state.db.lock().expect("db mutex poisoned");
        db.execute(
            "INSERT INTO sessions (created_at, source) VALUES (?1, 'mic')",
            [now_ms],
        )
        .map_err(|e| e.to_string())?;
        db.last_insert_rowid()
    };

    let (stop_tx, stop_rx) = oneshot::channel();
    state
        .pipelines
        .lock()
        .expect("pipelines mutex poisoned")
        .insert(session_id, stop_tx);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        run_pipeline(session_id, app_clone, stop_rx, language).await;
    });

    Ok(session_id)
}

/// Stop a running recording session.
///
/// Signals the pipeline to flush and exit, then sets `duration_ms` on the
/// session row.
#[tauri::command]
pub async fn stop_session(
    session_id: i64,
    state:      tauri::State<'_, AppState>,
) -> Result<(), String> {
    let stop_tx = state
        .pipelines
        .lock()
        .expect("pipelines mutex poisoned")
        .remove(&session_id);

    if let Some(tx) = stop_tx {
        // Ignore the error — the pipeline may have already exited.
        let _ = tx.send(());
    }

    let now_ms = unix_ms();
    let db = state.db.lock().expect("db mutex poisoned");
    db.execute(
        "UPDATE sessions SET duration_ms = ?1 - created_at WHERE id = ?2",
        rusqlite::params![now_ms, session_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
