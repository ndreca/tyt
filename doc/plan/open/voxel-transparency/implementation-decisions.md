# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. Material fields

- `fill_color` and `fill_scalar` take a setter closure instead of a field
  accessor. The base color keeps the whole `TyLinSrgbaF64` and the emissive
  color takes its `color`, so the alpha drop is at the call site that wants
  it.
- `normal_reflectance` has no special case for an `ior` of `0`. The formula
  gives `1` there on its own. At the default `1.5` it gives `0.2 * 0.2`,
  which differs from the old `0.04` literal in the last bits. No golden
  moved.
- An alpha outside `0..1` errors under `baseColor`, like the color's other
  channels. `ior` has its own check, `is_refractive_index`, because its
  range is `0` or `1..`.
- The three new values join the deduplication key, so two materials that
  differ only in coverage, transmission, or index of refraction stay two
  entries.
