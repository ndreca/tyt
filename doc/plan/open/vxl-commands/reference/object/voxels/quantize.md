# `vxl object voxels quantize`

*Part of the [Vxl Command-Line Reference](../../../README.md).*

```
vxl object voxels quantize <input> [output] --max-materials <n> [options]
```

Rewrites the selected objects' voxel samples so each selected layer samples at
most `--max-materials` materials of its palette. Palettes and value pools stay
untouched, which keeps material indices stable and leaves unselected objects
unchanged.

The candidates are the materials a layer samples, each weighted by its voxel
count. Clustering runs on `--property`. Each cluster snaps onto a
representative, its most-sampled material. Every merged voxel takes all of the
representative's values.

1. `--max-materials <n>` (required unless a profile sets it): the most materials
   each selected layer may sample afterward.
2. `--select <glob>`: quantize the objects a hierarchy-path glob selects. A
   node path selects its subtree. The flag repeats and unions with
   `--select-index`. See [Object selectors](../../conventions.md#object-selectors).
3. `--select-index <index>`: quantize the objects at an index, an integer or an
   `a-b` range. The flag repeats and unions with `--select`. Without either
   selector, every object is quantized.
4. `--layer-index <index>`: quantize the layers at an index into each selected
   object's `layers`, an integer or an `a-b` range. The flag repeats. Without
   it, every layer whose palette binds `--property` is quantized. The command
   errors when a selected object lacks a named layer, a named layer's palette
   lacks `--property`, or no selected layer's palette binds it.
5. `--shared` (default `false`): cluster together every selected layer that
   references one palette. The whole selection then shares at most
   `--max-materials` materials per palette. Without `--shared`, each object
   clusters separately.
6. `--property <key>` (default `baseColor`): which property to cluster on.
7. `--interpret-property` (default `auto`) and `--alpha` (default
   `partition`): how the property's values read as points and how a color's
   alpha takes part. See
   [`palette quantize`](../../palette/quantize.md).
8. `--partition <property>`: materials merge only when they agree on this
   property. The flag repeats, and `'*'` stands for every property other than
   `--property`.
9. `--method`, `--space`, and `--dither`: the clustering controls shared with
   [`palette quantize`](../../palette/quantize.md), defaulting the same way
   (`median-cut`, `oklab`, `none`). `--dither` walks each object's voxels in 3D
   order.
10. `--profile <profile>`: apply a saved reduction recipe. The flag repeats. See
    [Profiles](#profiles).

A layer that already samples at most `--max-materials` materials stays
unchanged.

## Profiles

Profiles live at `object.voxels.quantize.profiles` in a `.vxlconfig`, apart from
`palette quantize`'s. `vxl profile object voxels quantize list` prints them.
These profiles take the elements and `--profile` rules of
[`palette quantize` profiles](../../palette/quantize.md#profiles). `--select`,
`--select-index`, `--layer-index`, and `--shared` stay on the command line
because they pick the run's target.
