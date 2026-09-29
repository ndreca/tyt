# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. voxsurface split

- `SurfaceGrid` answers solid-at-cell with a handle: `cell` returns
  `Option<Self::Cell>`, and the provided `is_solid` takes a signed position
  and treats the outside as empty. The handle is what the mesher keys faces
  by and records per face, so a `VoxObject` grid hands back voxel ids and
  voxsmith's swatch lookups stay as they were. voxsmith's `MeshGeometry` is
  the alias `SurfaceMesh<U32Id<BVoxVoxel>>`.
- `SurfaceMesh` records the `SurfaceSpan` of each quad beside the cells
  under it. The span settles the corner order, and `corner_occlusion` reads
  a span, so one function serves a merged quad and a hit face alike.
- A span's `corners` fixes the winding: `u` then `v` turn counter-clockwise
  about `+d`, so the `+` side runs `u` first and the `-` side `v` first.
  The mesher pushes vertices in that order and `mesh_occlusion` relies on
  it. The cross-product check the old `push_face` made is gone.
- `mesh_grid_keyed` drops the old `track_materials` switch and the
  `material_indices` it filled. Nothing in voxsmith read them.
- voxsmith re-exports `SurfaceMethod` as `Method`, `SurfaceSpan` as
  `FaceSpan`, and the alias above, so its records and vxl's flags are
  unchanged. The meshing and occlusion functions are called from
  `voxsurface` directly.
- voxsurface depends on `voxcore` without its `color` feature. voxsmith
  pulls it in under `object`.

## S2. voxrender scaffold

- `RenderLight` is one enum with a struct variant per kind. Each variant
  carries the part of a pose it reads: a rotation for `Directional`, a
  position for `Point`, nothing for `Hemisphere`.
- `RenderProjection` pairs the field of view with `Perspective` and the
  scale with `Orthographic`, so a view never carries the one its projection
  ignores. The field of view is in radians.
- `RenderMaterial` colors are `TyLinSrgbF64` and the image's pixels are
  `TyLinSrgbaF32`. The scene stays `f64` and narrows at the image.
- `RenderImage::new` accepts a zero side. The buffer is empty and harmless.
  The zero-side error belongs to the operation that asks for a render.
- The `cpu` feature is declared and empty until the DDA lands.

## S3. Render scene

- An entity that mirrors one voxcore entity takes its id. Objects are
  keyed by `BVoxObject` and cells by `BVoxVoxel`, which keeps the
  correspondence a sync layer against `VoxMain` edits would need. Nothing
  builds that layer yet. Materials, placements, lights, and views match
  nothing one to one and keep brands of their own. voxcore's
  `VoxObject::raster_id` and `raster_position` became public, so the render
  grid numbers cells with voxcore's formula instead of a copy.
- Objects sit in an `IdVec<BVoxObject, Option<RenderObject>>`, indexed by
  the caller's id. The other kinds mirror `VoxState`: an `IdStruct` beside
  an `IdField`, `release_stable` so survivors keep their order, and a
  `Drop` that releases every column. There is no `gc`, because nothing
  saves a scene.
- A placement's transform maps grid units straight onto world meters. The
  flatten folds the grid origin and the voxel size into it: the path's
  world position scales by the voxel size, and the placement's scale is the
  path's scale times the voxel size. The renderer then never sees an origin
  or a voxel size. Scaling every node position by the voxel size is one
  uniform scale of the whole path, so this matches `object mesh`.
- The path walk composes node transforms with `TyTransformF64::compose`,
  which is exact under uniform scale. A non-uniform scale under a rotated
  child would differ from a matrix composition.
- Placements come from the roots. An object only an orphan node lists is
  unplaced and gets the identity placement.
- A `RenderView` carries no subject placements. The subject is resolved
  where the transforms are, in voxsmith, and `subject_bounds` is a scene
  query over placement ids.
- `subject_bounds` frames the live extent, not the grid bounds, so empty
  margins of a grid never push the camera back.
- `RenderObject::set_voxel_material` is public and unchecked, so a
  standalone object can be filled before `retain_object` checks every
  reference. Once retained, voxels change through the scene's
  `set_voxel_material`, which checks the material.
- The material table deduplicates on the bit patterns of the six values.
- A voxel's material resolves through `VoxEffectivePalette::voxel_value`,
  one lookup per property per voxel. A scalar reads a float pool and a
  color a float vector pool, dropping alpha. Any other kind errors.

## S4. Transforms

- `TyQuaternionExt::from_look_direction` takes an explicit up and returns
  `None` for a zero direction or one along the up. The up rule of the
  contract, +Y unless the direction runs along +Y or -Y, then -Z, lives in
  voxsmith's `look_rotation`, because the fallback is the contract's choice
  and not math.
- `TyVector3Ext::from_azimuth_elevation` turns from +Z toward +X and then
  toward +Y, the direction the `angles` and `orbit` forms share. Both take
  degrees and pass through `TyAngleUnit`.
- `FIT_MARGIN` is five percent of the bounding radius. `fit_distance` fits
  the sphere into the vertical field of view on a square or wide image and
  into the narrower horizontal one on a tall image. `fit_scale` is the
  sphere's diameter with the margin.
- An orbit under an orthographic projection with a `fit` distance sits one
  `fit_scale` out from the center, past the sphere. The distance never
  changes what an orthographic image frames.
