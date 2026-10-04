# Checklist

The design is in the [README](README.md). Check steps off as they land. Log
code-level choices in [implementation-decisions.md](implementation-decisions.md).

## Ground rules

- Each step is staged for review before the next starts.
- The contract page changes in the step that changes the behavior it
  describes, never ahead of it.
- Every golden without glass stays within tolerance at every step. A step
  that changes a rule regenerates the goldens it moves and lists them.
- Tests are inline `#[cfg(test)] mod tests` per file.

## Steps

- [x] **S1. Transparent output.** `RenderOutput::from_image` solves the
      transparent PNG to be exact over white for sRGB blending.
      1. The coverage and the layer stay as they are, and the background
         branch is unchanged
      2. The transparent branch derives `W`, `t`, the alpha, and the color
         as the README says
      3. Tests: an opaque pixel and a miss encode as today. Red glass,
         clear glass, a pane, a highlight, and a halo laid over white in
         sRGB values match `Some(white)` within a level. A colorless
         transmittance `T` gives an alpha of `1 - T`
      4. The goldens regenerate with a list of what moved
      5. The contract's Output section and the crate README's Images
         section state the rule
- [x] **S2. Exits.** `RenderRayWalk` yields an exit where the ray leaves a
      material for an empty cell, the outside of the grid, or a material
      that is not opaque.
      1. A `RenderHit` says whether it is an exit. An exit's face is the one
         the ray leaves by, and the shading turns its normal back along the
         ray
      2. An exit comes before the entry at the same face. The material a ray
         starts inside leaves without one
      3. The pixel walk and the shadow walk take every exit's coverage and
         pass. The corner occlusion of an exit reads the glass side
      4. Tests: a pane yields two hits, a slab of one material two, glass on
         an opaque floor one, a ray from inside none for its start material,
         an exit at the grid's edge, a half-alpha voxel covering three
         quarters, and a slab shadowing by its pass squared
      5. The goldens regenerate with a list of what moved
      6. The contract's Surface and Shading sections and the crate README's
         Rays section state exits
- [x] **S3. Fresnel by angle.** The transmitted share takes Schlick's
      Fresnel on the ray's cosine, capped by the roughness.
      1. The pass takes the cosine. A ray that starts inside a material
         takes the pass at normal incidence, and opacity stays the pass at
         normal incidence being zero
      2. The hemisphere's reflection off the transmitted share mixes sky and
         ground by the reflected direction and scales by the Fresnel
      3. Tests: the pass at normal incidence matches voxcore's, a grazing
         ray passes less, a material without transmission shades as before,
         and the goldens without glass hold
      4. The glass goldens regenerate with a list of what moved
      5. The contract's Shading section states the Fresnel
- [ ] **S4. Double-sided glass.** The `glass` built-in profile writes
      `doubleSided` on its blended material.
      1. The built-in profiles and the profile reference both carry the slot
      2. Tests over the expansion
- [ ] **S5. Review and close.** Rerender the review's scenes from main's
      `vxl` and compare them with the picked prototype over each backdrop.
      The follow-ups plan records the plan closed.
