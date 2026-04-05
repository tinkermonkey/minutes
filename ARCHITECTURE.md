# Minutes — Application Architecture

**Minutes** is the desktop (and future mobile) app built on top of speech-swift. It records conversations, identifies speakers across sessions, and produces a durable labeled transcript. This document covers the application architecture: audio capture, transcript storage, speaker registry UX, and the programmatic API.

## Scope

The speech-swift library and audio-server handle all ML inference: diarization, ASR, speaker registry, VAD. This document covers the **application layer** built on top of those primitives.

Core capabilities:
- Live mic capture with VAD gating → streaming transcription with speaker labels
- File upload → diarize + transcribe → stored session
- UX for viewing transcripts and managing speaker identities
- Local REST API for programmatic transcript access

## System Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Tauri App                            │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Web Frontend (React + Vite)            │    │
│  │                                                     │    │
│  │  ┌────────────────┐  ┌──────────────────────────┐  │    │
│  │  │ Transcript View│  │  Speaker Registry UI     │  │    │
│  │  │ (live + stored)│  │  (name, merge, samples)  │  │    │
│  │  └────────────────┘  └──────────────────────────┘  │    │
│  └──────────────────────────┬──────────────────────────┘    │
│                             │ Tauri commands + events       │
│  ┌──────────────────────────▼──────────────────────────┐    │
│  │              Rust Backend (Tauri)                   │    │
│  │  - mic capture (CPAL)                               │    │
│  │  - VAD gating before sending to audio-server        │    │
│  │  - HTTP client → audio-server                       │    │
│  │  - SQLite (rusqlite) — sessions, segments, samples  │    │
│  │  - local REST API (axum) on 127.0.0.1:8765          │    │
│  └──────────────┬──────────────────────────────────────┘    │
└─────────────────┼────────────────────────────────────────────┘
                  │ spawn as sidecar (macOS)
                  │ in-process plugin (iOS)
                  ▼
       ┌─────────────────────┐
       │   speech-swift      │
       │   audio-server      │
       │                     │
       │  POST /registry/sessions   (file upload)
       │  GET|PATCH /registry/speakers
       │  POST /registry/speakers/merge
       │  /v1/realtime (WebSocket, future)
       └─────────────────────┘
```

## Components

### App Shell: Tauri v2

Tauri v2 is the shell for desktop and mobile. Key properties:
- ~10MB overhead vs ~150MB for Electron
- Native macOS WebView (WKWebView) — no bundled Chromium
- speech-swift binary runs as a [Tauri sidecar](https://v2.tauri.app/develop/sidecar/), bundled in the `.app` and spawned/killed by Tauri lifecycle hooks
- Tauri v2 has official iOS and Android support, so the React frontend and Rust backend carry forward to mobile without rewrite (see [Mobile section](#mobile))

Audio capture lives in Rust (CPAL), not the frontend. Attempting mic capture from WKWebView introduces permission complexity and extra latency; the Rust layer owns the audio device directly.

### Frontend: React + Vite

- **flowbite-react** — Tailwind-based component library
- **TanStack Router** — type-safe routing between views
- **TanStack Query** — REST polling and cache management
- **TanStack Virtual** — virtualized list for long transcripts (avoids DOM blowup on multi-hour sessions)
- **Tauri event API** — receives live segment updates pushed from Rust as they arrive

The frontend never talks directly to the audio-server. All communication goes through Tauri commands and events, keeping the audio-server port internal.

### Rust Backend

Owns four concerns:

**1. Audio capture and VAD gating**

CPAL captures mic input as a ring buffer. A VAD gate (calling audio-server's VAD endpoint, or a small Rust VAD implementation) suppresses silence before sending anything upstream. This avoids posting every 200ms chunk regardless of speech activity.

**2. HTTP client to audio-server**

Two modes:

| Mode | Flow |
|---|---|
| Live capture | VAD-gated chunks → POST `/registry/sessions` or stream via `/v1/realtime` WS |
| File upload | User selects file → POST `/registry/sessions` (whole file, up to ~100MB / ~90 min WAV) |

For very long files (>60 min), implement a **sliding window** in Rust: 10-15 min windows with ~2 min overlap, stitch speaker IDs across windows using cosine similarity against the registry. This is app-layer logic, not speech-swift's concern. For MVP, process files whole and only add windowing when users actually hit the limit.

**3. SQLite persistence (rusqlite)**

The speech-swift audio-server owns the speaker registry (JSON files). The app layer owns a separate SQLite DB for session and transcript data:

```
sessions
  id, created_at, label, duration_ms, source (mic | file), file_path?

