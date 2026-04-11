# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Minutes** is a Tauri v2 desktop app that records conversations, identifies speakers across sessions, and produces durable labeled transcripts. It is built on top of **speech-swift** (an external ML inference service) which handles all diarization, ASR, speaker registry, and VAD — the app layer is responsible for everything above that.

## Folder Structure

```
minutes/
├── src/                        # React + Vite frontend
│   ├── main.tsx                # App entry point
│   └── App.tsx                 # Root component
├── src-tauri/                  # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs             # Tauri app entry
│   │   └── lib.rs              # Tauri commands, events, state
│   ├── capabilities/           # Tauri v2 permission declarations
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .claude/
│   ├── agents/                 # Project-scoped Claude agents
│   │   ├── frontend-react-engineer.md
│   │   └── rust-tauri-backend.md
│   └── agent-memory/           # Persistent agent memory (per agent)
├── documentation/              # Design docs, ADRs, research
├── index.html
├── vite.config.ts
└── package.json
```

## Architecture

The system has three layers:

1. **React + Vite frontend** — transcript view, speaker registry UI. Never communicates directly with the audio-server; all communication goes through Tauri commands and events.
2. **Rust backend (Tauri)** — owns mic capture (CPAL), VAD gating, HTTP client to audio-server, SQLite persistence (rusqlite), and a local REST API (axum) on `127.0.0.1:8765`.
3. **speech-swift audio-server** — external sidecar binary (macOS) or in-process plugin (iOS). Treated as a black-box inference service; the app never replicates its speaker identity data.

### Key Architectural Constraints

- **Frontend isolation**: The frontend never calls audio-server directly. The Tauri event API pushes live segment updates from Rust to the frontend.
- **Data ownership split**: speech-swift owns the speaker registry (JSON files). The app's SQLite DB owns sessions, transcript segments, and audio sample paths. `speaker_id` in SQLite is just a foreign reference to the speech-swift registry — never duplicated.
- **Audio in Rust, not WebView**: Mic capture lives in CPAL (Rust) to avoid WKWebView permission complexity and extra latency.
- **Local REST API is read-only**: `127.0.0.1:8765` — no auth, no writes. External scripts use this.

### SQLite Schema

```
sessions        — id, created_at, label, duration_ms, source (mic|file), file_path?
segments        — id, session_id, speaker_id, start_ms, end_ms, transcript_text
speaker_samples — id, speaker_id, session_id, start_ms, end_ms, audio_path
```

### audio-server Endpoints Used

| Endpoint | Purpose |
|---|---|
| `POST /registry/sessions` | Diarize + transcribe one audio chunk |
| `GET /registry/speakers` | List registry speakers |
| `PATCH /registry/speakers/:id` | Set display name |
| `POST /registry/speakers/merge` | Merge duplicate identities |
| `DELETE /registry/speakers/:id` | Remove speaker |

### File Upload: Large File Handling

Files ≤ ~100MB are posted whole. Files > ~100MB use a **sliding window** (10–15 min windows, ~2 min overlap); speaker IDs are stitched across windows via cosine similarity against the registry. For MVP, process files whole and add windowing only when users hit the limit.

### Mobile Path

Tauri v2 targets iOS without frontend or Rust changes. The only seam that changes is how the Rust layer reaches speech models:
- **macOS**: sidecar binary, HTTP on localhost
- **iOS**: native Tauri plugin (Swift, embedded `.xcframework`), CoreML/Neural Engine
- **Android**: significant work (ONNX/TFLite model porting required) — do not let this constrain MVP

## Tech Stack

| Layer | Technology |
|---|---|
| App shell | Tauri v2 |
| Frontend | React + Vite + TypeScript |
| UI components | Flowbite React + Tailwind CSS |
| Data fetching | TanStack Query |
| Routing | TanStack Router |
| Tables | TanStack Table |
| Long lists | TanStack Virtual |
| Audio capture | CPAL (Rust) |
| Persistence | rusqlite (SQLite) |
| Local API | axum (Rust) |
| ML inference | speech-swift audio-server (external) |

## Custom Agents

### Project-scoped (`/.claude/agents/`)

These agents are checked into the repo and have deep context about this codebase.

| Agent | When to use |
|---|---|
| `frontend-react-engineer` | Building, reviewing, or refactoring React components, TanStack Query hooks, routes, tables, or Flowbite UI |
| `rust-tauri-backend` | Rust backend work: CPAL audio, VAD, SQLite, axum API, HTTP client to audio-server, Tauri commands/events |

### User-scoped (`~/.claude/agents/`)

General-purpose agents available across all projects.

| Agent | When to use |
|---|---|
| `concept-researcher` | Exploring a nascent idea or approach before any design begins |
| `requirements-analyst` | Decomposing features into user stories and acceptance criteria |
| `system-architect` | Designing system architecture, writing ADRs, evaluating trade-offs |
| `ux-design-architect` | Designing UI screens and flows using Pencil.dev MCP with Flowbite React |
