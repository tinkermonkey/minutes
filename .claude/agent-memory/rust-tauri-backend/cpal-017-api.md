---
name: cpal 0.17 API changes
description: SampleRate is a u32 type alias, not a tuple struct; with_sample_rate returns Option; device.name() is deprecated
type: reference
---

In cpal 0.17, `SampleRate` is `pub type SampleRate = u32` — a plain type alias, not a newtype wrapper.

Consequences:
- `cpal::SampleRate(16_000)` does not compile — use the literal `16_000u32` directly.
- `config.sample_rate` is a `u32`, not `.0`-accessible.
- `c.min_sample_rate()` and `c.max_sample_rate()` return `u32`.
- `SupportedStreamConfigRange::with_sample_rate()` panics on out-of-range; use `try_with_sample_rate()` which returns `Option<SupportedStreamConfig>`.
- `DeviceTrait::name()` is deprecated in 0.17.3 — use `description()` or `id()` for stable identifiers (we keep `name()` for now with a `#[allow(deprecated)]` note; tolerate the compiler warning).
