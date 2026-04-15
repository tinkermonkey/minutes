---
name: Query Key Patterns
description: TanStack Query key conventions established in the codebase
type: project
---

## Established Query Keys

| Key | Purpose |
|---|---|
| `['speech_swift_status']` | Speech-swift reachability boolean. staleTime: Infinity — only updated via Tauri event invalidation or setQueryData. |
| `['speakers']` | All speakers list. Exported as `SPEAKERS_KEY` constant from `useSpeakers.ts`. Invalidated by rename/merge/delete mutations. |
| `['speaker_sample', speechSwiftId]` | Voice sample file path for one speaker. |
| `['sessions', filter]` | Paginated session list. Full filter object as second element — deep equality via TanStack Query. |
| `['session', sessionId]` | Single session by numeric ID. |
| `['segments', sessionId]` | All segments for a session. |
| `['audio_devices']` | List of audio input devices. Invalidated by `set_audio_device` mutation. |
| `['preferred_device']` | User's saved audio device preference. Invalidated by `set_audio_device` mutation. |
| `['speech_swift_port']` | speech-swift port string. Optimistic update via `setQueryData` on field change; persisted on blur. |
| `['vad_mode']` | VAD backend: `"Silero"` or `"WebRtc"`. Invalidated by `set_vad_mode` mutation. |

## Convention

Query keys are plain arrays of string literals for now (no factory functions yet). As more queries are added, extract to a `queryKeys.ts` constants file when there are 3+ keys sharing a prefix.

**Why:** Inline keys are fine for Stage 0 but will need centralizing once sessions, speakers, and search queries are added.

**How to apply:** When adding new queries, use `['resource', ...params]` shape. Create `src/lib/queryKeys.ts` when the number of keys grows beyond a few.
