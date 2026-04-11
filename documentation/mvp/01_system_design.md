# Minutes — System Design

## System Layers

```
┌─────────────────────────────────────────────────────────────────┐
│  React Frontend (Tauri WebView)                                 │
│                                                                 │
│  TanStack Router  ·  TanStack Query  ·  Flowbite React          │
│                                                                 │
│  /record        /speakers       /sessions        /search        │
│  LiveTranscript  SpeakerList    SessionList      SearchView     │
│                                                                 │
│  No audio code. No direct DB access. No speech-swift calls.     │
└──────────────────────────┬──────────────────────────────────────┘
                           │  Tauri commands + events
                           │  (IPC over secure bridge)
┌──────────────────────────▼──────────────────────────────────────┐
│  Rust Backend (Tauri)                                           │
│                                                                 │
│  ┌─────────────────┐  ┌──────────────┐  ┌────────────────────┐ │
│  │  Audio Pipeline │  │  DB Layer    │  │  Local REST API    │ │
│  │                 │  │              │  │  axum 0.8          │ │
│  │  CPAL capture   │  │  rusqlite    │  │  127.0.0.1:8765    │ │
│  │  webrtc-vad     │  │  sqlite-vec  │  │  (read-only)       │ │
│  │  WAV chunker    │  │  fastembed   │  │                    │ │
│  └────────┬────────┘  └──────┬───────┘  └────────────────────┘ │
│           │                  │                                  │
│  ┌────────▼──────────────────▼──────────────────────────────┐   │
│  │  speech-swift HTTP Client (reqwest)                      │   │
│  │  POST /registry/sessions  ·  GET/PATCH/POST/DELETE /...  │   │
│  └────────────────────────────────┬─────────────────────────┘   │
└───────────────────────────────────┼─────────────────────────────┘
                                    │  HTTP (localhost)
┌───────────────────────────────────▼─────────────────────────────┐
│  speech-swift audio-server (external sidecar)                   │
│                                                                 │
│  Diarization · ASR · Speaker Registry · VAD (internal)          │
│  Port: 8080 (configurable)                                      │
│                                                                 │
│  Source of truth: speaker identity + embeddings                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities

### React Frontend

| Component | Responsibility |
|---|---|
| `LiveTranscriptView` | Subscribes to `segment_added` Tauri event; renders growing transcript with speaker chips; auto-scrolls |
| `SpeakerListView` | Displays cached speakers; triggers rename/merge via Tauri commands |
| `SessionListView` | Paginates `sessions` table via TanStack Query + Tauri command; links to transcript replay |
| `SearchView` | Accepts query + optional filters; calls `search_segments` Tauri command; renders scored results |
| TanStack Query hooks | All data fetching — wraps Tauri `invoke` calls; handles loading/error states |
| TanStack Router | Route management — no auth, no guards |

### Rust Backend

| Module | Responsibility |
|---|---|
| `audio::capture` | CPAL input stream; samples at 16kHz mono f32 |
| `audio::vad` | `webrtc-vad` wrapper; classifies frames as speech/silence; accumulates voiced chunks |
| `audio::chunker` | Flushes complete chunks (silence boundary or 30s max) as WAV bytes |
| `client::speech_swift` | reqwest HTTP client; `POST /registry/sessions`, speaker CRUD |
| `db::migrations` | rusqlite_migration; applies schema on startup |
| `db::segments` | Insert/query segments; trigger embedding on insert |
| `db::speakers` | Upsert speaker cache; detect newly-identified speakers |
| `db::search` | sqlite-vec ANN query; JOIN with segments + sessions for filtered results |
| `embed` | fastembed-rs wrapper; lazy-loads `all-MiniLM-L6-v2`; generates 384-dim embeddings |
| `api` | axum router; read-only REST endpoints at `127.0.0.1:8765` |
| `commands` | Tauri command handlers (`start_session`, `stop_session`, `start_recording`, etc.) |
| `events` | Tauri event emitters (`segment_added`, `new_speaker`, `speech_swift_unreachable`) |

### speech-swift audio-server

External binary (sidecar on macOS). The app treats it as a black box:
- Accepts WAV audio
- Returns diarized + transcribed segments with speaker IDs
- Maintains its own speaker registry (JSON files, cosine similarity matching)
- The app never reads or writes speech-swift's internal registry files directly

---

## Data Flows

### Live Transcription

```
Microphone
    │
    ▼ (16kHz f32 samples)
CPAL capture loop
    │
    ▼
webrtc-vad frame classifier
    │ voiced frames buffered
    ▼ (silence detected OR 30s elapsed)
WAV encoder (in-memory bytes)
    │
    ▼ POST /registry/sessions
speech-swift
    │
    ▼ { segments: [{speaker_id, speaker_label, start_ms, end_ms, transcript}] }
Rust handler
    ├─→ INSERT INTO segments
    ├─→ save WAV chunk to disk → INSERT INTO speaker_samples
    ├─→ UPSERT INTO speakers  (cache sync)
    ├─→ fastembed-rs → INSERT INTO segment_embeddings (vec0)
    ├─→ emit segment_added event
    └─→ if speaker has no display_name → emit new_speaker event
              │
              ▼ (Tauri IPC)
React LiveTranscriptView
    └─→ append segment card, scroll
```

### Speaker Registry Sync

```
App startup                     After each session chunk
    │                                    │
    ▼                                    ▼
GET /registry/speakers          UPSERT speakers from segment response
    │
    ▼
UPSERT all into local speakers table
    │
    ▼
speakers with no display_name → emit new_speaker event
    │
    ▼
React: surface "Name this speaker" prompt
```

### Vector Search

```
User types query + optional filters
    │
    ▼ invoke search_segments(query, speaker_id?, start?, end?)
