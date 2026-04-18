---
name: RecordingContext — persistent recording state
description: All recording state and Tauri event subscriptions live in RecordingContext, not in the record route
type: project
---

## Location

`src/contexts/RecordingContext.tsx`

## What it contains

All recording state lifted out of `record.tsx` so it survives navigation:

- `sessionState` (idle / recording / stopping), `language`, `segments`, `elapsed`, `pipelineEntries`, `accumulatorSecs`, `accumulatorTrigger`, `showNewSpeakerBanner`
- `vadActive` from `useVadState(sessionState.status === 'recording')`
- All `useTauriEvent` subscriptions: `segment_added`, `speaker_resolved`, `new_speaker`, `speech_swift_unreachable`, `chunk_sent`, `chunk_processed`, `accumulator_updated`, `slow_path_sent`, `slow_path_done`
- `handleStart` / `handleStop` async functions
- `retryHealth` mutation (`retry_health_check` invoke)
- `updateSpeakerName(speakerId: number, displayName: string)` — patches `display_name` on all segments with that `speaker_id`; called by `SessionSpeakersSidebar` after a successful rename so the sidebar stays reactive without waiting for a `speaker_resolved` event
- Window title `useEffect`

## Exports

`RecordingProvider` — wraps the root layout; must sit inside `QueryClientProvider`.

`useRecording()` — throws if used outside the provider.

`SessionState` — discriminated union type (also exported for consumers).

## Where it is mounted

`src/routes/__root.tsx` — `RootLayout` wraps `RootLayoutInner` in `<RecordingProvider>`. Recording controls (RecordButton, SessionStatusBadge, AudioMeter, AccumulatorBar, language select) are rendered in a persistent 56 px top bar in `RootLayoutInner`, reading from `useRecording()`.

## record.tsx

Stripped to a pure display component. Reads from `useRecording()`. No local state, no effects, no mutations. Still guards with `useSpeechSwiftStatus()` and shows `SpeechSwiftErrorPanel` when speech-swift is unreachable.

## Why

Navigation away from `/record` previously unmounted the route, losing all recording state and causing duplicate event handler registrations on return. Lifting to context makes the recording session persistent across the entire session lifetime.
