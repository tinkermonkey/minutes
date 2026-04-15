# Audio Accumulation for Live Speaker Recognition

## Why This Document Exists

The live capture pipeline in Minutes currently sends short VAD-gated chunks to
`POST /registry/sessions` and expects speaker-labeled transcripts back. This
works fine for ASR but **not** for speaker recognition. The speech-swift speaker
registry requires substantially more audio per speaker than a typical VAD chunk
provides. This mismatch causes the registry to create a new speaker identity
for every chunk, producing an exploding registry with no useful matches.

This document defines the dual-stream accumulation design that resolves the
tension and specifies exactly what the Rust backend is responsible for.

---

## The Fundamental Tension

| Goal | Chunk requirement |
|---|---|
| Low-latency transcription | 2–5s VAD utterances |
| Centroid enrollment (min) | ≥ 2s of a single speaker's speech per chunk |
| Reliable embedding quality | ≥ 5–10s of a single speaker's speech |
| Stable speaker matching | ≥ 30s cumulative speech per speaker |

A single chunk size cannot satisfy all of these. Sending 30s chunks gives good
speaker recognition but unacceptable transcription latency. Sending 2s chunks
gives good latency but the registry produces noise.

**Solution: run two parallel streams from the same audio.**

---

## Division of Responsibility

### speech-swift / audio-server

Stateless inference primitives. It answers: *"Who is speaking in this audio
clip, and what are they saying?"* It has no concept of a conversation timeline,
pending utterances, or re-processing triggers.

Relevant endpoints:

| Endpoint | Used for |
|---|---|
| `POST /registry/sessions` | Diarize + transcribe + resolve speakers for one clip |
| `GET /registry/speakers` | Query the registry |
| `PATCH /registry/speakers/:id` | Assign display name |
| `POST /registry/speakers/merge` | Merge duplicate identities |

The registry persists speaker identities and centroids across sessions. It does
not know which utterances are pending resolution or when to re-process.

### Minutes Rust backend

Owns conversation state. It is responsible for:

- Deciding when enough audio has accumulated to trigger speaker recognition
- Keeping the audio buffer for re-processing
- Mapping long-clip diarization results back to buffered short-clip utterances
- Retroactively updating segments in SQLite when speaker IDs are resolved
- Pushing updates to the frontend via Tauri events

**The trigger logic belongs here because the signal — "I have accumulated enough
speech" — requires knowledge of the conversation timeline that only this layer
has.**

---

## Dual-Stream Architecture

```
Mic (CPAL)
  │
  ├──► Fast Path  (per VAD utterance, ~2–5s)
  │      VAD → ASR via POST /registry/sessions
  │      write segment to SQLite: speaker_id = NULL, status = "pending"
  │      emit Tauri event: provisional transcript line appears in UI
  │
  └──► Slow Path  (accumulation buffer)
         append VAD speech frames to speech_buffer (silence stripped)
         when speech_buffer reaches threshold:
           POST /registry/sessions with long clip
           receive speaker IDs
           map back to pending segments by timestamp
           update SQLite: speaker_id = resolved, status = "confirmed"
           emit Tauri event: UI updates speaker labels retroactively
```

The fast path gives the user text immediately. The slow path gives the user
speaker identity when there is enough audio to be confident.

---

## Accumulation Buffer Design

### What to accumulate

Accumulate **VAD-speech frames only** — strip silence before appending to the
buffer. This maximises the acoustic information density in the long clip and
avoids wasting diarization capacity on silence.

The VAD gate you already have for the fast path produces exactly the right
frames. Tap into the same output.

### When to trigger

Trigger long-clip processing when **any** of these conditions is met:

1. **Speech duration threshold:** accumulated speech ≥ 30 seconds
2. **Session end:** flush the remaining buffer regardless of duration
3. **Inactivity timeout:** no new speech for 10s (speaker may have left)

Do **not** trigger on wall-clock time — 30s of silence-padded time is not 30s
of speech. Measure speech frames, not elapsed time.

### Overlap between long clips