segments
  id, session_id, speaker_id, start_ms, end_ms, transcript_text

speaker_samples
  id, speaker_id, session_id, start_ms, end_ms, audio_path
```

`speaker_id` is the registry ID from speech-swift — the app DB never duplicates speaker identity data, it just references it.

**4. Local REST API (axum)**

Exposed on `127.0.0.1:8765` for programmatic access by external scripts or integrations:

```
GET  /sessions                     list sessions (paginated)
GET  /sessions/:id/transcript      full transcript with speaker labels
GET  /speakers                     speaker summary from registry + app DB
```

Read-only. No auth required for a single-user local tool.

### audio-server (speech-swift)

Unchanged from its current role. The app treats it as a black-box inference service. Relevant endpoints:

| Endpoint | Used for |
|---|---|
| `POST /registry/sessions` | Diarize + transcribe + resolve speakers for one audio chunk |
| `GET /registry/speakers` | List all registry speakers |
| `PATCH /registry/speakers/:id` | Assign display name, update notes |
| `POST /registry/speakers/merge` | Merge duplicate speaker identities |
| `DELETE /registry/speakers/:id` | Remove a speaker |

The `/v1/realtime` WebSocket path is the future home of true streaming (word-by-word transcript as audio arrives). For MVP, the request/response pattern via `/registry/sessions` is sufficient.

## Data Flow: Live Capture

```
mic (CPAL)
  → ring buffer
  → VAD gate: silence suppressed, speech_start triggers send
  → POST /registry/sessions with VAD-gated WAV chunk
  → ProcessedSession response: segments with speaker_id + transcript_text
  → write segments to SQLite
  → emit Tauri event → frontend updates transcript view live
```

## Data Flow: File Upload

```
user selects file
  → Rust reads file, checks size
  → if ≤ ~100MB: POST /registry/sessions whole file
  → if > ~100MB: sliding window (10-15 min, 2 min overlap), POST each window,
                 stitch speaker IDs across windows via registry cosine match
  → write session + segments to SQLite
  → navigate frontend to session transcript view
```

## Speaker Registry UX

The registry UI reads from both the audio-server (identity data) and the app SQLite (sample clips). Key interactions:

- **List view**: all speakers, labeled and unlabeled, with segment count and last-seen time
- **Speaker detail**: play audio samples from `speaker_samples` table to identify who it is, then assign a display name via `PATCH /registry/speakers/:id`
- **Merge**: select two speakers → `POST /registry/speakers/merge` — all segments in app DB are updated to the surviving speaker ID

## Mobile

Tauri v2 supports iOS and Android. The React frontend is 100% reusable. The Rust layer (SQLite, axum API, audio logic) compiles to both targets.

The only seam that changes per platform is how the Rust layer reaches the speech models:

| Platform | speech-swift integration |
|---|---|
| macOS | Sidecar binary, spawned at launch, HTTP on localhost |
| iOS | Native Tauri plugin (Swift, embedded `.xcframework`); CoreML models run on Neural Engine |
| Android | JNI bridge (Kotlin/C); MLX/CoreML swapped for ONNX or TFLite — significant model porting work |

speech-swift already anticipates iOS: `MemoryTier` in `SpeechCore` auto-detects device RAM and selects `.full`/`.standard`/`.constrained`/`.minimal` model configs. The CoreML model variants (Parakeet ASR, Kokoro TTS, Silero VAD, DeepFilterNet3) run on Neural Engine without changes. The MLX-based models (Qwen3-ASR, Qwen3-TTS) are macOS-only.

The Rust layer always addresses the backend via the same interface (`http://127.0.0.1:{port}/...`). Swapping the sidecar for an in-process plugin on iOS is transparent to the frontend and to the axum API layer.

Android is harder (no CoreML, model ONNX exports required) and should not constrain MVP decisions.

## Tech Stack Summary

| Layer | Choice | Reason |
|---|---|---|
| App shell | Tauri v2 | Light, native WebView, iOS path, Rust sidecar support |
| Frontend | React + Vite | Reusable on mobile, large ecosystem |
| UI components | flowbite-react | Tailwind-based component library |
| Routing | TanStack Router | Type-safe routing |
| Data fetching | TanStack Query | Cache + polling with minimal boilerplate |
| Long lists | TanStack Virtual | DOM-safe for hour-long transcripts |
| Audio capture | CPAL (Rust) | Direct device access, avoids WebView permission flow |
| Persistence | rusqlite (SQLite) | Local-first, zero server, simple schema |
| Local API | axum (Rust) | Lightweight, compiles to iOS/Android |
| ML inference | speech-swift audio-server | Existing, handles diarization + ASR + registry |
