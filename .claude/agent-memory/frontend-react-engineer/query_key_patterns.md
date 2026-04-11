---
name: Query Key Patterns
description: TanStack Query key conventions established in the codebase
type: project
---

## Established Query Keys

| Key | Purpose |
|---|---|
| `['speech_swift_status']` | Speech-swift reachability boolean. staleTime: Infinity — only updated via Tauri event invalidation or setQueryData. |

## Convention

Query keys are plain arrays of string literals for now (no factory functions yet). As more queries are added, extract to a `queryKeys.ts` constants file when there are 3+ keys sharing a prefix.

**Why:** Inline keys are fine for Stage 0 but will need centralizing once sessions, speakers, and search queries are added.

**How to apply:** When adding new queries, use `['resource', ...params]` shape. Create `src/lib/queryKeys.ts` when the number of keys grows beyond a few.
