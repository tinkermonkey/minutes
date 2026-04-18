# Agent Memory Index

- [Tauri Commands and Events](tauri_commands_and_events.md) — Stage 0–4 invoke/listen contracts: session CRUD, speaker registry, speech-swift status; segment_added/new_speaker events
- [Custom Hooks Inventory](custom_hooks.md) — useSpeechSwiftStatus, useStartSession, useStopSession, useSpeakers mutations, useSessions/useSession/useSegments — locations and arg shapes
- [Query Key Patterns](query_key_patterns.md) — TanStack Query key conventions; 6 keys established across stages; factory extraction pending growth
- [Routing Structure](routing_structure.md) — Manual TanStack Router tree in src/router.ts; 6 routes + root redirect; named exports throughout
- [Tailwind + Flowbite Setup](tailwind_setup.md) — Tailwind v4 via @tailwindcss/vite plugin; CSS-first config; no tailwind.config.js needed
- [Flowbite React Gotchas](flowbite_react_gotchas.md) — v0.12.x: Drawer sub-components are named exports; no Skeleton or isProcessing; use Spinner child for loading buttons
- [RecordingContext](recording_context.md) — All recording state + Tauri events live in RecordingContext (src/contexts/); record.tsx is a pure display component; top bar in __root.tsx holds controls
