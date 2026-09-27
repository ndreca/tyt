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
- vmax refreshes only on a voxel remap. The content center (`e_c`) depends on
  bounds and live voxels, not `origin`, and the build volume `vp` is already
  derived on write. The camera target `cam.o` moves by as much as the center
  moved, keeping the author's framing, rather than reframing on the center.
- goxl moves each affected placement by as much as its stamped box moved, for
  a node transform, an object origin, or an object bounds change. A children
  change drops the placements of unplaced objects and stamps each new object
  once. The placements group by child order only when the new order breaks the
  writer's first-stamped check, so an edit that keeps the order keeps the
  stored stamp order.
- mvox keeps a node's kind while its children still fit it, and otherwise
  gives it the kind and body a retained node would. A transform's first frame
  takes the rounded translation and, when it is a signed permutation, the
  node's rotation and scale. A frame the transform cannot become is left for
  the writer's existing check to report, since the output of these commands
  is voxj and an mvox-only limit should not block the edit.
- A setter hook refuses when the ext has no entry for the node or object it
  refreshes, since the ext was already out of step.
- The mvox writer ignores object `origin`, and the loader never sets one, so an
  `origin` or bounds change has nothing to refresh there. This is existing
  behavior outside the plan.
