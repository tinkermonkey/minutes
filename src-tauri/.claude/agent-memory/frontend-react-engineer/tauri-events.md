---
name: Tauri event contracts
description: All Tauri event names, payload shapes, and how they are consumed in the frontend
type: project
---

## Events emitted by the Rust backend

### `segment_added`
Fired when a VAD chunk produces a transcript segment. Fast-path; speaker may not be resolved yet.

```typescript
{
  id:              number;
  session_id:      number;
  speaker_id:      number | null;   // null = pending
  speaker_label:   string | null;   // null = pending
  display_name:    string | null;
  start_ms:        number;
  end_ms:          number;
  transcript_text: string;
  status:          'pending' | 'confirmed';
}
```

Consumed in `routes/record.tsx`: appended to `segments` state.

### `segments_replaced` (replaces `speaker_resolved` as of 2026-04-16)
Fired by the slow-path long-clip diarization when a batch of pending segments is atomically replaced with fully-resolved segments.

```typescript
{
  removed_ids: number[];   // Segment.id values to remove from state
  added:       Segment[];  // replacement segments (fully resolved, status 'confirmed')
}
```

Consumed in `RecordingContext.tsx`: filters out `removed_ids`, appends non-empty `added` segments. No buffering needed — `segments_replaced` always arrives after its corresponding `segment_added` events.

**`speaker_resolved` no longer exists.** Do not reference it.

### `new_speaker`
Fired when speech-swift registers a previously-unseen speaker. Triggers `NewSpeakerBanner`.

Payload type: `SpeakerNotification { id, speech_swift_id, display_name }`

### `speech_swift_unreachable`
Fired when the audio-server cannot be reached. Sets `['speech_swift_status']` query cache to `false`.

Payload: `void`

### `chunk_sent`
Fired when a VAD chunk is dispatched to the audio-server.

```typescript
{ start_ms: number; end_ms: number; sent_at_ms: number; }
```

Consumed in two places:
- `routes/record.tsx`: appends to `pipelineEntries` for the pipeline log.
- `hooks/useVadState.ts`: debounce-based VAD state — sets `vadActive = true` on each event, resets to `false` after 500ms silence. VAD state is passed as `vadActive` prop to `AudioMeter` and `AudioLevelGraph`.

### `chunk_processed`
Fired when the audio-server returns a response for a sent chunk. Matched to `chunk_sent` by `start_ms`.

```typescript
{ start_ms: number; response_ms: number; word_count: number; speaker_count: number; }
```

### `audio_level`
Fired every ~50 ms (20 fps) by the Rust mic-capture loop while a recording session is active.

Payload type: `number` — f32 RMS value in `[0.0, 1.0]`.

Consumed in two places:
- `components/AudioMeter.tsx`: applies `sqrt(payload) * 2.5` scaling and drives a 20-segment discrete bar via `useState`. Also renders a single VAD indicator segment (blue) driven by `vadActive` prop.
- `components/AudioLevelGraph.tsx`: clamps to `[0, 1]`, pushes `{ level, vad }` samples into a `useRef<Sample[]>` rolling buffer (200 samples = 10 s), then imperatively redraws a `<canvas>` — zero `setState` on the hot path. VAD-active samples are tinted with `rgba(59,130,246,0.15)` behind the waveform.

---

**Why:** Dual-stream architecture: VAD chunks arrive fast with no speaker (pending), a later long-clip pass atomically replaces them via `segments_replaced`. The replacement model is simpler than the old `speaker_resolved` patch-in-place: remove old IDs, append new segments.

**How to apply:** Always handle `speaker_id: null` and `status: 'pending'` in any component that renders segments. Never assume a segment has a speaker when it first arrives.
