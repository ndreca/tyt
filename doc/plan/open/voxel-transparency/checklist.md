# Checklist

The design is in the [README](README.md). Check steps off as they land. Log
code-level choices in [implementation-decisions.md](implementation-decisions.md).

## Ground rules

- Each step is staged for review before the next starts.
- `voxrender`'s dependencies stay as they are. Nothing below voxsmith
  mentions a vxl flag.
- Every existing golden stays within tolerance at every step. Transparency
  is absent from them, and the rules reduce to today's output without it.
- A value that does not fit errors: an alpha or transmission outside
  `0..1`, an `ior` that is neither `0` nor `1..`.
- Tests are inline `#[cfg(test)] mod tests` per file. A golden PNG sits
  beside the test that embeds it.
- The contract page changes in the step that changes the behavior it
  describes, never ahead of it.

## Steps

- [x] **S1. Material fields.** `RenderMaterial.base_color` becomes
      `TyLinSrgbaF64`, and the struct gains `transmission` and `ior` with the
      vocabulary defaults.
      1. `check_material` in `render_scene.rs` checks the alpha and
         `transmission` as unit scalars and `ior` as finite and `0` or `1..`
      2. `material_key` dedupes on the three new values too
      3. `fill_color` keeps `baseColor`'s alpha and still drops
         `emissiveColor`'s. `fill_scalar` fills `TRANSMISSION` and `IOR`
      4. `normal_reflectance` in `render.rs` derives the dielectric
         reflectance from `ior`, replacing `DIELECTRIC_F0`. The shading reads
         the base color's `color`
      5. Tests: a transparent palette resolves, each range error fires, the
         reflectance is `0.04` at `1.5` and `1` at `0`, and the goldens hold
      6. The crate README's materials section and the contract's Materials
         section describe the fields, with `ior`'s effect
- [ ] **S2. The ray walk.** `cast_ray.rs` becomes a `RenderRayWalk`, an
      iterator over `RenderHit`s in distance order, with `cast_ray` as its
      first hit within a distance.
      1. One DDA state per placement carries the material the ray is inside.
         An entered cell whose material differs becomes the inside material
         and yields a hit when live. The slab's entry cell counts as entered
         when the ray starts outside the grid. A ray starting inside the grid
         starts inside its cell's material
      2. The walk peeks one hit per placement and yields the nearest
      3. Tests: a two-voxel bar of one material yields one hit and none at
         the seam, a bar of two materials yields two, a hollow box yields the
         near wall and the far wall, a ray from inside leaves its material
         first, two placements interleave by distance, and the existing
         `cast_ray` tests pass unchanged
      4. The crate README's rays section describes the walk. The contract's
         Surface section states the surface rule
- [ ] **S3. The pixel walk.** `render` composites each pixel's walk front
      to back into an image of light and transmittance.
      1. `RenderImage`'s pixel becomes a pair of `TyLinSrgbF32`s, the light
         and the transmittance. A miss has no light and full transmittance
      2. `shade_hit` scales the diffuse term by `1 - transmission` in
         `direct_radiance` and `hemisphere_radiance`, and returns the
         material's pass beside the color and the emission
      3. The pixel adds `throughput * alpha * shade` to its light and
         `throughput * alpha * emission` to the bloom buffer, multiplies the
         throughput by the pass, and stops at zero or at the walk's end. The
         remainder is the transmittance
      4. `apply_bloom` adds the halo to the light and scales the
         transmittance by one minus the halo's peak channel, clamped to one.
         The miss special case goes
      5. `RenderOutput::from_image` derives the alpha as the larger of one
         minus the peak transmittance and the peak of the light, clamped to
         one, and takes the light over it as the color. Under a background
         it adds the background scaled by the transmittance after the
         tonemap and clamps before the sRGB encode
      6. Tests: a clear pane over a colored wall shows the wall through one
         highlight, a two-thick pane matches a one-thick pane, a half-alpha
         voxel averages its shade and the wall, a transmissive emitter adds
         its emission at its coverage, a glass pixel over no background has
         the derived alpha, red glass over a white background is red, clear
         glass over white stays white, and every golden holds
      7. The contract's Shading and Output sections state the pass, the
         walk, and the output rules
- [ ] **S4. Shadows.** `shadow_factor` returns a `TyLinSrgbF64`.
      1. The shadow ray walks `RenderRayWalk` and multiplies the pass of each
         hit until the throughput is zero or the ray reaches its distance cap
      2. `bilinear` blends per channel for `per-corner`
      3. Each light multiplies its contribution by the color
      4. Tests: a red pane throws a red shadow on the floor, a two-thick pane
         shadows like a thin one, an alpha-zero voxel casts none, an opaque
         wall still blocks fully at every granularity, and the goldens hold
      5. The contract's Lights section states that a shadow ray transmits
         by the pass
- [ ] **S5. The glass fixture.** A `glass_scene` under `test_utilities` and
      a review render.
      1. The fixture: a floor, a red pane two voxels thick with the key light
         behind it so its tinted shadow falls on the floor, a clear block
         three voxels thick against a colored wall, a half-alpha voxel, and
         glass in front of the empty background. Goldens for the four
         variants
      2. A hand-authored `.voxj` glass asset in `submodules/tyt-assets`,
         rendered under `studio` at the three granularities, and a look at
         the images. The submodule change is its own commit there
      3. The crate README's rendering section mentions transparency
