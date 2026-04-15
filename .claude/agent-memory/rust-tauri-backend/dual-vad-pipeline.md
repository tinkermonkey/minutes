---
name: Dual VAD pipeline (Silero + WebRTC)
description: VadBackend trait, SileroBackend (tract-onnx), WebRtcBackend, generic VadClassifier<B>, DynChunker enum, fallback logic
type: project
---

## Architecture

`vad.rs` defines a `VadBackend` trait with three methods: `frame_size()`, `classify_frame()`, `reset()`. Two backends implement it:

- `WebRtcBackend` (in `vad.rs`) — wraps `webrtc_vad::Vad`, frame_size=160 (10ms). `!Send` due to raw C pointer. `reset()` is a no-op.
- `SileroBackend` (in `silero.rs`) — uses `tract-onnx 0.21`, frame_size=512 (32ms). `Send+Sync`. Carries LSTM `h`/`c` state between frames; `reset()` zeros them. Model loaded from path at construction.

`VadClassifier<B: VadBackend>` is generic and computes silence thresholds dynamically from `backend.frame_size()`:
- silence flush threshold = `500ms / frame_duration_ms` (50 frames for WebRTC, 15 for Silero)
- silence pad = 200ms worth of samples (constant in samples)

`Chunker<B: VadBackend>` reads `vad.frame_size()` at runtime — no hardcoded 160.

`DynChunker` enum in `audio/mod.rs` unifies the two concrete chunker types so `commands/mod.rs` has one type to hold on the OS thread:
```rust
enum DynChunker { WebRtc(Chunker<WebRtcBackend>), Silero(Chunker<SileroBackend>) }
```

## Default behavior

`VadMode::Silero` is the default. Model loaded from `app_handle.path().resource_dir()?.join("silero_vad.onnx")`. If the path is unavailable or `SileroBackend::new()` fails, `DynChunker` falls back to `WebRtcBackend` with an `eprintln!` warning.

## Silero ONNX inputs/outputs

- Inputs: `input` f32[1,512], `sr` i64 scalar (16000), `h` f32[2,1,64], `c` f32[2,1,64]
- Outputs: `output` f32[1,1] (probability), `hn` f32[2,1,64], `cn` f32[2,1,64]
- Voiced if `output[0] > 0.5`
- State `hn`/`cn` carried forward each frame via `Tensor::clone().into_tensor()`

## Resource file

`src-tauri/resources/silero_vad.onnx` (2.2MB, v5.1.2) is listed in `tauri.conf.json` under `bundle.resources`.

**Why:** Silero v5 is more accurate than WebRTC VAD for natural speech, especially with background noise. tract-onnx is pure Rust and Send+Sync, avoiding the OnnxRuntime threading issues that caused the original Silero implementation to be ripped out.

**How to apply:** When adding a new session or changing session start logic, construct `DynChunker::new(VadMode::Silero, &model_path)` in the OS thread spawn closure. The OS thread owns it (both backends are safe on OS threads; only WebRtcBackend is also !Send on async executors).
