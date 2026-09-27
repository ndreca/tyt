# vxl object downsample and upsample

Status: **open.** The design was settled 2026-09-27 and every step is built
and staged for review. The steps live in [checklist.md](checklist.md).
Code-level choices are logged in
[implementation-decisions.md](implementation-decisions.md).

## Model

An object's grid sits on its placing node's lattice: `origin` places the
grid's min corner in node-local voxels, and the node's scale sizes a voxel in
the scene. A resample by a whole-number factor swaps that lattice for one
`factor` times finer or coarser and re-expresses the object on it. The voxels
keep their node-local positions in the new units, `bounds` and `origin` scale
with the lattice, and the node is untouched. The object's size in the scene
therefore changes by the factor, and `node set scale` restores it.

## Rules

1. Both commands sit under `object`, since they write `bounds` and `origin`
   and move the voxels along
2. `--factor <n>` takes a whole number of at least 2 and applies on every
   axis. A factor of 1 changes nothing and errors
3. `upsample` is exact. Every voxel becomes an `n x n x n` block with its
   samples, and `bounds` and `origin` multiply by `n`
4. `downsample` merges the `n x n x n` blocks of the node lattice. The new
   grid covers the old box, with `origin` rounded down and the far corner
   rounded up to the coarse lattice, so an origin the factor does not divide
   grows the grid by one block on that side instead of moving the object.
   `object trim` tightens the grid afterwards
5. `--keep` decides whether a block is live by how many of its `n^3` cells
   are: `any`, `majority` (at least half, the default), or `all`. A cell
   outside the old grid counts as empty
6. A kept block takes the samples most of its live cells share, one material
   per layer. A tie goes to the earliest cell in raster order
7. `upsample` then `downsample` by one factor restores the object under every
   keep rule
8. A scaled grid past the u32 range or the cell cap, or a scaled origin past
   the i32 range, errors and writes nothing

## Commands

Each usage follows `vxl <command> <input> [output]`. `<select>` stands for
`--select` and `--select-index`.

| Command             | Usage                                                     |
| ------------------- | --------------------------------------------------------- |
| `object downsample` | `<select> --factor <n> [--keep <any\|majority\|all>]`     |
| `object upsample`  | `<select> --factor <n>`                                   |

## Example

```sh
# Halves the crate's resolution: a 10 x 10 x 10 grid at origin 0 becomes 5 x 5 x 5.
vxl object downsample scene.voxj --select crate --factor 2

# Doubles the crate node's scale so the coarser crate keeps its size in the scene.
vxl node set scale scene.voxj --select crate --scale 2 2 2

# Tenfold resolution: 10 x 10 x 10 becomes 100 x 100 x 100.
vxl object upsample scene.voxj --select crate --factor 10
```

## Design notes

1. Two verbs rather than one `resample`. A single command would name the
   direction by a fraction or a sign, and `--keep` applies only to merging.
   Each verb reads its direction off its name and carries only the flags that
   mean something to it.
2. The coarse lattice belongs to the node, not to the object's edit box. An
   edit box is an authoring margin that `object set edit-bounds` can grow at
   will, so anchoring blocks to its corner would let a margin change which
   voxels merge. Anchoring to the node lattice makes the result a function of
   where the voxels sit under the node, and keeps `node set scale` an exact
   inverse of the size change.
3. `majority` counts a tie as kept. With an even factor, a wall one voxel
   thick fills exactly half of each block it crosses, and a rule that dropped
   ties would erase every such wall.

## voxcore

A resample is neither a one-to-one remap nor a run of per-voxel retains: an
upsample fans one voxel out to many cells and a downsample folds many into
one. voxcore gains `VoxObject::resample_voxels(bounds, source)`, a pull-based
rebuild in which each new cell names the old cell it copies, the
`VoxMain::resample_object_voxels` setter over it, and the
`object_voxels_did_resample(old_bounds, old_voxel_ids)` hook. vmax moves the
camera target with the content center and goxl moves the stamps with the box,
as each does on a remap.
