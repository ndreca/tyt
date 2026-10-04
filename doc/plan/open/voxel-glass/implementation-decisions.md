# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. Transparent output

- The transparent branch solves in `f32` after the sRGB transfer. The color
  needs no clamp because every channel over white lies between the
  transparency and one. palette's `u8` conversion saturates rounding noise.
- The plan said colorless transmittance moves by a level or two. A colorless
  transmittance keeps today's alpha, but its color moves toward the exact
  composite by up to 19 levels in a halo's core. The README now states the
  measured result. The test checks the alpha instead of the move.
- Against exact composites of the `glow` golden, the halos' mean error over
  white falls from 2.7 levels to 0.1. Over black it falls from 28.7 to 26.3.
- Only the `glass` and `glow` goldens moved, all four variants of each. The
  others hold byte for byte.

## S2. Exits

- An exit's face is the face of the cell it leaves, so its normal points
  along the ray. `shade_hit` turns the normal back. The face's plane and
  `face_origin` need no change. A shadow ray from the face starts inside the
  material by the boundary rule.
- The glass side of an exit through a grid's low face would be a span on
  layer -1, which `SurfaceSpan`'s `u32` layer cannot hold. voxsurface gained
  `inner_corner_occlusion`, which reads the span's own layer. Both it and
  `corner_occlusion` call a crate-internal `layer_occlusion`.
- `PlacementWalk` holds the scene because the walk needs opacity to skip an
  exit into an opaque cell. `is_opaque` now owns the test for the
  walk and `RenderGrid`. `RenderScene` has no `Debug` because `IdList` has
  none, so `PlacementWalk` prints a few fields by hand.
- `leaves` tells a step past the grid's edge from a step past the ray's end.
  A shadow ray that ends at a light inside a grid meets no exit there.
- An opaque material exits too because the rule reads only what the ray
  leaves for. The pixel and shadow walks stop at a zero throughput, so no
  renderer shades one.
- The entry after an exit waits in `queued` and comes out on the next call
  whatever the limit. The merge holds it as a peeked hit until it is the
  nearest.
- A transmissive emitter emits at both walls because an exit shades as an
  entry does.
- `bar_scene`, `matte`, and `glass` moved from the render tests to
  `test_utilities`. The walk tests share them.
- Only the four `glass` goldens moved. The others hold byte for byte.

## S3. Fresnel by angle

- voxrender's `material_pass` takes the cosine and restates voxcore's
  formula with `F` in place of `F0`. A test holds `material_pass` to
  voxcore's at normal incidence. voxcore's `pass` is unchanged because the
  mesher reads it.
- `transmitted_reflectance` owns `F`. The pass and the hemisphere term both
  read it.
- Both walks take the cosine in world space against the shading normal. A
  shadow ray takes a directional light's direction, or the direction from
  the ray's world origin to a point or spot light.
- The hemisphere term splits `F0` between the shares: the transmitted share
  reflects `F` along the mirror direction, and the rest reflects `F0` along
  the normal. With no transmitted share the term matches today's to the bit.
- `is_opaque` keeps reading normal incidence because a pass that is zero
  there is zero at every angle a ray can cross a face at.
- In the review's hero view the key light grazes the red pillar's side at a
  cosine of 0.024. That wall passes 0.27 of the red where it passed 0.92.
  The pillar's shadow on the floor behind it darkens. The shadow's per-corner
  staircase now shows through the pillar.
- Only the four `glass` goldens moved, by up to 21 levels. The others hold
  byte for byte.

## S4. Double-sided glass

- The `glass` material writes a literal `true` because one blended material
  serves every transparent swatch. The `body` material keeps glTF's
  back-face culling.
- Both copies of the profile were edited by hand because no test ties the
  reference's listing to the embedded map.
- Meshing `glass.voxj` under the profile writes `doubleSided` on the `glass`
  material alone.
