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

## S2. Edit scaffolding

- S2 lands `object remove`, the one command the table marks S2, so the
  scaffolding has a caller.
- vxl's `edit_document(dependencies, input, output, edit)` loads with the ext,
  runs the edit closure, calls `gc`, and saves as voxj. It lives in the crate
  `internal` because the `object`, `object-voxels`, and `node` groups share
  it.
- `VoxjOutput` flattens `[output]`, `VoxjEncodingOptions`, `--ext`, and
  `--edit-state`. `to voxj` moves onto it, and `EditStateMode`'s `CliValue`
  moves to the crate `internal`.
- `resolve_output` takes the container from `--format`, else the output
  extension, else the input extension, else compact JSON. This covers
  `to voxj` too, so `to voxj scene.voxjz` now rewrites `scene.voxjz` instead
  of writing `scene.voxj`. `voxelize` reads a mesh, so it keeps compact JSON.
- The required variant is `RequiredSelection`, a clap group with
  `required = true, multiple = true`. It carries `resolve_objects` and
  `resolve_nodes`, each of which errors on an empty match. The help text
  covers both readings.
- `ParentSelection::resolve` returns `None` for the root list when no selector
  is given. `object link` and `object unlink` require a parent and will
  check for `None`.
- `ParentSelection` and `resolve_nodes` have no caller until S4 and S6. They
  carry `cfg_attr(not(test), expect(...))` marks that fail once a caller
  lands, so the marks cannot linger.
- `node_paths` moves out of `select_objects` into a crate-internal file that
  `select_nodes` shares. It also walks each node that neither the roots nor
  a node lists, taking its bare name as the path, so an unlinked node stays
  selectable. An object under such a node now matches through that node's
  path instead of by its bare name.
- `select_nodes` selects a node when the node's own path matches as a
  directory and no ancestor is excluded. `is_directory_path_match` would
  carry an ancestor's match down, which suits objects but not nodes.
- voxsmith gains an `object` feature for `operations/object/`.
  `remove_objects` sets the roots, then the child lists, then releases the
  nodes and the objects, each pass in listing order, so the hooks fire
  deterministically.
- voxsmith tests share a `HookRecorder` ext under a `cfg(test)`
  `test_utilities` module. It logs the node and object hooks.

## S3. Object properties

- `object set name`, `object set origin`, and `object reorder` each make one
  voxcore setter call per object, so vxl calls `set_object_name`,
  `set_object_origin`, and `move_object` directly. `trim_objects` and
  `set_edit_bounds` carry their own logic and live in voxsmith.
- `trim` and `set edit-bounds` call `remap_object_voxels` and then
  `set_object_origin`, so an ext sees the voxel remap under the old origin
  before the origin moves.
- `set edit-bounds` checks every live voxel against the node-local box before
  it remaps, so the error reports the box the user typed. A `max` below `min`
  errors. A zero-size axis is allowed.
- `trim` errors when the moved origin leaves the `i32` range. An object that
  is already tight still remaps and fires its hooks.
- Vector flags parse as `Vec<i32>` with `num_args = 3`,
  `allow_negative_numbers`, and `ArgAction::Set`, so a repeated flag errors.
  vxl's `vector3_i32` turns the values into a `TyVector3I32`, and vxl gains a
  `ty-math` dependency for it.
- `RequiredSelection::resolve_one_object` enforces the exactly-one rule for
  `set name` and `reorder`.

## S4. Object placement and copies

- `VoxObject`, `VoxPalette`, and `VoxValuePool` implement `Clone` by hand in
  voxcore. Their `IdField` columns hold non-`Copy` values, and branded-id
  clones an `IdField` only for `Copy` values, so `derive` does not compile.
  branded-id 0.1.11 adds `IdField::clone_retained`, which copies the value at
  each id the pool retains. A clone keeps its ids and holes, so material and
  value ids stay valid.
- branded-id is a submodule the workspace patches to, so voxcore builds
  against 0.1.11 before it publishes.
- `VoxObject::relabel_layer_palettes` and `VoxPalette::relabel_value_pools`
  are public and take a closure. They translate the ids that point into
  another main, and `gc` calls them with its remaps. `VoxMain` hands out only
  shared references, so a relabel reaches only a detached value and cannot
  skip a hook.
- `link_objects`, `unlink_objects`, `duplicate_objects`, and `add_objects`
  live in voxsmith. `add_objects` takes the source as a `&VoxState`.
- `link` appends to the parent's child objects. `unlink` leaves a childless
  parent in place, unlike `remove`.
- A copy from `duplicate` or `add` goes at the end of the object list and at
  the end of each placing node's child objects.
- `add` copies the referenced palettes and value pools in source listing
  order. The new root nodes for unparented copies are appended to the roots
  in one `set_root_hierarchy_node_ids` call.
- `ParentSelection::resolve_required` gives `link` and `unlink` a usage error
  when no parent selector is given.
- `object add` resolves `--source-from` inline, the way `VoxelInput` and
  `MeshInput` each do for `--from`.
- `HookRecorder` also logs `palette_did_retain`.
