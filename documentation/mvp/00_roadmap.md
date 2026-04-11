# Minutes — MVP Roadmap

## Goal

A desktop app (macOS-first, iOS-compatible) that:
- Records conversations with live diarized transcription
- Identifies and names recurring speakers across sessions
- Persists all sessions and transcripts locally
- Supports semantic vector search across transcript history

---

## Tech Stack

### Existing (already in project)

| Layer | Technology |
|---|---|
| App shell | Tauri v2 |
| Frontend | React 19 + Vite 7 + TypeScript |
| Data fetching | TanStack Query v5 |
| Routing | TanStack Router v1 |
| Audio capture | CPAL (Rust) |
| Persistence | rusqlite + rusqlite_migration |
| Local REST API | axum 0.8 |
| HTTP client | reqwest 0.12 |
| Async runtime | tokio (full) |
| ML inference | speech-swift audio-server (external sidecar, macOS) |

### To install in Stage 0

| Layer | Technology |
|---|---|
| UI components | Flowbite React + Tailwind CSS |
| Tables | TanStack Table |
| Long lists | TanStack Virtual |

### New Rust crates for MVP

| Technology | Layer | Purpose | Rationale |
|---|---|---|---|
| `webrtc-vad` | Rust | Voice activity detection | Lightweight, WebRTC-based; determines utterance boundaries from the CPAL audio stream |
| `sqlite-vec` | Rust (SQLite ext) | Vector similarity search | Vector index lives in the same SQLite file — no separate service |
| `fastembed-rs` | Rust | Embedding generation | Bundles `all-MiniLM-L6-v2` (384-dim); fully offline, loaded once at startup |

---

## Stages

### Stage 0 — Foundation

**Outcome**: App boots with a working UI shell, persistent DB schema, and confirmed speech-swift connectivity.

- Install Flowbite React + Tailwind CSS, TanStack Table, TanStack Virtual
- Set up TanStack Router with stub routes: `/record`, `/speakers`, `/sessions`, `/search`
- Initialize SQLite on startup with rusqlite_migration: `sessions`, `segments`, `speaker_samples`, `speakers`, `segment_embeddings`
- Load sqlite-vec extension at DB init
- Configure Tauri capabilities: microphone, file dialog
- Call `GET /health` on speech-swift at launch; store result in app state (used by all subsequent stages to gate recording features)

---

### Stage 1 — Audio Pipeline + Persistence

**Outcome**: The full Rust pipeline from microphone to SQLite is operational; segments and embeddings accumulate correctly without any UI.

The pipeline runs inside a tokio task, started and stopped via Tauri commands.

- CPAL mic capture loop: 16kHz, mono, f32 samples streamed into a ring buffer
- `webrtc-vad` classifies frames as speech/silence; accumulates voiced frames into a chunk buffer
- Chunk flushed when silence is detected or 30s elapses; encoded as in-memory WAV bytes
- `POST /registry/sessions` to speech-swift; parse response: `{segments: [{speaker_id, speaker_label, start_ms, end_ms, transcript}]}`
- On response:
  - `INSERT INTO segments` for each segment
  - Save the WAV chunk to disk; `INSERT INTO speaker_samples` rows keyed to the chunk's speaker/time offsets (required for Stage 3 audio playback)
  - `UPSERT INTO speakers` for any new `speaker_id` values (local cache, not a rename)
  - Generate 384-dim embedding per segment via fastembed-rs; `INSERT INTO segment_embeddings` (vec0)
  - Emit `segment_added` Tauri event (payload: full `Segment` struct)
  - If a speaker has no `display_name`, emit `new_speaker` event
- Tauri commands: `start_session() -> SessionId`, `stop_session(session_id)`
  - `start_session` creates the `sessions` row and starts the pipeline task
  - `stop_session` stops the task and finalizes `duration_ms`

> **Note on embedding timing**: Embeddings are generated synchronously with segment insert in Stage 1. Any segments written before fastembed finishes loading (cold start, ~1–2s) are queued and embedded once the model is ready. No backfill step is needed.

