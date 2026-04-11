---
name: Tauri Commands and Events (Stage 0)
description: Known Tauri invoke() command names, their signatures, and Tauri event names used for live updates
type: project
---

## Tauri Commands

| Command | Signature | Notes |
|---|---|---|
| `get_speech_swift_status` | `() -> bool` | Returns true if speech-swift sidecar is reachable. Called once on mount with staleTime: Infinity. |

## Tauri Events

| Event | Payload | Notes |
|---|---|---|
| `speech_swift_unreachable` | none | Emitted on startup if health check fails. Sets query data for `speech_swift_status` to false. |

**Why:** These cannot be derived from reading the frontend code alone — they are the Rust-side contract.

**How to apply:** When adding any new invoke() call or listen() subscription, verify the command/event name matches exactly what Rust exposes. Use these as the ground truth for Stage 0.
