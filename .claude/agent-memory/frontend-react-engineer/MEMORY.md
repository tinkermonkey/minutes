# Agent Memory Index

- [Tauri Commands and Events](tauri_commands_and_events.md) — Stage 0–2 invoke/listen contracts: `start_session`, `stop_session`, `get_speech_swift_status`; `segment_added`, `new_speaker`, `speech_swift_unreachable` events
- [Custom Hooks Inventory](custom_hooks.md) — `useSpeechSwiftStatus`, `useStartSession`, `useStopSession` — locations and arg shapes
- [Query Key Patterns](query_key_patterns.md) — TanStack Query key conventions; `['speech_swift_status']` established; factory extraction pending growth
- [Routing Structure](routing_structure.md) — Manual TanStack Router tree in src/router.ts; 4 routes + root redirect; named exports throughout
- [Tailwind + Flowbite Setup](tailwind_setup.md) — Tailwind v4 via @tailwindcss/vite plugin; CSS-first config; no tailwind.config.js needed