Rust: fastembed-rs embeds query → [f32; 384]
    │
    ▼ sqlite-vec ANN: SELECT segment_id, distance FROM segment_embeddings
      WHERE embedding MATCH ? ORDER BY distance LIMIT 50
    │
    ▼ JOIN segments, sessions WHERE speaker_id=? AND created_at BETWEEN ? AND ?
    │
    ▼ Vec<SearchResult> { segment, session_label, speaker_name, score }
    │
    ▼ (Tauri IPC)
React SearchView: render ranked result cards with session + speaker context
```

### File Ingestion (Post-MVP)

For files ≤ ~100MB: POST entire WAV to `/registry/sessions` as a single request.

For files > ~100MB: sliding window (10–15 min windows, ~2 min overlap); stitch speaker IDs across windows via cosine similarity against the registry. **Deferred to post-MVP.**

---

## SQLite Schema

```sql
-- Core session record
CREATE TABLE sessions (
    id          INTEGER PRIMARY KEY,
    created_at  INTEGER NOT NULL,      -- unix ms
    label       TEXT,
    duration_ms INTEGER,
    source      TEXT NOT NULL          -- 'mic' | 'file'
);

-- Diarized transcript segments (from speech-swift)
CREATE TABLE segments (
    id              INTEGER PRIMARY KEY,
    session_id      INTEGER NOT NULL REFERENCES sessions(id),
    speaker_id      INTEGER NOT NULL,  -- foreign ref to speech-swift registry
    start_ms        INTEGER NOT NULL,
    end_ms          INTEGER NOT NULL,
    transcript_text TEXT NOT NULL
);

-- Raw audio slices per speaker (for playback)
CREATE TABLE speaker_samples (
    id          INTEGER PRIMARY KEY,
    speaker_id  INTEGER NOT NULL,
    session_id  INTEGER NOT NULL REFERENCES sessions(id),
    start_ms    INTEGER NOT NULL,
    end_ms      INTEGER NOT NULL,
    audio_path  TEXT NOT NULL          -- path to WAV file on disk
);

-- Local cache of speech-swift speaker registry
CREATE TABLE speakers (
    id              INTEGER PRIMARY KEY,
    speech_swift_id INTEGER NOT NULL UNIQUE,
    display_name    TEXT,              -- null until user names them
    notes           TEXT,
    first_seen_at   INTEGER NOT NULL,  -- unix ms
    last_seen_at    INTEGER NOT NULL
);

-- Vector index for semantic search (sqlite-vec vec0 virtual table)
CREATE VIRTUAL TABLE segment_embeddings USING vec0(
    segment_id INTEGER PRIMARY KEY,
    embedding  FLOAT[384]
);
```

---

## speech-swift Endpoints Used

| Endpoint | Method | Purpose |
|---|---|---|
| `/health` | GET | Startup health check |
| `/registry/sessions` | POST | Diarize + transcribe one audio chunk; returns segments |
| `/registry/speakers` | GET | Full speaker list (for cache sync) |
| `/registry/speakers/:id` | PATCH | Set display name / notes |
| `/registry/speakers/merge` | POST | Merge two speaker identities (`{src, dst}`) |
| `/registry/speakers/:id` | DELETE | Remove a speaker |

speech-swift runs on `localhost:8080` by default. Port is configurable.

---

## Tauri Command + Event Surface

### Commands (frontend → Rust)

| Command | Signature | Description |
|---|---|---|
| `start_session` | `() -> SessionId` | Creates sessions row, starts recording pipeline |
| `stop_session` | `(session_id) -> ()` | Stops pipeline, finalizes duration |
| `get_sessions` | `(page, limit) -> Vec<Session>` | Paginated session list |
| `get_segments` | `(session_id) -> Vec<Segment>` | All segments for a session |
| `get_speakers` | `() -> Vec<Speaker>` | Local speaker cache |
| `rename_speaker` | `(speech_swift_id, name) -> ()` | PATCH + update cache |
| `merge_speakers` | `(src_id, dst_id) -> ()` | POST merge + re-point segments + update cache |
| `delete_speaker` | `(speech_swift_id) -> ()` | DELETE + disassociate segments + update cache |
| `search_segments` | `(query, filters) -> Vec<SearchResult>` | Vector search |
| `get_audio_devices` | `() -> Vec<AudioDevice>` | CPAL input device list |
| `set_audio_device` | `(device_id) -> ()` | Persist preferred device |

### Events (Rust → frontend)

| Event | Payload | Description |
|---|---|---|
| `segment_added` | `Segment` | New transcript segment ready |
| `new_speaker` | `Speaker` | Unnamed speaker detected |
| `speech_swift_unreachable` | `()` | Health check failed |

---

## Local REST API (axum, port 8765)

Read-only. No auth. For external tooling (scripts, integrations).

| Endpoint | Method | Returns |
|---|---|---|
| `/sessions` | GET | Session list |
| `/sessions/:id/segments` | GET | Segments for session |
| `/speakers` | GET | Speaker list |
| `/search` | GET | `?q=&speaker_id=&start=&end=` → search results |

---

## Mobile Seam (iOS)

The only component that changes for iOS is how the Rust layer reaches speech models:

| Platform | Audio capture | VAD | speech-swift interface |
|---|---|---|---|
| macOS | CPAL | webrtc-vad (Rust) | HTTP sidecar (localhost:8080) |
| iOS | CPAL | webrtc-vad (Rust) | Native Tauri plugin (Swift, CoreML/Neural Engine) |

All other layers — frontend, DB, embeddings, axum API — are identical. Tauri v2 compiles to iOS without frontend changes. The sidecar binary is replaced by an `.xcframework` embedded in the app bundle.
