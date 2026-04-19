---
name: Pen File Locations
description: Where Pencil.dev design artifacts live for the Minutes app
type: reference
---

## Main Design File

`/Users/austinsand/workspace/minutes/documentation/designs/minutes.pen`

Contains two screen frames side by side:
- Left (x:0, y:0): Record View — Idle + Live Transcript (node: fdX0o) — 1280×800px
- Right (x:1340, y:0): Speakers View — Recording Active (node: Ul45e) — 1280×800px

### Record View (fdX0o) structure
- TopBar (g2c6v): horizontal, height 56, fill white, bottom border gray-200
  - Brand text "Minutes" | divider | RecordButton (green idle / red recording) | StatusBadge | micPulse (recording only) | AudioMeter (mic segments) | AccumulatorBar (fill_container, dual bars) | langSelect
- Three columns via absolutely-positioned children of fdX0o at y:56, height:744:
  - Sidebar (3SUUZ): x:0, w:220 — logoRow (52px) + nav items + spacer + settings
  - MainContent (8SwL4): x:220, w:820 — AudioLevelGraph + PipelineSection + TranscriptSection
  - SessionSpeakersSidebar (hcKFC): x:1040, w:240 — header + speaker rows + footer

### Speakers View (Ul45e) structure
- Same TopBar pattern (X1h8a) — red Stop Recording button, 00:42 timer, recording active state
- Body (1lsMM): Sidebar (8yXBE, w:220) + SpeakersContent (y8WXW)
- Annotation notes at x:1340 with layoutPosition:absolute

### Speakers Management Frames (added Apr 2026)
Four frames below the existing two, at y:960:
- Frame 1 (x:0, y:960, node:z2C6I) — Default State: full speaker list, recognized + unrecognized sections, toolbar with filter tabs
- Frame 2 (x:1340, y:960, node:nMbn1) — Speaker Selected: Alice row highlighted blue, right 320px similar speakers panel with 3 match cards + footer
- Frame 3 (x:2680, y:960, node:wA4sF) — Speaker Detail: list narrowed to 620px, 440px detail panel slides in from right with avatar, sessions, transcripts, voice sample, action buttons
- Frame 4 (x:4020, y:960, node:epV9m) — Merge Confirmation: right panel shows amber confirm header, merge description, blue confirm/cancel area, amber warning note

## Navigation Redesign File

`/Users/austinsand/workspace/minutes/documentation/designs/navigation-redesign.pen`

Separate file for navigation explorations.
