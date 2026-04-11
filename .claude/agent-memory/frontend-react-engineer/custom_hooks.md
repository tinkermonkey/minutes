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
