---
name: Design System & Color Palette
description: Speaker color system, surface/border tokens, and sidebar styling for the Minutes app
type: project
---

## Speaker Color System

Applied consistently across all screens wherever speaker identity is shown (segment chips, table participant badges, search result badges, card accent borders).

| Speaker     | Chip bg   | Chip text | Accent border | Usage                    |
|-------------|-----------|-----------|---------------|--------------------------|
| SPEAKER_00  | #DBEAFE   | #1D4ED8   | #2563EB       | "Alice" — blue           |
| SPEAKER_01  | #DCFCE7   | #15803D   | #16A34A       | "Bob" — green            |
| SPEAKER_02  | #F3E8FF   | #7E22CE   | #7C3AED       | "Carol" — purple         |
| SPEAKER_03  | #FEF9C3   | #A16207   | #D97706       | "Dave" / Unknown — amber |

## Surface & Border Palette

- Page background: `#F9FAFB` (gray-50)
- Card / panel surface: `#FFFFFF`
- Card border: `#E5E7EB` (gray-200)
- Table row divider: `#F3F4F6` (gray-100)
- Alternating table row: `#F9FAFB`
- Sidebar border (right): `#E5E7EB`
- Header border (bottom): `#E5E7EB`

## Typography

- Page title / h2: `fontSize:20, fontWeight:"700", fill:"#111827"`
- Section label / column header: `fontSize:12, fontWeight:"600", fill:"#374151"`
- Body text: `fontSize:13, fill:"#111827", lineHeight:1.5`
- Meta / muted: `fontSize:12, fill:"#9CA3AF"`
- Badge text: `fontSize:11, fontWeight:"600"`

## Action Colors

- Primary button (Record, Search): `fill:"#2563EB"` (blue-600)
- Record/active button: `fill:"#16A34A"` (green-600)
- Destructive (trash): bg `#FEF2F2`, icon `#DC2626`, border `#FECACA`
- Warning banner: bg `#FFFBEB`, border `#FDE68A`, text `#92400E`, icon `#D97706`

## Status Badge (Ready)

- bg: `#F3F4F6`, dot: `#9CA3AF`, text: `#6B7280`
- Dot size: 7×7 ellipse
