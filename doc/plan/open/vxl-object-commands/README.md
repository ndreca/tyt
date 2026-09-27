# vxl object, object-voxels, and node commands

Status: **open.** The design was settled 2026-09-26 and is unimplemented. Steps
are in [checklist.md](checklist.md). Code-level choices are logged in
[implementation-decisions.md](implementation-decisions.md).

## Model

An object owns `name`, `bounds`, `origin`, layers, and voxels. `bounds` sizes
the build volume. `origin` places the build volume's min corner relative to the
placing node. A voxel's id is its raster index, so changing `bounds` or moving
voxels renumbers ids.

A node owns `name`, `transform`, child nodes, and child objects. `transform`
holds `position`, `rotation`, and `scale`. The hierarchy is a DAG, where a node
or object can have several parents. The root list acts as one more parent.

## Rules

1. What a command writes decides its group. `object` commands write object
   properties and move voxels along when needed. `object-voxels` commands write
   only voxels. `node` commands write nodes and hierarchy edges
2. Every command takes `<input> [output]`. The input can be any format voxconv
   reads. `--from` overrides the inferred format. The output is always voxj.
   `[output]` defaults to the input path with a `.voxj` extension, or `.voxjz`
   for a `.voxjz` input
3. Commands take the `to voxj` encoding options, `--ext`, and `--edit-state`.
   The voxj `ext` block carries the loaded ext
4. Every command except `node add` requires `--select <glob>` or
   `--select-index <index>`, which repeat and combine as they do today. They
   pick what the command acts on. `object` and `object-voxels` commands select
   objects, where a matched node selects every object below it. `node`
   commands select nodes, where a match selects that node alone and
   `--select-index` counts the node list. A node reached by several paths
   counts once
