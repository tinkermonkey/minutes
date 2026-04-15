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
    events::{self, SegmentEvent, SpeakerEvent},
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
/// Returns the segment IDs inserted so the caller can feed the slow-path
/// accumulator.
///
/// The DB mutex is acquired, all inserts are done synchronously, and the lock
/// is released before any `.await`. This is the critical invariant that keeps
/// `Mutex<Connection>` safe on the async executor.
async fn handle_chunk(
    session_id:  i64,
    chunk:       AudioChunk,
    app:         &tauri::AppHandle,
    embed_queue: &mut Vec<(i64, String)>,
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
    let response = match speech_swift::transcribe_chunk(&base_url, chunk.wav_bytes).await {
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
        events::emit_chunk_processed(app, events::ChunkProcessedEvent {
            start_ms: chunk.start_ms,
            response_ms,
            word_count,
            speaker_count,
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
/// and for each resulting segment finds the overlapping fast-path segment IDs
/// and updates their `speaker_id` / `status` in SQLite, firing
/// `speaker_resolved` events so the frontend can update in place.
async fn run_slow_path(
    base_url:    &str,
    app:         &tauri::AppHandle,
    accumulator: &mut crate::audio::SpeechAccumulator,
) {
    let Some(clip) = accumulator.drain() else { return };

    // Emit zeroed accumulator state after drain.
    events::emit_accumulator_updated(app, events::AccumulatorUpdatedEvent {
        speech_secs:  0.0,
        trigger_secs: crate::audio::accumulator::SPEECH_TRIGGER_SECS,
    });

    let wav_bytes = crate::audio::chunker::encode_wav(&clip.frames);
    let speech_secs = clip.frames.len() as f64 / 16_000.0;

    events::emit_slow_path_sent(app, events::SlowPathSentEvent {
        start_ms:         clip.clip_start_ms,
        end_ms:           clip.clip_end_ms,
        clip_speech_secs: speech_secs,
        sent_at_ms:       unix_ms() as u64,
    });

    let t0 = std::time::Instant::now();
    let response = match speech_swift::transcribe_chunk(base_url, wav_bytes).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("slow-path speech-swift error: {e}");
            events::emit_speech_swift_unreachable(app);
            return;
        }
    };
    let response_ms = t0.elapsed().as_millis() as u64;

    events::emit_slow_path_done(app, events::SlowPathDoneEvent {
        start_ms:      clip.clip_start_ms,
        response_ms,
        segment_count: response.segments.len() as u32,
    });

    // For each slow-path segment, match it back to overlapping fast-path DB
    // rows via the accumulator's chunk list and update their speaker_id.
    let state = app.state::<AppState>();
    let now_ms = unix_ms();
    let db = state.db.lock().expect("db mutex poisoned");

    for seg in &response.segments {
        let Some(speaker_id) = seg.speaker_id else { continue };

        // Convert slow-path segment times (seconds from clip start) to session
        // ms so we can match against the chunk timeline.
        let seg_start_ms = clip.clip_start_ms + (seg.start * 1000.0) as u64;
        let seg_end_ms   = clip.clip_start_ms + (seg.end   * 1000.0) as u64;

        // Collect fast-path DB row IDs whose chunk overlaps this segment.
        let overlapping_ids: Vec<i64> = clip.chunks.iter()
            .filter(|ch| ch.session_start_ms < seg_end_ms && ch.session_end_ms > seg_start_ms)
            .flat_map(|ch| ch.segment_ids.iter().copied())
            .collect();

        if overlapping_ids.is_empty() {
            continue;
        }

        // Upsert the speaker so it exists in our local DB before referencing it.
        let (speaker, _) = match db::speakers::upsert_speaker(&db, speaker_id, now_ms) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("slow-path upsert speaker error: {e}");
                continue;
            }
        };

        for segment_id in overlapping_ids {
            if let Err(e) = db::segments::update_segment_speaker(&db, segment_id, speaker_id) {
                eprintln!("slow-path update_segment_speaker error: {e}");
                continue;
            }
            events::emit_segment_speaker_resolved(app, events::SegmentSpeakerResolvedEvent {
                segment_id,
                speaker_id,
                speaker_label: seg.speaker_label.clone(),
                display_name:  speaker.display_name.clone(),
            });
        }
    }
    // `db` MutexGuard dropped here.
}

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
            .unwrap_or(VadMode::Silero)
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

        // Reset the meter on the frontend when capture ends.
        events::emit_audio_level(&level_app, 0.0);
        // chunk_tx dropped here, closing the channel and signalling the async
        // consumer to finish.
    });

    // Async consumer: handles network + DB work.
    let mut embed_queue: Vec<(i64, String)> = Vec::new();
    let mut accumulator = crate::audio::SpeechAccumulator::new();
    let base_url = app_handle.state::<AppState>().speech_swift_url.clone();

    /// Append a processed chunk to the accumulator and trigger the slow path
    /// if enough speech has built up or the accumulator has gone idle.
    async fn maybe_run_slow_path(
        base_url:      &str,
        app:           &tauri::AppHandle,
        accumulator:   &mut crate::audio::SpeechAccumulator,
        speech_frames: Vec<f32>,
        start_ms:      u64,
        end_ms:        u64,
        segment_ids:   Vec<i64>,
    ) {
        accumulator.append(speech_frames, start_ms, end_ms, segment_ids);
        events::emit_accumulator_updated(app, events::AccumulatorUpdatedEvent {
            speech_secs:  accumulator.speech_secs,
            trigger_secs: crate::audio::accumulator::SPEECH_TRIGGER_SECS,
        });

        if accumulator.should_trigger()
            || accumulator.should_flush_for_inactivity(std::time::Duration::from_secs(20))
        {
            run_slow_path(base_url, app, accumulator).await;
        }
    }

    tokio::select! {
        _ = stop_rx => {
            // Tell the capture thread to flush and exit.
            let _ = thread_stop_tx.send(());
            // Drain any remaining chunks the thread flushed before exiting.
            while let Some(chunk) = chunk_rx.recv().await {
                let speech_frames = chunk.speech_frames.clone();
                let start_ms = chunk.start_ms;
                let end_ms   = chunk.end_ms;
                let ids = handle_chunk(session_id, chunk, &app_handle, &mut embed_queue).await;
                maybe_run_slow_path(
                    &base_url, &app_handle, &mut accumulator,
                    speech_frames, start_ms, end_ms, ids,
                ).await;
            }
            // Final drain at session end.
            if !accumulator.is_empty() {
                run_slow_path(&base_url, &app_handle, &mut accumulator).await;
            }
        }
        _ = async {
            while let Some(chunk) = chunk_rx.recv().await {
                let speech_frames = chunk.speech_frames.clone();
                let start_ms = chunk.start_ms;
                let end_ms   = chunk.end_ms;
                let ids = handle_chunk(session_id, chunk, &app_handle, &mut embed_queue).await;
                maybe_run_slow_path(
                    &base_url, &app_handle, &mut accumulator,
                    speech_frames, start_ms, end_ms, ids,
                ).await;
            }
        } => {
            // Capture thread exited on its own (e.g. device disconnected).
            // Final drain at session end.
            if !accumulator.is_empty() {
                run_slow_path(&base_url, &app_handle, &mut accumulator).await;
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
    app:   tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
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
        run_pipeline(session_id, app_clone, stop_rx).await;
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
