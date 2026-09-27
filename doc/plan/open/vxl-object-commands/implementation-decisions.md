# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. voxcore hooks

S1 lands in two commits: voxcore setters, hooks, and forwarding first, then the
vmax, goxl, and mvox refreshes.

- Setters fire `did` hooks after the mutation with the value they replaced:
  `object_name_did_set(old_name)`, `object_origin_did_set(old_origin)`,
  `object_did_move(old_index)`, `hierarchy_node_name_did_set(old_name)`,
  `hierarchy_node_transform_did_set(old_transform)`,
  `hierarchy_node_children_did_set(old_child_node_ids, old_child_object_ids)`,
  and `root_hierarchy_node_ids_did_set(old_root_ids)`. The new value is on the
  state, so an ext can refresh by the change rather than rebuild.
- One `set_hierarchy_node_children` sets both child lists, since an ext such
  as mvox reads a node's kind off both at once. It reports node-keyed errors
  (`ChildNode`, `DuplicateChildObject`, `Cycle`) instead of the batch-indexed
  `Inserted*` ones `set_hierarchy_node` used. `validate` shares the checks.
- Both root setters fire the roots hook.
- The voxel remap setter is `VoxMain::remap_object_voxels(object_id, bounds,
  position)` over `VoxObject::remap_voxels`. `position` maps an old grid
  position to a signed new one, so trim, edit bounds, translate, flip, and
  rotate all ride it. It errors on a voxel landing outside the grid or two
  landing on one cell. `origin` is left to `set_object_origin`.
- `object_voxels_did_remap` passes the old bounds and a `HashMap` of old to new
  id per live voxel, the shape `materials_did_repaint` uses.
  `branded_id::soa::IdRemap` has no public constructor.
- voxconv takes `ty-math` as an optional dependency of its `ext` feature, for
  the hook signatures.
