# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. Integer traversal

- `RenderRayWalk::new` takes one `RenderGridRay` per placement, in
  placement order. Primary rays come from `RenderGridViewRays` and shadow
  rays from fixed points on the face. `from_ray` quantizes a world ray for
  callers that hold one, such as `cast_ray`.
- The walk mirrors each axis the ray moves down, so it only steps up. A ray
  from outside starts one cell short of the face it enters through, which
  makes the entry an ordinary first step.
- Entry products and end cells use `i128`, so the walk is total over any
  `RenderGridRay`, whose fields are public. `GRID_RANGE_BITS` keeps the
  same values within `i64` for the GPU tiers.
- The error terms also give a hit's place across its face: `ONE * d` plus
  or minus a term before the step. `along` divides that once in `f64`.
  `point` rounds it, and a per-pixel shadow ray starts there.
- A point or spot light's ray comes from `toward`: the offset shifted right
  to 16 bits, ending where it reaches the light's coordinate on its major
  axis. A shift needs only a leading-bit count on the GPU, where a 64-bit
  division would not fit WGSL. The end replaces the `f64` distance cap, so
  `RenderRayWalk` lost `max_distance` and `cast_ray` filters by distance.
- A shadow ray's origin enters the other placements' grids through world
  space in `f64`. Only the hit's own placement starts from the exact fixed
  point. Exact origins across placements would need an integer map per
  placement pair, and no scene needs that yet.
- `ShadowTarget` stays in world space, and each sample quantizes the light
  into every placement's grid. That gives the same rays as a per-render
  table, which a GPU tier would upload instead.
- The largest corner component of a view's direction lands under 2^16 by
  `1 + (width + height) >> 12`, the most the steps' rounding can add. No
  pixel's direction exceeds 2^16.
- Halves round away from zero, so mirrored inputs quantize mirrored.
- A point out of range errors as `GridRange` where it is quantized, so
  `shade_ray`, `shade_hit`, and `shadow_factor` return `Result`.
- The goldens regenerated. 15 of 28 moved, each on at most 10 pixels and
  by at most 1/255 per channel:
  1. glass per-pixel, per-face, and per-corner
  2. l-shape per-pixel, per-face, and per-corner
  3. room, all four
  4. spot-room, all four
  5. two-placements per-corner
