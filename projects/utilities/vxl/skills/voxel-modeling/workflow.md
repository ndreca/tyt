# Voxel modeling

A voxel model starts as a TypeScript model file that describes shapes and
materials in meters. Each pass records the model file as an `.sdfj` document,
voxelizes the document into voxj with a report, and renders four PNGs. The
report and the PNGs show what the next edit fixes. The modeling API below lists
every call a model file can make.

## A pass

One line runs a pass over `chair.ts`:

```sh
vxl sdf-doc build chair.ts --library materials && vxl sdf-doc voxelize chair.sdfj --voxel-size 0.025 --report && vxl object render chair.voxj --profile review --to png
```

The commands share one line because a pass that skips the build voxelizes a
stale `.sdfj` document. The line stops at the first error. A pass writes
`chair.sdfj`, `chair.voxj`, `chair-front.png`, `chair-right.png`,
`chair-top.png`, and `chair-hero.png` beside the model file.

## Building a model

1. **Pick a voxel size.** Most models read best 16 to 64 voxels across. At 2.5
   cm per voxel, a chair about 1 m tall stands 40 voxels high
2. **Write real sizes in meters.** A seat 45 cm up sits at `y = 0.45`. At 2.5 cm
   per voxel, the seat lands 18 voxels up. A constant such as `const v = 0.025;`
   holds the voxel size for the details sized to the grid
3. **Block out with boxes.** The first draft gives each large form one `box`
   with corners on multiples of `v`. Each box takes one library material
4. **Voxelize at the chosen size.** Every pass sets `--voxel-size` to the size
   from step 1
5. **Review.** After each pass, the [checks](#checks) cover the report and then
   the four PNGs
6. **Fix the proportions.** Edits between the block-out passes move and resize
   the boxes until every view matches the prompt
7. **Detail and paint.** Later edits replace boxes with the shapes the forms
   need, cut openings, add the small details, and paint with materials and
   patterns. Each edit changes a few steps

## Checks

The report comes first because numbers read more reliably than pixels:

1. The bounds on the model's line have to match the prompt's real size in meters
2. A step at `0 cells` missed every cell center. A step at `0 kept` lost every
   cell to later steps. Either needs a fix before anything else
3. A detail at `0 exposed` sits buried inside another form
4. A second piece means a shape floats. The piece lines list the steps behind
   each piece
5. A part reading `detached` has a gap at its joint
6. A step's bounds can feed a `box` directly to line one form up with another

The PNGs come next:

1. `front`, `right`, and `top` draw without perspective. The three views show
   the forms' heights, widths, symmetry, and alignment true to scale
2. `hero` shows whether the model reads as the prompt, whether neighboring forms
   take distinct materials, and whether a large surface reads flat. A flat
   surface takes `shades` or a pattern
