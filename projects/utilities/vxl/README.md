# vxl

A command-line tool for working with voxels.

## Editing

The `object`, `object-voxels`, and `node` commands edit a document. Each reads
any format voxconv reads and writes Voxel JSON, beside the input by default.
`--select` takes a hierarchy-path glob. `--select-index` takes an index or a
range. Both repeat and pick what the command acts on. `--select-parent` and
`--select-parent-index` pick the one node at the parent end of an edge.

```sh
# Reads Voxel Max and writes scene.voxj beside it.
vxl object remove scene.vmax --select 'debris/**'
```

## Objects

`object` commands write object properties and move the voxels along when
needed. `set origin` moves an object's grid within its node. `set edit-bounds`
and `trim` resize the grid without moving a voxel in the scene. `link` and
`unlink` add or drop one placement. `duplicate` copies objects within the
document. `add` copies objects from another file along with their palettes.

```sh
# Copies the first three props under the one node matching house.
vxl object add scene.voxj --source props.voxj --select-index 0-2 --select-parent house

# Grows the crate's edit box from [-2 -6 0]..[2 2 4] by 2 voxels on -x and -z.
vxl object set edit-bounds scene.voxj --select crate --min -4 -6 -2 --max 2 2 4
```

## Object Voxels

`object-voxels` commands move voxels within the grid and never change `origin`
or `bounds`. `translate` shifts the voxels, `flip` mirrors them, and `rotate`
turns them in quarter turns that follow the right-hand rule. One or three turns
need the two turned dimensions to be equal. `node set rotation` turns any
object.

```sh
# Turns the 6 x 8 x 6 crate a quarter turn about y.
vxl object-voxels rotate scene.voxj --select crate --axis y --turns 1
```

## Nodes

`node` commands write nodes and hierarchy edges. `set` writes a node's name,
position, rotation, or scale. `set rotation` takes Euler angles in the order
`hierarchy show --show-transforms` prints them, in degrees by default. `add`
creates an empty node. `link` and `unlink` add or drop one child edge. Without
a parent selector, `add`, `link`, and `unlink` act on the root list. `remove`
releases nodes along with each descendant left without a parent.

```sh
# Swings the door node 37 degrees about y.
vxl node set rotation scene.voxj --select house/door --rotation 0 37 0

# Places the same door node under garage too.
vxl node link scene.voxj --select house/door --select-parent garage
```
