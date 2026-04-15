pub mod devices;
pub mod health;
pub mod search;
pub mod sessions;
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
    wav_bytes: Vec<u8>,
    start_ms:  u64,
    end_ms:    u64,
}

/// Send one WAV chunk to the audio-server, persist results to SQLite, compute
/// embeddings, and fire Tauri events.
///
/// The DB mutex is acquired, all inserts are done synchronously, and the lock
/// is released before any `.await`. This is the critical invariant that keeps
/// `Mutex<Connection>` safe on the async executor.
async fn handle_chunk(
    session_id:  i64,
    chunk:       AudioChunk,
    app:         &tauri::AppHandle,
    embed_queue: &mut Vec<(i64, String)>,
) {
    let audio_path = save_wav_chunk(app, session_id, chunk.start_ms, &chunk.wav_bytes);

    let state = app.state::<AppState>();
    let base_url = state.speech_swift_url.clone();

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
            return;
        }
    };
    let response_ms = t0.elapsed().as_millis() as u64;

    // --- DB work: acquire lock, do all inserts, release before any await. ---
    let now_ms = unix_ms();
    let events_to_emit: Vec<(SegmentEvent, Option<SpeakerEvent>)> = {
        let db = state.db.lock().expect("db mutex poisoned");

        let mut events = Vec::with_capacity(response.segments.len());

        for seg in &response.segments {
            let (speaker, is_new) =
                match db::speakers::upsert_speaker(&db, seg.speaker_id, now_ms) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("upsert speaker error: {e}");
                        continue;
                    }
                };

            let segment_id = match db::segments::insert_segment(
                &db,
                &NewSegment {
                    session_id,
                    speaker_id: seg.speaker_id,
                    start_ms:   (seg.start * 1000.0) as i64,
                    end_ms:     (seg.end   * 1000.0) as i64,
                    transcript_text: seg.transcript.clone(),
                },
            ) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("insert segment error: {e}");
                    continue;
                }
            };

            let _ = db::samples::insert_speaker_sample(
                &db,
                speaker.id,
                session_id,
                (seg.start * 1000.0) as i64,
                (seg.end   * 1000.0) as i64,
                &audio_path,
            );

            // Try embedding synchronously; on failure, defer to the drain queue.
            match embed::embed(&seg.transcript) {
                Ok(vec) => {
                    let _ = db::segments::insert_segment_embedding(&db, segment_id, &vec);
                }
                Err(_) => {
                    embed_queue.push((segment_id, seg.transcript.clone()));
                }
            }

            let speaker_event = if is_new || speaker.display_name.is_none() {
                Some(SpeakerEvent {
                    id:              speaker.id,
                    speech_swift_id: speaker.speech_swift_id,
                    display_name:    speaker.display_name.clone(),
                })
            } else {
                None
            };

            events.push((
                SegmentEvent {
                    id:              segment_id,
                    session_id,
                    speaker_id:      seg.speaker_id,
                    speaker_label:   seg.speaker_label.clone(),
                    display_name:    speaker.display_name.clone(),
                    start_ms:        (seg.start * 1000.0) as i64,
                    end_ms:          (seg.end   * 1000.0) as i64,
                    transcript_text: seg.transcript.clone(),
                },
                speaker_event,
            ));
        }

        events
        // `db` MutexGuard dropped here — lock released before any await.
    };

    // Emit chunk_processed now that we have response stats and DB work is done.
    {
        use std::collections::HashSet;
        let word_count: u32 = response.segments.iter()
            .map(|s| s.transcript.split_whitespace().count() as u32)
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

    // Resolve the Silero model path from the Tauri resource directory.
    // Falls back to WebRTC VAD if the path cannot be determined.
    let model_path = app_handle
        .path()
        .resource_dir()
        .ok()
        .map(|d| d.join("resources").join("silero_vad.onnx"));

    let level_app = app_handle.clone();

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

        // Build the DynChunker: Silero by default, WebRTC fallback.
        let mut chunker = match model_path {
            Some(ref p) => DynChunker::new(VadMode::Silero, p),
            None => {
                eprintln!("VAD: resource_dir unavailable, falling back to WebRTC VAD");
                DynChunker::new(VadMode::WebRtc, std::path::Path::new(""))
            }
        };
        let mut last_level_emit = std::time::Instant::now();

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
                if let Some((wav, start, end)) = chunker.push_samples(&samples) {
                    let _ = chunk_tx.blocking_send(AudioChunk {
                        wav_bytes: wav,
                        start_ms:  start,
                        end_ms:    end,
                    });
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
        if let Some((wav, start, end)) = chunker.flush() {
            let _ = chunk_tx.blocking_send(AudioChunk {
                wav_bytes: wav,
                start_ms:  start,
                end_ms:    end,
            });
        }

        // Reset the meter on the frontend when capture ends.
        events::emit_audio_level(&level_app, 0.0);
        // chunk_tx dropped here, closing the channel and signalling the async
        // consumer to finish.
    });

    // Async consumer: handles network + DB work.
    let mut embed_queue: Vec<(i64, String)> = Vec::new();

    tokio::select! {
        _ = stop_rx => {
            // Tell the capture thread to flush and exit.
            let _ = thread_stop_tx.send(());
            // Drain any remaining chunks the thread flushed before exiting.
            while let Some(chunk) = chunk_rx.recv().await {
                handle_chunk(session_id, chunk, &app_handle, &mut embed_queue).await;
            }
        }
        _ = async {
            while let Some(chunk) = chunk_rx.recv().await {
                handle_chunk(session_id, chunk, &app_handle, &mut embed_queue).await;
            }
        } => {
            // Capture thread exited on its own (e.g. device disconnected).
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
