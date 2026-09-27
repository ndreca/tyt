# Checklist

The design is in the [README](README.md). Each step builds the commands the
[table](README.md#commands) marks with that step. Check steps off as they land.

## Ground rules

- Voxsmith operations are generic over `VoxExt` and live under
  `operations/object/`, `operations/object_voxels/`, and `operations/node/`.
  vxl stays a thin clap layer shaped like `commands/palette/`.
- Each voxsmith operation gets unit tests, including hook firing through a
  recording ext. Each vxl command gets a parse test.

## Steps

- [x] **S1. voxcore hooks.** Add the setters and hooks from
      [voxcore hooks](README.md#voxcore-hooks) to `VoxMain` and `VoxExt`.
      Forward the hooks in voxconv's `CompositeVoxExt`. Make the vmax, goxl, and
      mvox exts refresh on them. `set_object_origin` and `move_object` exist
      and only need hooks. Move voxsmith's `keep_objects` onto the children
      setter.
- [x] **S2. Edit scaffolding.** Model the load-edit-gc-save helper on vxl's
      `commands/to/internal/convert.rs`. Give `ObjectSelection` a required
      variant that node commands reuse. Add the `<parent>` group. Build
      `select_nodes` beside voxsmith's `select_objects`, reusing its node
      paths. The voxj output reuses `VoxjEncodingOptions`. Its `resolve_output`
      defaults to compact JSON today and must default to the input's
      container instead.
- [x] **S3. Object properties.** `trim` and `set edit-bounds` renumber voxel
      ids and fire the voxel remap hook.
- [x] **S4. Object placement and copies.** `VoxObject` has no `Clone`. Derive
      it or rebuild copies through `new`, `retain_layer`, and `retain_voxel`,
      and log the choice. `add` remaps
      palette, material, and value pool ids from the source.
- [x] **S5. `object-voxels`.** Tests check that four turns and two flips leave
      an object unchanged.
- [x] **S6. Node setters.** `glam`'s `DQuat::from_euler(EulerRot::XYZEx, ...)`
      inverts the `to_euler_radians` that `hierarchy show` prints with.
- [ ] **S7. Node graph.** `link` reuses voxcore's cycle check.
- [ ] **S8. Docs.** Add vxl README sections and extend the command list in the
      [vxl-commands README](../vxl-commands/README.md#commands).
