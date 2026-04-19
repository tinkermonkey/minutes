---
name: Component Patterns
description: Established layout and component patterns for the Minutes app UI
type: project
---

## Sidebar Pattern

Width: 220px, always visible, `fill:"#FFFFFF"`, right border `#E5E7EB`.

Structure (vertical layout, `padding:[0,8]`, `gap:2`):
1. Logo row — 52px tall, horizontal, `padding:[0,8]`, "Minutes" bold `fontSize:18`, bottom border `#F3F4F6`
2. Nav items — each horizontal frame, 36px tall, `padding:[0,12]`, `cornerRadius:8`, `fill_container` width
   - Active: `fill:"#EFF6FF"`, text `#1D4ED8`, `fontWeight:"600"`, `fontSize:13`
   - Inactive: no fill, text `#374151`, `fontSize:13`
3. Spacer (`fill_container`)
4. Settings row — 36px, top border `#F3F4F6`, text `#6B7280`

Nav order: Record → Speakers → Sessions → Search

## Segment Card (Transcript)

Horizontal frame, `width:"fill_container"`, `padding:12`, `gap:10`, `alignItems:"start"`, white bg, `cornerRadius:8`, border `#E5E7EB`.

Left column: vertical frame with speaker chip + timestamp below (gap:4, alignItems:"center")
- Speaker chip: horizontal frame, `height:22`, `padding:[0,8]`, colored bg+text, `cornerRadius:4`
- Timestamp: `fontSize:11`, `fill:"#9CA3AF"`

Right: transcript text with `textGrowth:"fixed-width"`, `width:"fill_container"`, `fontSize:13`, `lineHeight:1.5`

## Speaker Card (Registry)

Horizontal frame, full width, white bg, `cornerRadius:8`, border `#E5E7EB`.
- Left: 4px wide accent rectangle with speaker color, `cornerRadius:[8,0,0,8]`
- Body: vertical frame, `padding:[12,16]`, `gap:6`
  - Name row: horizontal, pencil edit icon, spacer, action buttons right-aligned
  - Meta row: muted text, `fontSize:12`, `fill:"#9CA3AF"`
  - Action row: headphone play btn (30×30) + Select btn + trash btn, right-aligned

Edit state: text input field with blue 2px border + confirm (green) / cancel (red) icon buttons.
Merge state: "Merge with Alice" button with git-merge icon, `fill:"#EFF6FF"`, `fill:"#2563EB"` text.

## TopBar Pattern (Persistent — RootLayout)

Height: 56px, `fill:"#FFFFFF"`, bottom border `#E5E7EB`, horizontal, `padding:[0,16]`, `gap:12`, `alignItems:"center"`.

Children left-to-right:
1. Brand text "Minutes" — `fontSize:16, fontWeight:"700", fill:"#111827"`
2. Vertical divider — `width:1, height:28, fill:"#E5E7EB"`
3. RecordButton — `cornerRadius:18, height:36, padding:[0,16]`
   - Idle: `fill:"#16A34A"`, text "Record" white
   - Recording: `fill:"#EF4444"`, red dot + text "Stop Recording" white
4. StatusBadge — `cornerRadius:6, height:28, padding:[0,10]`
   - Idle: `fill:"#F3F4F6"`, gray dot, text "Ready" `#6B7280`
   - Recording: `fill:"#FEE2E2"`, red dot, timer text `#991B1B`
5. MicPulse (recording only) — horizontal, gap:6, red dot `#EF4444` 10×10 + "Mic" text `#DC2626`
6. AudioMeter — horizontal, gap:4, `alignItems:"center"`
   - "mic" label `fontSize:11, fill:"#9CA3AF"`
   - 20 segments × `width:2, height:12, cornerRadius:1`: green (lit) / yellow (near peak) / gray (unlit)
   - VAD segment: `width:2, height:12, fill:"#1D4ED8"` (dim `#BFDBFE` when inactive)
7. AccumulatorBar — `width:"fill_container"`, horizontal, `gap:6`, `alignItems:"center"`
   - "Accumulator" label
   - Dual progress tracks (fast=blue `#60A5FA`, slow=purple `#A855F7`, bg `#F3F4F6`), each 5px tall, `cornerRadius:999`
   - Trigger labels column: "2s" / "10s" `fontSize:10, fill:"#9CA3AF"`
8. Language selector — `cornerRadius:6, height:30, fill:"#FFFFFF"`, border `#D1D5DB`

## SessionSpeakersSidebar Pattern (Record View — right panel)

Width: 240px, `fill:"#FFFFFF"`, left border `#E5E7EB`, vertical layout.

Structure:
1. Header — 48px, horizontal, `padding:[0,16]`, `gap:8`, bottom border `#F3F4F6`
   - "Session Speakers" `fontSize:13, fontWeight:"600", fill:"#1F2937"`
   - Spacer
   - Count badge: `cornerRadius:10, fill:"#F3F4F6"`, number `fontSize:11, fill:"#6B7280", fontWeight:"600"`
