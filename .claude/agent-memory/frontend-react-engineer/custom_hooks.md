---
name: Custom Hooks Inventory
description: Reusable custom hooks created in this project, their locations, and what they encapsulate
type: project
---

## src/hooks/useSpeechSwiftStatus.ts

`useSpeechSwiftStatus()` — TanStack Query wrapper for `get_speech_swift_status` invoke. Returns `{ data: boolean | undefined, ... }`. Uses `staleTime: Infinity`. Query key: `['speech_swift_status']`.

## src/hooks/useSession.ts

`useStartSession()` — useMutation wrapping `invoke('start_session')`. Returns `number` (session ID).

`useStopSession()` — useMutation wrapping `invoke('stop_session', { sessionId })`. Takes `sessionId: number`.

**Note:** `stop_session` argument key must be `sessionId` (camelCase) to match the Tauri command parameter name.

## src/hooks/useSpeakers.ts

Stage 3 Speaker Registry hooks. Query key constant: `SPEAKERS_KEY = ['speakers'] as const` (exported).

`useSpeakers()` — query for all speakers. Returns `Speaker[]`.

`useRenameSpeaker()` — mutation `{ speechSwiftId, name }`. Invalidates `SPEAKERS_KEY` on success.

`useMergeSpeakers()` — mutation `{ srcId, dstId }`. Invalidates `SPEAKERS_KEY` on success.

`useDeleteSpeaker()` — mutation `(speechSwiftId: number)`. Invalidates `SPEAKERS_KEY` on success.

`useSpeakerSamplePath(speechSwiftId)` — query for voice sample path. Query key `['speaker_sample', speechSwiftId]`. Returns `string | null`. Disabled when `speechSwiftId <= 0`.

## src/hooks/useSessions.ts

Stage 4 Session History hooks.

`useSessions(filter: SessionFilter)` — paginated+filtered query. Query key `['sessions', filter]`. Uses `placeholderData: (prev) => prev` to avoid flash on filter changes.

`useSession(sessionId: number)` — single session query. Query key `['session', sessionId]`. Disabled when `sessionId <= 0`.

`useSegments(sessionId: number)` — segments with speaker names query. Query key `['segments', sessionId]`. Disabled when `sessionId <= 0`.

## src/hooks/useSearch.ts

Stage 5 Semantic Search hook.

`useSearch()` — useMutation wrapping `invoke('search_segments', { query, filters })`. Takes `{ query: string; filters: SearchFilters }`. Returns `SearchResult[]`. No query key (mutation, not query). No side-effect invalidations needed.
