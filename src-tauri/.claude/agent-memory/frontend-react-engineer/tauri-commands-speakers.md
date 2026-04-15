---
name: Tauri speaker commands
description: All Tauri invoke() command names related to speaker registry management and their input/output types
type: project
---

## Speaker registry commands

| Command | Input | Output |
|---|---|---|
| `get_speakers` | none | `Speaker[]` |
| `rename_speaker` | `{ speechSwiftId: number, name: string }` | `void` |
| `merge_speakers` | `{ srcId: number, dstId: number }` | `void` |
| `delete_speaker` | `{ speechSwiftId: number }` | `void` |
| `reset_speaker_registry` | none | `void` |
| `get_speaker_sample_path` | `{ speechSwiftId: number }` | `string | null` |

All of these are wrapped in hooks in `src/hooks/useSpeakers.ts`. Query key constant is `SPEAKERS_KEY = ['speakers']`.

The `reset_speaker_registry` command destroys all speaker identities and voice data in the speech-swift registry. All segments are left intact but marked unidentified. Added 2026-04-13.
