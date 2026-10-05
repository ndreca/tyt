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

The pass line runs on its own, and edits to the model file go through the
file-editing tool. A permission rule for `vxl` commands then covers every pass.
A pass chained after `sed`, `python`, or a `time` wrapper needs its own
approval. A scratch model that tries one call sits beside the model file and
runs through the same pass line because a write outside the working folder needs
its own approval too.

## Building a model

1. **Pick a voxel size from the smallest feature.** Every feature the prompt
   names spans at least 2 voxels. Six strings across a 5 cm neck take voxels of
   about 4 mm, and the guitar then runs about 250 voxels long. Models several
   hundred voxels across voxelize in minutes. A large scene under a voxel budget
   keeps its real size with `--fill-mode surface` or a coarser voxel
2. **Write real sizes in meters.** A seat 45 cm up sits at `y = 0.45`. At 2.5 cm
   per voxel, the seat lands 18 voxels up. A constant such as `const v = 0.025;`
   holds the voxel size for the details sized to the grid and matches
   `--voxel-size`
3. **Block out with boxes.** The first pass gives each large form one `box` with
   corners on multiples of `v`, even when the details are clear from the start.
   Each box takes one library material. Details written before the proportions
   settle get rewritten when the boxes move
4. **Face the subject toward the hero.** The `hero` view looks from the
   front-right-top. The detail the prompt is about faces +z or +x, and no post,
   roof, or wall stands between it and that corner. A room or a hull keeps its
   walls and takes a re-aimed or inside view from [Other views](#other-views)
5. **Voxelize at the chosen size.** Every pass sets `--voxel-size` to the size
   from step 1
6. **Review.** After each pass, the [checks](#checks) cover the report and then
   the four PNGs
7. **Fix the proportions.** Edits between the block-out passes move and resize
   the boxes until every view matches the prompt. A revision keeps the real
   sizes and the silhouette the earlier passes got right
8. **Detail and paint.** Later edits replace boxes with the shapes the forms
   need, cut openings, add the small details, and paint with materials and
   patterns. Each edit changes a few steps. Before each build, every new detail
   meets the [resolution](#resolution) rules
9. **Export.** A prompt that asks for a mesh gets one from [one more
   command](#exporting-a-mesh) after the last pass

## Choosing operations

The API already holds most of what a model builds by hand:

1. Gems on a pile, snow on a roof, and gilt on top faces take `coat` with
   `sides`. The coat finds the surface without a guessed height
2. Arms, spokes, and candles around a center take `repeatPolar`. Limbs, horns,
   and curled tips take `bend` or `twist`
3. A feature with nested layers, such as a flame's core inside its outer flame,
   takes one shape with a `gradient`. Nested shapes bury each other in any order
4. A pattern reads the frame its shape was built in. Flames built at the origin
   and moved into place each carry the whole `gradient`, and boards built that
   way streak with `grain`
5. A rim around a flat outline takes a 2D `offset` before the `extrude`. A 3D
   `offset` also pulls in the extrusion's ends
6. A `coat` or a `paint` also recolors a neighbor's live cells inside its reach.
   A coat with `within: torso` stays on the torso
7. A kit or a scene of repeated props places one part at each spot. Under
   `--frame local` every place then shares one object. A part built fresh per
   spot never shares, and neither does the default `--frame world`

## Checks

The report comes first because numbers read more reliably than pixels:

1. The bounds on the model's line have to match the prompt's real size in meters
2. A step at `0 cells` missed every cell center. A step at `0 kept` lost every
   cell to later steps. Either needs a fix before anything else unless later
   paints recolor that step on purpose
3. A detail at `0 exposed` sits buried inside another form
4. A second piece means a shape floats. The piece lines list the steps behind
   each piece
5. A part reading `detached` has a gap at its joint
6. A step's bounds can feed a `box` directly to line one form up with another

The PNGs come next, all four on every pass:

1. `front`, `right`, and `top` draw without perspective. The three views show
   the forms' heights, widths, symmetry, and alignment true to scale
2. `hero` shows whether the model reads as the prompt and whether the subject
   faces the camera
3. Every large face takes `shades`, `noise`, or a pattern. A terrain's cut sides
   take strata from `bands` or `gradient`. A face of one flat color reads
   unfinished
4. Neighboring materials need colors far apart in lightness or hue. Two dark
   tones or two metals side by side merge into one
5. Glass and water read faintly over the white background. A darker rim outlines
   clear glass that would otherwise vanish. Tinted glass tints what sits behind
   it and can turn a red potion mauve. Water darkens what sits inside it
6. A glow that reads pale or white where the prompt wants color takes a lower
   `emissiveStrength` or a darker `baseColor`

The model is done when every view shows the prompt and no flaw the review can
name. A flaw named in a review takes another pass. A fix counts once the next
pass's PNGs show it.

## Other views

The review views look at the whole model from outside. A close-up, another
angle, or a view inside a room comes from flags on the same render command:

```sh
# chair-hero.png from behind and to the left, beside the other review views
vxl object render chair.voxj
  --profile review
  --view-orbit hero 225 20 fit
  --to png

# robot-close.png with the head part alone
vxl object render robot.voxj
  --select 'robot/head'
  --view-orbit close 30 20 fit
  --to png
  --file-stem robot-close

# room-inside.png from eye height toward the fireplace
vxl object render room.voxj
  --view-frame inside world
  --view-position inside 1.2 1.6 1.4
  --view-look-at inside -0.3 0.9 -1.4
  --view-fov inside 60
  --to png
  --file-stem room-inside
```

1. `--view-orbit` places a view whole: an azimuth and an elevation in degrees,
   and a distance in meters or `fit`. It re-aims a profile view such as `hero`,
   and a new view name adds a PNG beside the review four
2. A view placed by position takes `--view-frame`, `--view-position`, and a
   rotation flag such as `--view-look-at`. `--view-angles` sets only the
   rotation and cannot re-aim `hero` alone
3. `--select` renders the matched parts alone. `--view-select <view> <glob>`
   keeps the whole model and frames the view on the matched parts

## Exporting a mesh

Every pass leaves the `.sdfj` and `.voxj` documents beside the model file. When
the prompt asks for a mesh, a glTF, or a `.glb` file, one more command writes
the mesh from the `.voxj` document:

```sh
# chair.glb with the materials baked into textures
vxl object mesh chair.voxj
  --profile pbr

# chair.glb with the gems, ice, and water kept transparent
vxl object mesh chair.voxj
  --profile glass
```

1. `pbr` bakes each material's color, metalness, roughness, and glow into
   textures
2. `glass` replaces `pbr` for a model with a transparent material. The gems,
   `ice`, `water`, and any material with `transmission` or a `baseColor` alpha
   below 1 are transparent
3. `--to gltf` writes a text `chair.gltf` with the textures embedded

The mesh keeps the model's size in meters and its parts as nodes.