5. Every glob follows the shared
   [glob rules](../vxl-commands/reference/conventions.md#glob-patterns). `!`
   subtracts a match. As in git, an excluded node blocks re-including anything
   below it
6. `object set name`, `object reorder`, and `node set name` need exactly one
   match. Every other command applies to every match
7. `--select-parent <glob>` and `--select-parent-index <index>` pick the
   parent end of an edge. They repeat and combine like a `node` command's
   `--select`. The combined result must be exactly one node. A `node` command
   without them targets the root list
8. Vector values are separate arguments and accept negatives
9. A value that does not fit raises an error instead of being clamped or
   coerced

## Commands

Each usage follows `vxl <command> <input> [output]`. `<select>` stands for
`--select` and `--select-index`. `<parent>` stands for
`--select-parent <glob> | --select-parent-index <n>`.

| Command                   | Usage                                                                                          | Step |
| ------------------------- | ---------------------------------------------------------------------------------------------- | ---- |
| `node add`                | `--name <name> [<parent>]`                                                                     | S7   |
| `node link`               | `<select> [<parent>]`                                                                          | S7   |
| `node remove`             | `<select>`                                                                                     | S7   |
| `node set name`           | `<select> --name <name>`                                                                       | S6   |
| `node set position`       | `<select> --position <x> <y> <z>`                                                              | S6   |
| `node set rotation`       | `<select> --rotation <x> <y> <z> [--unit deg\|rad]` or `<select> --quaternion <x> <y> <z> <w>` | S6   |
| `node set scale`          | `<select> --scale <x> <y> <z>`                                                                 | S6   |
| `node unlink`             | `<select> [<parent>]`                                                                          | S7   |
| `object add`              | `--source <path> [--source-from <format>] <select> [<parent>]`                                 | S4   |
| `object duplicate`        | `<select> [<parent>]`                                                                          | S4   |
| `object link`             | `<select> <parent>`                                                                            | S4   |
| `object remove`           | `<select>`                                                                                     | S2   |
| `object reorder`          | `<select> --index <n>`                                                                         | S3   |
| `object set edit-bounds`  | `<select> --min <x> <y> <z> --max <x> <y> <z>`                                                 | S3   |
| `object set name`         | `<select> --name <name>`                                                                       | S3   |
| `object set origin`       | `<select> --origin <x> <y> <z>`                                                                | S3   |
| `object trim`             | `<select>`                                                                                     | S3   |
| `object unlink`           | `<select> <parent>`                                                                            | S4   |
| `object-voxels flip`      | `<select> --axis <x\|y\|z>`                                                                    | S5   |
| `object-voxels rotate`    | `<select> --axis <x\|y\|z> --turns <1\|2\|3>`                                                  | S5   |
| `object-voxels translate` | `<select> --offset <dx> <dy> <dz>`                                                             | S5   |

## Behavior

### `vxl object`

- `remove` detaches each selected object from every parent and releases it. It
  then releases each node the removal left childless and repeats the check on
  that node's parents. Palettes, value pools, and nodes that were already empty
  stay.
- `set origin` moves the grid and the voxels with it.
- `set edit-bounds` sets the build volume to the node-local box that
  `hierarchy show --show-edit-bounds` prints. It writes `origin = min` and
  `bounds = max - min` and shifts voxel grid coordinates so no voxel moves in
  the scene. It errors when a live voxel would fall outside.
- `reorder` moves the object to position `n` in the object list that
  `--select-index` counts.
- `trim` shrinks `bounds` to the live extent and moves `origin` by the extent's
  min corner, so no voxel moves in the scene. An empty object trims to
  `bounds = [0, 0, 0]`.
- `link` and `unlink` add or remove one placement edge. `link` errors when the
  edge exists, and `unlink` errors when it does not. An object left with no
  parents stays in the document unplaced.
- `duplicate` gives each copy the original's palettes and name. Every parent
  of the original also places the copy, unless `<parent>` gives one node
  instead.
- `add` copies the objects the selectors match in the source. Each source
  palette a copy references is appended once with its value pools, even when
  an equal palette exists. Each copy goes under `<parent>`, or else under a new
  root node named for the copy with an identity transform. The source's ext is
  not carried.

### `vxl object-voxels`

`object-voxels` commands never change `origin` or `bounds`.

- `translate` moves voxels within the grid and errors when a voxel would leave
  it.
- `flip` mirrors voxels in place: `p.a' = bounds.a - 1 - p.a`.
- `rotate` turns voxels in quarter turns about the grid's center. A positive
  turn follows the right-hand rule, so one turn about `y` maps
  `(x, y, z)` to `(z, y, bounds.x - 1 - x)`. One or three turns need the two
  turned dimensions to be equal. Otherwise the command errors and points at
  `node set rotation`.

### `vxl node`

- `set rotation --rotation` takes Euler angles in the `EulerRot::XYZEx` order
  that `hierarchy show --show-transforms` prints. `--unit` defaults to `deg`.
  `--quaternion` takes the stored form and errors when its length is off unit
  by more than the spec's tolerance.
- `set scale` errors on a zero component. A negative component mirrors that
  axis.
- `add` creates an empty node with an identity transform.
- `remove` detaches each selected node from every parent and releases it. It
  then releases each descendant node and object left with no parent.
- `link` and `unlink` add or remove one child edge. `link` errors when the edge
  exists or would close a cycle, and `unlink` errors when the edge does not
  exist. A node left with no parents stays in the document unplaced.

## Example

```sh
# Reads Voxel Max and writes scene.voxj beside it.
vxl object remove scene.vmax --select 'debris/**'

# Copies the first three source objects under the one node matching house.
vxl object add scene.voxj --source props.voxj --select-index 0-2 --select-parent house

# Swings the door node 37 degrees about y.
vxl node set rotation scene.voxj --select house/door --rotation 0 37 0

# Places the same door node under garage too.
vxl node link scene.voxj --select house/door --select-parent garage

# Turns a 4 x 8 x 4 crate a quarter turn in place.
vxl object-voxels rotate scene.voxj --select crate --axis y --turns 1

# Errors: a 4 x 8 x 1 door has unequal x and z.
vxl object-voxels rotate scene.voxj --select house/door/door --axis y --turns 1

# Grows the crate's edit box from [0 0 0]..[4 8 4] by 2 voxels of margin on -x.
vxl object set edit-bounds scene.voxj --select crate --min -2 0 0 --max 4 8 4
```

## voxcore hooks

In-place mutations fire no `VoxExt` hook today. A format ext's derived data
goes stale as a result: the vmax view box and camera, goxl placement positions,
and the mvox node kind. voxcore gains a setter and a hook for each field this
plan edits:

1. Object name, origin, and list order
2. A voxel remap that fires when an edit renumbers voxel ids
3. Node name, transform, and children setters in place of `set_hierarchy_node`
4. The root list

Each ext refreshes its derived data and keeps identity fields such as vmax
uuids. Layers, palette order, properties, and value pools get hooks when a
command first edits them.