When you trigger and reset the buffer, **retain the last 10 seconds of speech**
as the start of the next buffer. This ensures speakers near the clip boundary
get consistent IDs across clips and avoids a cold start for the next enrollment
cycle.

```
Long clip 1: [════════════════════════════════════════]
Long clip 2:                           [══════════════════════════════════════]
                                       ↑ 10s overlap retained
```

### Buffer sizing

At 16 kHz Float32, 30s of speech = ~1.9 MB. This is cheap to hold in memory.
Keep the raw float samples, not a WAV file — encode to WAV only when sending
to audio-server.

---

## Session State

Add this state to the Rust struct that manages a live capture session:

```rust
struct SpeechAccumulator {
    // VAD-filtered speech frames, no silence
    speech_frames: Vec<f32>,

    // Total seconds of speech accumulated since last long-clip trigger
    speech_duration_secs: f32,

    // Wall-clock offset of speech_frames[0] relative to session start,
    // for mapping back to utterance timestamps
    buffer_start_offset_secs: f64,

    // Sample rate (always 16000 for speech-swift)
    sample_rate: u32,
}
```

Each VAD utterance also produces a segment record in SQLite:

```sql
-- Add these columns to the segments table
status        TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'confirmed'
utterance_id  TEXT NOT NULL,                    -- UUID, for retroactive update
chunk_start   REAL NOT NULL,                    -- session-relative start (seconds)
chunk_end     REAL NOT NULL,                    -- session-relative end (seconds)
```

---

## Fast Path: Per-Utterance Processing

For each VAD utterance:

1. Encode utterance frames as WAV
2. `POST /registry/sessions` — get ASR transcript + provisional speaker ID
   - The registry may return a matched speaker if one is enrolled from a prior
     session. Accept it.
   - If the registry creates a new speaker (no prior match), treat it as
     provisional — it may be merged or reassigned after the long-clip pass.
3. Write to SQLite:
   ```
   speaker_id = <registry result, or NULL if unconfident>
   status = "pending"
   utterance_id = <uuid>
   chunk_start / chunk_end = <session-relative timestamps>
   ```
4. Emit Tauri event to frontend with provisional speaker label
5. Append utterance speech frames to `SpeechAccumulator`

---

## Slow Path: Long-Clip Processing

When `speech_duration_secs >= 30.0` or session ends:

1. Encode `speech_frames` as WAV (16 kHz, mono, Float32 → PCM16)
2. `POST /registry/sessions` with the long clip
   - This is the enrollment-quality call. The registry will enroll new
     centroids, update existing ones, and return high-confidence speaker IDs.
3. Receive `ProcessedSession`:
   ```json
   {
     "segments": [
       { "speaker_id": 3, "speaker_label": "Alice", "start": 0.0, "end": 4.2 },
       { "speaker_id": 7, "speaker_label": "Speaker_7", "start": 4.5, "end": 9.1 }
     ]
   }
   ```
4. Map long-clip segments back to buffered utterances:
   ```
   for each long_clip_segment:
     long_clip_start = buffer_start_offset_secs + segment.start
     long_clip_end   = buffer_start_offset_secs + segment.end
     find all utterances where chunk_start..chunk_end overlaps long_clip_start..long_clip_end
     update those utterances: speaker_id = segment.speaker_id, status = "confirmed"
   ```
5. Write updates to SQLite
6. Emit Tauri events for each resolved utterance:
   ```json
   { "utterance_id": "...", "speaker_id": 3, "display_name": "Alice" }
   ```
7. Reset accumulator: retain last 10s of frames, set `buffer_start_offset_secs`
   to the new start, set `speech_duration_secs = 10.0`

---

## Timestamp Mapping

The long clip is built from **silence-stripped** frames, so its internal
timestamps do not directly correspond to session wall-clock time. You need to
maintain a mapping from long-clip frame offset to session time.

The simplest approach: keep a `Vec<(clip_start_frame, session_offset_secs)>` in
the accumulator. Each time you append a VAD utterance, record where it starts
in the buffer and what session time it corresponds to. Then during retroactive
mapping:

