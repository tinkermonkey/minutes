---
name: axum 0.8 patterns
description: Non-obvious axum 0.8 API differences from 0.7 used in this project
type: reference
---

## Path parameter syntax

axum 0.8 uses `{param}` braces, NOT `:param` colon syntax (which was axum 0.7):

```rust
// Correct (0.8):
.route("/sessions/{id}/segments", get(handler))

// Wrong (0.7 style, panics at runtime with "Path segments must not start with `:`"):
.route("/sessions/:id/segments", get(handler))
```

## Integration test approach

Use `tower::ServiceExt::oneshot` with `axum::body::Body` — no need for `axum-test` crate. Add `tower = { version = "0.5", features = ["util"] }` as a dev-dependency (it is already a transitive dep of axum).

```rust
use tower::ServiceExt;
use axum::{body::Body, http::Request};

let resp = app
    .oneshot(Request::builder().uri("/sessions").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(resp.status(), StatusCode::OK);
```

## ApiState design

Each handler opens a short-lived `db::open_readonly` connection rather than sharing a single connection. This avoids contention with the write `Mutex<Connection>` in `AppState` and is safe under SQLite WAL mode (concurrent readers + single writer).
