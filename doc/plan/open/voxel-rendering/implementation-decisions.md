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
