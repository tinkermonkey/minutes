---
name: Custom hooks catalog
description: Reusable custom hooks in src/hooks/, their locations, and what they do
type: project
---

## `useVadState(enabled: boolean): boolean`
**Location:** `src/hooks/useVadState.ts`

Derives `vadActive` boolean from `chunk_sent` Tauri events. When `chunk_sent` fires, sets `vadActive = true`; after 500ms with no new event, resets to `false`. When `enabled` is false, the hook no-ops (safe to call with `isRecording` derived from session state).

Used in `routes/record.tsx` and passed as `vadActive` prop to `AudioMeter` and `AudioLevelGraph`.

## `useTauriEvent<T>(event, handler): void`
**Location:** `src/hooks/useTauriEvent.ts`

Subscribes to a Tauri event for the lifetime of the component. Handler is stored in a ref so it always sees fresh closure values without triggering re-subscribe. Handles StrictMode double-invoke race via a `cancelled` flag.

**Why:** Avoids the common bug of stale closures in Tauri event listeners without adding event names to the dependency array.
