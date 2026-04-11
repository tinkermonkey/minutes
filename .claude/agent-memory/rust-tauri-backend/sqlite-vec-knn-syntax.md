---
name: sqlite-vec KNN query syntax
description: Correct SQL syntax for ANN/KNN queries against a vec0 virtual table in sqlite-vec 0.1.9
type: reference
---

sqlite-vec 0.1.9 requires a `MATCH` constraint on the vector column plus a bound on result count. Two valid forms:

**Form 1 — LIMIT (preferred for CTEs):**
```sql
SELECT segment_id, distance
FROM   segment_embeddings
WHERE  embedding MATCH ?1
ORDER  BY distance
LIMIT  ?2
```

**Form 2 — k = ? constraint:**
```sql
SELECT segment_id, distance
FROM   segment_embeddings
WHERE  embedding MATCH ?1 AND k = ?2
ORDER  BY distance
```

Both forms are confirmed in `sqlite-vec.c` (line 5547: "A LIMIT or 'k = ?' constraint is required on vec0 knn queries.").

**Query vector format:** same as insert — raw little-endian f32 bytes (`Vec<u8>` built by `f.to_le_bytes()`).

**CTE join pattern used in db/search.rs:**
```sql
WITH matches AS (
    SELECT segment_id, distance
    FROM   segment_embeddings
    WHERE  embedding MATCH ?1
    ORDER  BY distance
    LIMIT  ?2
)
SELECT ... FROM matches m JOIN segments sg ON sg.id = m.segment_id ...
```

The `ORDER BY distance` inside the CTE is required — sqlite-vec needs it to trigger the KNN code path. The outer `ORDER BY m.distance ASC` re-sorts after the post-CTE filter.

**Optional filter pattern (nullable params, single prepared statement):**
```sql
WHERE (?3 IS NULL OR sg.speaker_id  = ?3)
  AND (?4 IS NULL OR ss.created_at >= ?4)
  AND (?5 IS NULL OR ss.created_at <= ?5)
```
Pass `Option<i64>` directly as rusqlite params — rusqlite maps `None` to SQL NULL.
