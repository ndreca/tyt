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

## S2. The ray walk

- `RenderRayWalk` carries the distance cap. `cast_ray` is its first hit.
  The S4 shadow walk ends at the light with no check in the caller.
- The one-item rule puts the walk in `render_ray_walk.rs`. `cast_ray.rs`
  keeps the function.
- The merge is lazy. Each placement advances only as far as the nearest hit
  another placement has found, so a placement behind a near hit never walks
  past it. A placement stopped at that limit keeps its DDA state and
  resumes on a later `next`.
- Materials compare by id. `from_vox_main` dedupes equal values into one
  id. Two equal materials retained by hand are two entries whose boundary
  is a hit.
- At a tie in distance the earlier placement yields first. `cast_ray` used
  to return the later one. No golden has a tie.
- The old `cast_ray` test of a ray from inside asserted a hit at the seam
  of a one-material bar. Under the surface rule that seam is internal, so
  the test now uses a two-material bar.
- `cells_scene` in `test_utilities` builds one object from `(cell, material
  index)` pairs under a list of transforms. Both the `cast_ray` and the walk
  tests use it.

## S3. The pixel walk

- `RenderPixel` is a struct with named fields in its own file. Its `Default`
  is a miss, so `RenderImage::new` fills with it.
- `shade_ray` in `render.rs` runs the walk for one ray and returns the light,
  the emission, and the transmittance in `f64`. `render` narrows the three
  per pixel. The compositing tests call `shade_ray` on a bar of materials
  under a white hemisphere light. There every shade is the sum of a diffuse
  color and the `0.04` reflectance, checkable by hand.
- `shade_hit` returns the coverage and the pass beside the color and the
  emission. The walk multiplies by the coverage. `material_pass` is a free
  function because the S4 shadow walk needs the pass without shading.
- The walk breaks at a throughput of exactly zero, which an opaque
  material's pass is. An opaque hit ends the walk after one hit as `cast_ray`
  did.
- `apply_bloom` has no miss case. Adding the halo to the light and scaling
  the transmittance by one minus the halo's peak gives a miss the same alpha
  and color the old case computed.
- `from_image` has no miss case either. A miss comes out with an alpha of
  zero, or under a background as the background times a transmittance of
  one.
- The output clamps the composited color per channel at one before the sRGB
  encode. A highlight on clear glass over white sums past one.
- `RenderOutput` guards the division by the alpha. The alpha is never below
  the light's peak. At zero the light is black and passes through as is.
- `bar`, the `render` test helper, dedupes equal materials into one id
  because the two-thick pane's voxels must share a material.

## S4. Shadows

- `shadow_factor` walks `RenderRayWalk` to the light and multiplies the
  pass of each hit, breaking at black as the pixel walk does. `cast_ray` has
  no caller left in `render.rs`.
- `hit_material` looks a hit's material up through its placement and
  object. `shade_hit` and the shadow walk both call it. `shade_hit` still
  fetches the placement and the object for the transform and the occlusion.
- `bilinear` is generic over the value, bound by `Default`, `Add`, and
  `Mul<f64>`, so the occlusion blends scalars and the shadow blends colors
  through one function. The blend folds from `Default` because palette's
  colors have no `Sum`.
- Under a light straight down, an axis-aligned pane shadows every corner
  sample of a floor voxel alike because the inset keeps each sample inside
  the voxel's footprint. The per-channel corner blend runs under the walled
  fixture's slanted light instead. The fixture takes the wall's material for
  that test.