---

### Stage 2 — Live Transcription UI

**Outcome**: A user can open the app, press record, and watch a live diarized transcript build in real time.

- `/record` route: record button (start/stop), live transcript panel, session status indicator
- Start button calls `start_session`; stop button calls `stop_session(session_id)`
- Record button disabled (with explanation) when speech-swift health check from Stage 0 failed
- Transcript panel subscribes to `segment_added` event; appends segment cards with speaker label chips and timestamps
- Auto-scrolls to newest segment; speaker label chips are visually distinct per speaker ID
- `new_speaker` event surfaces an inline prompt ("Unknown speaker detected — name them?") linking to `/speakers`

---

### Stage 3 — Speaker Registry

**Outcome**: Users can identify every speaker by name, merge mistaken duplicates, and listen to voice samples — completing the diarization workflow.

Two sync paths exist and serve distinct purposes:
- **Per-chunk upsert** (Stage 1): keeps `speakers` table current as new speakers are detected mid-session
- **Full registry refresh on startup**: `GET /registry/speakers` → UPSERT all; catches any changes made externally to speech-swift (e.g., renames from another tool)

Stage 3 builds the UI layer on top of this already-synced data:

- `/speakers` route:
  - List all cached speakers: name (or "Unknown"), first/last seen date, session count
  - **Name**: inline edit → `PATCH /registry/speakers/:id` + update `display_name` in local cache
  - **Merge**: select two speakers → `POST /registry/speakers/merge {src, dst}` → update `speaker_id` on affected `segments` rows in SQLite → remove merged speaker from cache
  - **Delete**: `DELETE /registry/speakers/:id` → remove from local `speakers` cache and disassociate `segments` rows
  - **Listen**: play audio clip from `speaker_samples.audio_path` for the selected speaker; enables voice-based identification

---

### Stage 4 — Session History

**Outcome**: Users can revisit any past session and re-read its transcript in full.

- `/sessions` route: paginated session list (TanStack Table), sortable by date and duration; shows participant speaker chips
- Session detail view: full read-only transcript sorted by `start_ms`; speaker name labels resolved from local `speakers` cache
- Date range filter on session list

---

### Stage 5 — Vector Search

**Outcome**: Users can ask a natural-language question and get relevant transcript excerpts back, filterable by speaker and time window.

The embedding infrastructure is already in place from Stage 1; this stage adds only the search command and UI.

- Tauri command: `search_segments(query, speaker_id?, start_date?, end_date?) -> Vec<SearchResult>`
  - Embed query string via fastembed-rs → 384-dim vector
  - ANN search via sqlite-vec on `segment_embeddings`
  - JOIN to `segments` and `sessions`; apply speaker and date filters in SQL
  - Return ranked results with segment text, speaker name, session label, and similarity score
- `/search` route: query input, speaker dropdown, date range pickers, result cards showing excerpt + session context

---

### Stage 6 — MVP Polish

**Outcome**: The app is stable, self-explanatory, and ready for daily use without developer involvement.

- Error boundaries and loading skeletons throughout all routes
- Actionable error state when speech-swift is unreachable: message explaining what speech-swift is, how to start it, and a retry button (builds on the Stage 0 health check infrastructure)
- Audio device selection: list CPAL input devices, persist preferred device, surface in a settings panel
- App icon, sensible default window dimensions, persist window position/size across restarts
- Window title reflects current recording state ("Recording…" vs. "Minutes")

---

## iOS Compatibility Note

The audio pipeline (CPAL + webrtc-vad), persistence layer (rusqlite + sqlite-vec), and embedding pipeline (fastembed-rs) are all iOS-compatible. The only seam that changes on iOS is how the Rust layer reaches speech models — via a native Tauri plugin (Swift, CoreML/Neural Engine) rather than the HTTP sidecar. No frontend or pipeline changes are required. This work is deferred to post-MVP.
