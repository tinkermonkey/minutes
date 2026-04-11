---
name: rusqlite query_map lifetime pattern
description: query_map result must be collected before stmt goes out of scope; use a let binding before the closing brace
type: reference
---

When a function ends with a `stmt.query_map(...)?. collect::<Result<Vec<_>,_>>().map_err(Into::into)` chain, Rust infers that the `MappedRows` iterator — which borrows `stmt` — must be dropped before `stmt` is dropped. If the entire chain is an implicit tail expression, the temporary holding the iterator lives to the end of the block alongside `stmt`, causing E0597.

**Fix:** collect into a named binding before the block closes, then return it:

```rust
let rows = stmt.query_map([], |row| { ... })?
    .collect::<Result<Vec<_>, _>>()?;
Ok(rows)
```

Do NOT use `.map_err(Into::into)` at the end of a tail expression when `stmt` is in the same scope — break it into two statements.
