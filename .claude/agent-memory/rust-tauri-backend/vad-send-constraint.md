---
name: webrtc-vad Send constraint and pipeline threading
description: webrtc_vad::Vad is !Send (raw C pointer); audio capture+VAD runs on a dedicated OS thread, chunks forwarded via mpsc to async consumer
type: project
---

`webrtc_vad::Vad` wraps `*mut Fvad` (a C FFI pointer) and is therefore `!Send`. Placing `Chunker` (which contains `VadClassifier` which contains `Vad`) inside a `tokio::select!` async block fails to compile with "future is not Send".

**Solution**: isolate all `!Send` work on a dedicated OS thread (`std::thread::spawn`). The thread owns:
- CPAL `Stream` (the `CaptureHandle._stream`)
- `Chunker` → `VadClassifier` → `Vad`

Completed WAV chunks are sent via `tokio::sync::mpsc::channel::<AudioChunk>` to the async task, which handles network (reqwest) + DB (rusqlite) work. The async task sends a stop signal to the capture thread via `std::sync::mpsc::channel::<()>`.

**Why not unsafe Send**: avoids lying to the compiler about a raw C pointer. The thread-based design is correct and self-documenting.

**How to apply**: whenever audio pipeline work must span an await boundary, keep VAD+chunker on the OS thread side of the mpsc boundary.
