---
name: Routing Structure
description: TanStack Router setup — route tree, file locations, and redirect behavior
type: project
---

## Router Setup

- Router created in `src/router.ts` using manual route tree (not file-based codegen).
- Root layout in `src/routes/__root.tsx` — exports `RootLayout`.
- Route components are named exports (not default exports) to match project conventions.

## Routes

| Path | Component | File |
|---|---|---|
| `/` | redirect to `/record` | `src/router.ts` (indexRoute) |
| `/record` | `RecordRoute` | `src/routes/record.tsx` |
| `/speakers` | `SpeakersRoute` | `src/routes/speakers.tsx` |
| `/sessions` | `SessionsRoute` | `src/routes/sessions.tsx` |
| `/sessions/$sessionId` | `SessionDetailRoute` | `src/routes/sessions.$sessionId.tsx` |
| `/search` | `SearchRoute` | `src/routes/search.tsx` |

## Active Link Detection

Uses `useRouterState().location.pathname` + string comparison (exact or prefix). TanStack Router's `Link` `activeProps` was not used — active styles applied manually via className to allow fine-grained Tailwind control.

**Why:** Manual routing tree chosen over file-based codegen to avoid adding `@tanstack/router-plugin` to the build and keep Stage 0 simple.

**How to apply:** Add new routes by: (1) creating `src/routes/<name>.tsx`, (2) creating a `createRoute` in `src/router.ts`, (3) adding to `rootRoute.addChildren([...])`.
