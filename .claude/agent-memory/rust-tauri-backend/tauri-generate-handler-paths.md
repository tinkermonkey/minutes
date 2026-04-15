---
name: tauri generate_handler! path requirements
description: generate_handler! requires fully-qualified module paths; pub use re-exports cause E0433 resolution failures
type: reference
---

`tauri::generate_handler!` expands to internal symbols like `__cmd__<fn_name>` resolved via the path you provide. A `pub use` re-export in a parent module does NOT satisfy this — the macro looks up the path literally.

**Rule:** Always use the canonical module path in `generate_handler!`:

```rust
// CORRECT
commands::settings::get_vad_mode,

// WRONG — causes E0433: could not find `__cmd__get_vad_mode` in `commands`
commands::get_vad_mode,  // even with `pub use settings::get_vad_mode` in commands/mod.rs
```

**Pattern in use:** Device commands follow `commands::devices::get_preferred_device`, settings commands follow `commands::settings::get_vad_mode`. All sub-module commands must use the full two-level path `commands::<submodule>::<fn>`.