```rust
fn clip_time_to_session_time(
    clip_secs: f64,
    index: &[(usize, f64)],  // (clip_frame_offset, session_secs)
    sample_rate: u32,
) -> f64 {
    let clip_frame = (clip_secs * sample_rate as f64) as usize;
    // find the index entry whose clip_frame_offset is <= clip_frame
    // interpolate linearly
}
```

This is O(n) but n is small (one entry per utterance).

---

## Frontend UX

The UI needs to handle two states for each transcript line:

| State | Display |
|---|---|
| `pending` | Show transcript text immediately. Speaker label shown as "..." or a loading indicator. |
| `confirmed` | Speaker label appears, possibly different from any provisional guess. |

When a `speaker_resolved` Tauri event arrives, find the transcript line by
`utterance_id` and update the speaker label in place. This should be a smooth
visual update, not a flash or re-render of the full transcript.

If the confirmed speaker differs from a provisional guess that was shown to the
user, use a brief highlight or fade to draw attention to the correction.

---

## Edge Cases

### Speaker appears only in one short utterance

If a speaker only says one short thing (< 2s) across the entire session, the
registry will never enroll a centroid for them. Their segment stays `pending`
and gets flushed at session end with whatever the long-clip diarizer assigns.
This is acceptable — you did your best with the audio available.

### Two speakers overlap (crosstalk)

The diarizer assigns each frame to one speaker. Overlapping speech gets assigned
to whichever speaker has the higher probability. The resulting segments will be
slightly inaccurate near the overlap. Nothing to handle specially — this is a
known limitation of single-channel diarization.

### Session ends before 30s of speech

Flush the buffer at session end regardless of duration. The quality of the
speaker IDs will be lower but it's better than returning nothing. Lower the
enrollment quality threshold for end-of-session flushes if needed — the
speech-swift API accepts a `threshold` query parameter on `POST
/registry/sessions` to override the default 0.75 cosine similarity threshold.

### New speaker appears mid-session (after first long clip)

They accumulate in the next buffer window. Previously enrolled speakers match
immediately via the registry. The new speaker's utterances are `pending` until
the next slow-path trigger. This is the expected behaviour — the lag is bounded
by the accumulation threshold (≤ 30s of speech).

### Registry creates duplicate speakers

This can happen if the same person speaks in two sessions with different audio
conditions (noise, phone vs. microphone). The `POST /registry/speakers/merge`
endpoint handles this — surface it in the speaker registry UI so the user can
fix it manually.

---

## Parameters to Expose

These should be configurable in the Rust backend, not hardcoded:

| Parameter | Default | Notes |
|---|---|---|
| `speech_accumulation_threshold_secs` | 30.0 | Seconds of speech before slow-path trigger |
| `overlap_secs` | 10.0 | Speech seconds retained between long clips |
| `min_enrollment_quality_secs` | 2.0 | Must match speech-swift registry threshold |
| `inactivity_flush_secs` | 10.0 | Trigger flush after N seconds of silence |
| `long_clip_similarity_threshold` | 0.75 | Passed as `?threshold=` to audio-server |

---

## What speech-swift Handles vs. What Minutes Handles

| Concern | speech-swift | Minutes |
|---|---|---|
| VAD frame detection | ✓ (available, not required) | ✓ (CPAL + VAD gate already exists) |
| ASR transcription | ✓ | — |
| Speaker diarization | ✓ | — |
| Registry centroid storage | ✓ | — |
| Speaker identity persistence across sessions | ✓ | — |
| Utterance buffer with timestamps | — | ✓ |
| Accumulation threshold logic | — | ✓ |
| Retroactive speaker label assignment | — | ✓ |
| SQLite segment storage | — | ✓ |
| Frontend Tauri events | — | ✓ |
| Re-processing trigger | — | ✓ |

The speech-swift audio-server does not need any changes to support this design.
All accumulation and re-processing logic lives in the Minutes Rust backend.