2. Body — `fill_container`, vertical, `gap:2`, `padding:[8,0]`
   - Section label wrap (horizontal `padding:[0,12]`): "RECOGNIZED" / "UNRECOGNIZED" `fontSize:10, fill:"#9CA3AF", fontWeight:"700"`
   - Speaker rows: horizontal, 36px, `padding:[0,12]`, `gap:8`, `cornerRadius:8`
     - Color dot: `ellipse width:10, height:10` with speaker color
     - Name column: vertical, `fill_container`, `gap:1`
       - Label: `fontSize:11, fill:"#374151", fontWeight:"600"`
       - Display name (recognized): `fontSize:11, fill:"#9CA3AF"`
     - Edit pencil icon button: 24×24 frame, `cornerRadius:4`, pencil lucide icon 12×12 `fill:"#9CA3AF"`
3. Footer — 36px, `padding:[0,16]`, top border `#F3F4F6`
   - "Names sync to Speakers registry" `fontSize:11, fill:"#9CA3AF"`

Speaker colors (index-based): 0=#2563EB (blue), 1=#7C3AED (purple), 2=#0891B2 (cyan), 3=#059669 (green), 4=#D97706 (amber), 5=#DC2626 (red)

## PipelineEventLog Pattern

Inline card, `fill:"#FFFFFF"`, `cornerRadius:8`, border `#E5E7EB`, vertical layout, `padding:[8,12]`, `gap:2`.

Section header above card: "PIPELINE" label `fontSize:10, fill:"#6B7280", fontWeight:"600"`.

Header row: horizontal, 24px, `gap:12`, `padding:[0,0,4,0]`, bottom border `#F3F4F6`
- Spacer 20×1, then 7 column headers: Time(80px), Position(100px), Len(40px), Response(70px), Speed(50px), Best score(60px) — all `fontSize:10, fill:"#6B7280", fontWeight:"500", textGrowth:"fixed-width"`

Data rows: horizontal, 24px, `gap:12`, `padding:[0,4]`, `cornerRadius:4`
- Fast row: `fill:"#EFF6FF"`, F badge `fill:"#DBEAFE"` text `#1D4ED8`
- Slow row: no fill, S badge `fill:"#F3E8FF"` text `#7C3AED`
- Badge: 20×20 circle, centered letter `fontSize:10, fontWeight:"700"`
- Data cells: `fontSize:11`, fixed widths matching header columns, `textGrowth:"fixed-width"`
  - Time/Len/Speed: `fill:"#6B7280"` — Position/Response/BestScore: `fill:"#374151"`

## SegmentCard Pattern (Transcript)

Horizontal frame, `fill:"#FFFFFF"`, `cornerRadius:8`, border `#E5E7EB`, `padding:12`, `gap:12`, `alignItems:"start"`.

Children left-to-right:
1. Chip column — vertical, `gap:2`, `alignItems:"start"`
   - Speaker chip: horizontal, `height:22`, `padding:[0,8]`, `cornerRadius:4`, colored bg+text
     - Confirmed: speaker color bg/text (e.g. `fill:"#DBEAFE"`, text `fill:"#1D4ED8"`)
     - Pending: `fill:"#F3F4F6"`, text "Identifying..." `fill:"#6B7280", fontStyle:"italic"`
   - Debug badge (debug mode only): `fontSize:11, fill:"#9CA3AF"` showing "(f) 2.1s"
2. Timestamp — `fontSize:11, fill:"#9CA3AF"`, `width:40, textGrowth:"fixed-width"`, monospace feel
3. Transcript text — `fontSize:13, fill:"#1F2937"`, `width:"fill_container", textGrowth:"fixed-width"`

Transcript section header: horizontal, 20px, `padding:[0,4]`, `gap:8`, `alignItems:"center"`
- "TRANSCRIPT" label + spacer + debug toggle (checkbox 14×14 `cornerRadius:3` + "Debug" label)

Newest segment appears at TOP (TranscriptPanel reverses and virtualizes).

## Pen File Layout Quirk — Absolute Positioning for Body Columns

When `fdX0o` (Record View) body columns need horizontal layout, the `layout:"horizontal"` property
does not persist correctly on intermediate wrapper frames in this file. The workaround is to place
the three column frames (Sidebar, MainContent, SessionSpeakersSidebar) as direct children of `fdX0o`
with `layoutPosition:"absolute"` and explicit x/y/width/height values:
- Sidebar: x:0, y:56, w:220, h:744
- MainContent: x:220, y:56, w:820, h:744
- SessionSpeakersSidebar: x:1040, y:56, w:240, h:744

## Speakers Management Page Patterns

### Speaker Table Row (Dense List, 48px)
Horizontal frame, `width:"fill_container"`, `height:48`, `padding:[0,24]`, `gap:0`, `alignItems:"center"`.
Column widths: Speaker name col 260px (dot + name inside), Sessions 90px, Last Seen 120px, Transcript fill_container.
- Recognized row: white bg, bottom border `#F3F4F6`
- Unrecognized row: `fill:"#FFFBEB"`, left+bottom border `#FDE68A`, left thickness 3px
- Selected row: `fill:"#EFF6FF"`, border `#BFDBFE`, left 3px blue; name `fill:"#1D4ED8"`, checkmark icon `circle-check` replacing color dot