- The shapes are voxsmith enums with the README's names: `PoseTransform`,
  `RotationTransform`, `PositionTransform`, and `Rotation`. `FitOrFixed`
  is one enum for the orbit distance and the orthographic scale, and
  `ViewProjection` pairs each projection with the one length it reads, so a
  `fov` under `orthographic` is unrepresentable in voxsmith. vxl errors on
  that pairing while it builds the record.
- The resolvers take the `RenderElement` they report on. A `subject` or
  `orbit` frame over a subject with no voxel, a `fit` over one, and a
  look-at aimed at the entity's own position error on that element.
- A directional light's look-at needs a target because the light sits at
  its frame's origin.
- A `camera`-frame light composes with the view's resolved pose, so the
  render operation resolves it once per view.

## S5. DDA

- `cast_ray` moves the ray into each placement's grid units with the
  inverse rotation and a division by the scale. The direction keeps its
  length, which leaves the ray parameter in world distance. Hits in
  different placements compare by that distance directly.
- A hit's face is a unit `SurfaceSpan`, which `corner_occlusion` reads
  like a merged quad. The hit's `along` holds the fractions across the
  face's `u` and `v`, the coordinates the corner blend takes.
- A ray that starts inside the grid skips its starting cell. A camera
  inside a voxel sees out of it. A shadow ray cast from a face cannot hit
  that face's voxel through rounding.
- The entry cell is the entry point floored and clamped into the grid. A
  ray that enters on a face lands in the cell behind that face. A solid
  entry cell reports its hit on the slab's entry axis.
- `RenderViewRays` lowers a view's projection once per render into pixel
  (0, 0)'s ray plus per-pixel steps for the origin and the direction.
  Perspective zeroes the origin steps and orthographic zeroes the direction
  steps. Building a pixel's ray takes no branch on the projection and
  recomputes no per-view constant. The contract keeps `RenderProjection`
  unchanged.
- The `cpu` feature gates `RenderViewRays`, `cast_ray`, `RenderRay`, and
  `RenderHit`. The GPU crate computes its own hits.

## S6. Shading

- Shading math runs on `TyLinSrgbF64` through palette's component
  arithmetic. The image narrows to `f32` at `set_pixel`.
- The hemisphere term reflects off the diffuse color plus the
  normal-incidence reflectance. The reflectance keeps a metal under
  ambient light alone from going black: the metal reflects the sky in its
  base color. glTF fixes the BRDF and leaves the ambient approximation to
  the renderer.
- A floor of `1e-3` on GGX alpha keeps a roughness of zero from producing
  a NaN at the reflection.
- The corner occlusion darkens only the hemisphere term, as glTF's
  occlusion texture does. The material's occlusion strength scales it.
- A shadow sample starts `1e-4` grid units out along the face normal. A
  per-pixel or per-corner sample also sits at least `1e-3` of the face in
  from its edges. Every sample starts in the empty cell in front of its
  face, and a corner ray still meets a solid neighbor across the edge.
- `bilinear` serves both the per-corner shadow blend and the corner
  occlusion blend. It reads values in the span's winding order.
- A point light's shadow ray ends at the light, so an occluder past the
  light casts nothing.
- `render.rs` is the only caller of the shading and shadow helpers, so
  they live there as private functions with their tests.

## S7. Output encoding

- `tonemap` implements the published Khronos PBR Neutral curve at the
  image's `f32` precision. A mid gray keeps its value less the flare
  offset. Only a color whose brightest channel passes `0.8` compresses.
- `RenderOutput::from_image` composites the background after the tonemap
  and the transfer. The caller's sRGB byte background lands in the file
  unchanged.
- `RenderOutput` and `tonemap` stay outside the `cpu` feature because a
  GPU tier reads back a linear image and encodes it the same way.

## S8. Profiles

- The profile types mirror the README's TypeScript with serde:
  1. The shapes and rotation forms tag on `kind` in kebab case
  2. Every struct denies an unknown key
  3. Every element is optional, so a stack member can set a view's
     projection alone
  4. The entry types carry an `Entry` suffix, as the mesh profile's do
- Each field's type rejects a bad value at load:
  1. `width` and `height` are `NonZeroU32`
  2. Strengths are `NonNegativeF64`
  3. Ranges, distances, fields of view, and scales are `PositiveF64`
  4. Colors are `SrgbColor`, parsed from a `#RRGGBB` hex
- The record builder checks the field of view's upper bound.
- Views stack by name, element by element. The lights stack as one
  element, the rig. Stacking two rigs errors instead of merging lights by
  position. A view set and a rig compose.
- `viewsFrom` and `lightsFrom` resolve as the stack lands, the way mesh's
  program builder resolves `valuesFrom`. A cycle errors when a stack
  reaches it. The claims track the profile that set each element, so an
  error reports the imported profile itself.
- Each profile's views and rig land once. A stack of `turnaround` and
  `hero` lands `hero`'s view once instead of erroring on a collision with
  itself.
- `lightsFrom` takes a list for symmetry with `viewsFrom`. Only one entry
  can bring a rig because a rig is one element.
- `studio` sets no shadow granularity, so it takes the record builder's
  default. That default is the value's one home, which S12 changes.
- `flat` has a strength of pi. Under a head-on Lambert term, pi renders a
  white base color as white.
- vxl reaches the voxrender enums through voxsmith's `object_render`
  re-exports, so vxl still speaks voxsmith only.
- The mirroring flags land with the `object render` command in S10
  because a flag without a command has no parser to test.
