---
name: Tauri Commands and Events
description: Known Tauri invoke() command names, their signatures, and Tauri event names used for live updates
type: project
---

## Tauri Commands

| Command | Signature | Notes |
|---|---|---|
| `get_speech_swift_status` | `() -> bool` | Returns true if speech-swift sidecar is reachable. Called once on mount with staleTime: Infinity. |
| `start_session` | `() -> i64` | Starts recording, returns session ID as number in TS. |
| `stop_session` | `(sessionId: i64) -> void` | Stops recording for the given session. Arg key is `sessionId` (camelCase). |

## Stage 3 — Speaker Registry Commands

| Command | Signature | Notes |
|---|---|---|
| `get_speakers` | `() -> Speaker[]` | Returns all speakers with stats. Snake_case fields: `id, speech_swift_id, display_name, notes, first_seen_at, last_seen_at, session_count`. |
| `rename_speaker` | `(speechSwiftId: number, name: string) -> void` | Arg keys are camelCase. |
| `merge_speakers` | `(srcId: number, dstId: number) -> void` | Merges src into dst. All segments re-attributed. |
| `delete_speaker` | `(speechSwiftId: number) -> void` | Deletes speaker; segments remain but lose attribution. |
| `get_speaker_sample_path` | `(speechSwiftId: number) -> string \| null` | Returns local file path for voice sample. Use `convertFileSrc()` from `@tauri-apps/api/core` to get a playable URL. |

## Tauri Events

| Event | Payload | Notes |
|---|---|---|
| `speech_swift_unreachable` | none | Emitted when health check fails. Sets query data for `speech_swift_status` to false. Handled in both `__root.tsx` and `record.tsx` (idempotent). |
| `segment_added` | `Segment` | Live transcript segment. Payload shape: `{id, session_id, speaker_id, speaker_label, display_name, start_ms, end_ms, transcript_text}` |
| `new_speaker` | `SpeakerNotification` | New speaker detected. Payload: `{id, speech_swift_id, display_name}`. Used to show NewSpeakerBanner. |

**Why:** These cannot be derived from reading the frontend code alone — they are the Rust-side contract.

**How to apply:** When adding any new invoke() call or listen() subscription, verify the command/event name matches exactly what Rust exposes. Use these as the ground truth for the current stage.
