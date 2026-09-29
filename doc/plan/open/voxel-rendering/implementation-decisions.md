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