### Unrecognized Section Header
`fill:"#FFFBEB"`, `height:36`, left amber border 3px, warning icon + "Unrecognized (N)" label in `#92400E`, helper text right-aligned.

### Speaker Color Dots (Table)
`ellipse`, `width:10, height:10` with speaker palette color. Gray `#D1D5DB` for unrecognized.

### Similar Speakers Right Panel (320px)
Absolutely positioned at `x:960, y:0, width:320, height:744` inside body frame.
- Panel header 56px: large avatar circle (32px) + name/subtitle col + close X button
- Section header 40px: "Most Similar Speakers" label + spacer + result count
- Similarity score badges: green `#DCFCE7/#15803D` ≥85%, amber `#FEF9C3/#A16207` 60-84%, red `#FEE2E2/#DC2626` <60%
- Each card: score badge + name + recognized/unrecognized badge + folder icon + session count + audio player row + merge/delete action row
- Audio player: `height:32`, play circle `22×22` blue + waveform bar fill_container `height:3` gray + duration text
- Merge btn: blue filled, git-merge icon + "Merge into [Name]", `height:28`
- Delete btn: white with red outline, `height:28`
- Footer: "View all sessions" link (blue) + "Delete [Name]" full-width red outline btn

### Merge Confirmation Panel State
Same right panel but section header turns amber `#FEF9C3`. Three zones:
1. Merge description card `#F9FAFB`: bold "Merge B → A" + muted explanation text
2. Action card `#EFF6FF` (blue tint): "Confirm Merge" blue + "Cancel" gray stacked
3. Warning card `#FFFBEB`: "⚠ This cannot be undone" in amber

### Speaker Detail Panel (440px slide-in)
Replaces/overlays right portion of main content at x:620, width:440.
- Header 100px: large avatar 52px circle + editable name 18px bold + edit pencil badge
- Stats row 44px: session count chip, first/last seen dates in muted text
- Sections reuse table row pattern: section label row `#F9FAFB` 32px + data rows 48-60px
- Recent Sessions: session name + duration + segment count per row
- Recent Transcript: timestamp chip left + transcript text (wrapping) per entry
- Voice Sample: "VOICE SAMPLE" label row + "▶ Play sample · 0:08" link row
- Action buttons row 60px: Rename (blue filled) + Find Similar & Merge (gray outline) + Delete (red outline)

## Critical Pencil.dev Rendering Rule — Copy-Then-Update Pattern

**Rule**: Only nodes that exist at copy-time render in screenshots. Nodes added via Insert (I) or Replace (R) after a Copy (C) do NOT appear in screenshots, even though they exist in the data.

**Why**: The Pencil screenshot renderer caches the node tree at creation time. Fresh Insert/Replace operations after a Copy are invisible in the screenshot tool.

**How to apply**:
1. Always Copy an existing working frame as the starting point for new frames
2. Use Update (U) operations only on the copied nodes' existing children
3. Use `enabled:false` to hide children you don't need
4. Remap existing text/icon/shape children via U() to new content
5. Never use I() or R() to add net-new visible content to a copied frame's subtree

**Workaround for truly new content**: Copy another node as a sibling inside the body frame, position it absolutely, then update its children in-place.

## Freshly-Created Frame Rendering Bug

**Rule**: Frames created from scratch via I(document, ...) at canvas x>0 positions may fail to render in screenshots entirely (show blank), even with correct content.

**Why**: Unknown Pencil renderer bug — appears position-dependent or related to file state.

**How to apply**: Always use C() to copy an existing working frame rather than creating new frames from scratch. Even for completely different content, start with a copy and update in-place.

## Search Result Card

Vertical frame, `padding:14`, `gap:8`, white, `cornerRadius:8`, border `#E5E7EB`.
- Top row: speaker badge (left) + spacer + relevance bar (80px wide, `#F3F4F6` bg, green/amber fill) + % label
- Relevance color: ≥70% green (`#16A34A`), <70% amber (`#CA8A04`)
- Body: transcript excerpt, `textGrowth:"fixed-width"`, `fontSize:13`
- Footer: calendar icon + "Apr X, 2026 · H:MM PM →" in muted text

## Table Pattern (Sessions)

Full-width table inside a white card with border `#E5E7EB`, `cornerRadius:8`.
- Header row: 40px, `fill:"#F9FAFB"`, bottom border `#E5E7EB`
  - Column headers: `fontSize:12, fontWeight:"600", fill:"#374151"`
  - Sort icon: arrow-down (active/blue `#2563EB`), chevrons-up-down (inactive/gray)
- Data rows: 48px, alternating `#FFFFFF` / `#F9FAFB`, divider `#F3F4F6`
- Column widths: Date 180px, Duration 120px, Participants fill, Source 100px
- Pagination bar: 52px, top border, "Showing X–Y of Z" left, Prev (disabled gray) + Next right

## Page Header Pattern

Horizontal frame, `height:64`, `padding:[0,24]`, `alignItems:"center"`, white, bottom border `#E5E7EB`.
- Title `fontSize:20, fontWeight:"700", fill:"#111827"` on left
- Controls/filters on right via spacer
