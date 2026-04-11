---
name: Tauri trait imports
description: Non-obvious trait imports required for Tauri APIs to compile
type: reference
---

Traits that must be explicitly imported — the compiler will suggest them but they are easy to miss:

| API used | Required import |
|---|---|
| `handle.emit(event, payload)` | `use tauri::Emitter;` |
| `app.path().app_data_dir()` | `use tauri::Manager;` |
| `app.manage(state)` | `use tauri::Manager;` |
| `handle.state::<T>()` | `use tauri::Manager;` |

Both `Emitter` and `Manager` are commonly needed together in `lib.rs`.
