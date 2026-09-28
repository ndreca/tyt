# `vxl palette quantize`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette quantize <input> [output] --max-materials <n> [--index 0] [--property baseColor] [options]
```

Reduces a palette to at most `--max-materials` materials and rewrites every
layer that references the palette.

The input is a voxj/voxjz document, written back in its format. The
candidates are the materials some voxel samples, each weighted by its voxel
count. A material no voxel samples is dropped.

1. `--max-materials <n>` (required): the most materials the palette keeps.
2. `--index <n>` (default `0`): which palette to quantize.
3. `--property <key>` (default `baseColor`): which property to cluster on.
4. `--partition <property>`: materials merge only when they agree on this
   property. `--partition metallic` keeps metals and dielectrics apart. The
   flag repeats, and `'*'` stands for every property other than `--property`.
   Every partition keeps at least one material, so the command errors when
   partitions outnumber `--max-materials`.
5. `--method` `median-cut` | `octree` | `kmeans` (default `median-cut`):
   clustering algorithm.
6. `--space` `oklab` | `lab` | `srgb` (default `oklab`): distance metric used
   when clustering. Applies to `baseColor`.
7. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error
   diffusion when snapping values, walking each object's voxels in 3D order, not
   a 2D image. Every referencing object dithers.
   [`object voxels quantize`](../object/voxels/quantize.md) takes the object
   selectors.

Clustering runs on `--property`. Each cluster collapses onto a representative,
its most-sampled material. Every merged voxel takes all of the representative's
values. The representative is always a real material, never an average. A
material lacking the property passes through unmerged but still counts toward
`--max-materials`.

Surviving materials compact, which shifts material indices. The compaction also
prunes the value-pool values only dropped materials referenced.

The command runs [`object voxels quantize`](../object/voxels/quantize.md) with
`--shared` over every layer referencing the palette, then compacts.
[`mesh-doc voxelize`](../mesh-doc/voxelize.md)'s `--quantize-max-materials`
applies the same reduction inline. See
[Palettes](../../../../../../projects/voxel-formats/voxj/docs/voxel-json-file-format.md#palettes).
