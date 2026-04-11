---
name: Component Patterns
description: Established layout and component patterns for the Minutes app UI
type: project
---

## Sidebar Pattern

Width: 220px, always visible, `fill:"#FFFFFF"`, right border `#E5E7EB`.

Structure (vertical layout):
1. Logo row — 56px tall, horizontal, `padding:[0,16]`, mic icon + "Minutes" bold
2. 1px divider rectangle
3. Nav section — vertical, `padding:[8,8]`, `gap:2`
   - Each nav item: horizontal frame, 38px tall, `padding:[0,10]`, `gap:8`, `cornerRadius:6`
   - Active: `fill:"#EFF6FF"`, icon+text `#2563EB`, `fontWeight:"600"`
   - Inactive: `fill:"#00000000"`, icon+text `#6B7280`
4. Spacer (`fill_container`)
5. Bottom bar — 52px, top border `#E5E7EB`, settings gear icon only

Nav order: mic (Record) → users (Speakers) → layout-list (Sessions) → search (Search)

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
