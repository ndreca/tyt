# voxsurface

The boundary of a voxel grid: its faces meshed into quads and the occlusion
at their corners. voxsmith meshes objects through it and voxrender shades
through it. Everything stays in grid units on the grid's axes. The caller
applies the voxel size and the placement.

## The grid

Every algorithm reads its grid through `SurfaceGrid`: the bounds and the
cell at a position. A solid cell has a handle, the `Cell` type, and an empty
cell has none. `is_solid` takes a signed position and treats the outside as
empty. A `VoxObject` is a grid whose handles are its voxel ids.

```rust
let object = VoxObject::new("crate".to_owned(), TyVector3U32::new(2, 1, 1))?;

assert!(!object.is_solid([2, 0, 0]));
```

## Meshing

`mesh_grid` triangulates a grid by a `SurfaceMethod`. A solid cell at
`(x, y, z)` fills the unit cube from `(x, y, z)` to `(x + 1, y + 1, z + 1)`.
The `SurfaceMesh` carries one position and one normal per vertex, triangle
indices, and per quad the `SurfaceSpan` it covers beside the handles of the
cells under it. Every quad has four vertices of its own, so shading stays
flat, and triangles wind counter-clockwise seen from outside.

```rust
let mesh = mesh_grid(&object, SurfaceMethod::Greedy);

assert_eq!(mesh.quad_count(), 6);
```

`mesh_grid_keyed` adds a per-cell key and a span rule to a greedy run. A
span merges only cells that share a key and grows only while the rule
accepts it.

```rust
let keyed = mesh_grid_keyed(
    &object,
    SurfaceMethod::Greedy,
    &|voxel_id| voxel_id.to_u32(),
    &|span| (span.u1 - span.u0) * (span.v1 - span.v0) <= 4,
);
```

## Occlusion

`corner_occlusion` reads how open each corner of a span's face is, in the
order of the span's `corners`: `1` fully open. Each of the three cells
beside the corner in the layer the face looks into closes a third, and both
cells along the face's edges together close it fully. `mesh_occlusion`
applies it to every quad of a mesh, one value per vertex.

```rust
let occlusion = mesh_occlusion(&object, &mesh);

assert_eq!(occlusion.len(), mesh.positions.len());
```
