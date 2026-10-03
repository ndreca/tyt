# Modeling API

_Part of the [voxel modeling plan](../README.md)._

A model file describes a voxel model in TypeScript as signed distance functions
measured in meters. Its code runs once and returns a list of steps.
`vxl sdf-doc build` records the steps as an `.sdfj` document.
`vxl sdf-doc voxelize` samples the `.sdfj` document on a voxel grid and writes
voxj. This page lists everything a model file can use and stands alone as the
context for writing one. [Model evaluation](model-evaluation.md) sets exactly
what each call computes.

## Running

```sh
# chair.sdfj, then chair.voxj at 2.5 cm per voxel
vxl sdf-doc build chair.ts
vxl sdf-doc voxelize chair.sdfj
  --voxel-size 0.025
  --report
```

`vxl sdf-doc build` runs the model file and writes its steps and parts as an
`.sdfj` document. An edit to the model file takes effect at the next build.

1. `[output]` sets the document's path. The path defaults to the model's path
   with an `.sdfj` extension
2. `--runtime <runtime>` runs the model under `node`, `bun`, or `deno` and
   defaults to `node`. The runtime has to be installed
3. `--profile <profile>` applies saved flags from a [profile](#profiles)

`vxl sdf-doc voxelize` samples the document on a [voxel grid](#coordinates) and
writes a voxj document. Its flags follow
[`vxl mesh-doc voxelize`](../../vxl-commands/reference/mesh-doc/voxelize.md) and
take the values that page lists:

1. `[output]` sets the voxj document's path. The path defaults to the input's
   path with a `.voxj` extension, or `.voxjz` under `--format zip`
2. `--voxel-size <meters>` sets the edge of a cell and defaults to 1 meter
3. `--resolution <reference> <n>` sets the voxel size to a reference side of the
   model divided by `n` and excludes `--voxel-size`
4. `--fill-mode surface` keeps only the cells that show as a hollow shell one
   cell thick. The default `--fill-mode solid` keeps every filled cell
5. `--flatten` writes a model's [parts](#parts) as one object, and
   `--flatten false` turns flattening off
6. `--report` prints the [report](#report), and `--report false` turns it off
7. `--profile <profile>` applies saved flags from a [profile](#profiles)
8. `--format`, `--encoding-preset`, `--position-encoding`, and
   `--sample-encoding` choose the container and the block encodings

For `--resolution`, world references such as `longest-world` and `world-y`
measure the whole model, and object references measure each part.

```sh
# chair.voxj at 32 cells across its widest side
vxl sdf-doc voxelize chair.sdfj
  --resolution longest-world 32
```

### Profiles

A profile holds a command's flags under a name in a `.vxlconfig`. A batch of
models can then share one profile's settings:

```jsonc
{
  "sdfDoc": {
    "build": {
      "profiles": {
        "bun": { "description": "Models under Bun", "runtime": "bun" },
      },
    },
    "voxelize": {
      "profiles": {
        "props": {
          "description": "Props at 2.5 cm per voxel",
          "voxelSize": 0.025,
          "report": true,
        },
        "icons": {
          "resolution": { "reference": "longest-world", "count": 16 },
          "flatten": true,
        },
      },
    },
  },
}
```

```sh
vxl sdf-doc voxelize lantern.sdfj
  --profile props
```

A build profile takes `runtime`. A voxelize profile takes `resolution` or
`voxelSize`, `fillMode`, `flatten`, and `report`. Either takes an optional
`description`. Each flag key takes its flag's values. The output and the
encoding flags stay on the command line. A flag on the command line overrides
the profile, and either size flag replaces both `resolution` and `voxelSize`.
The `.vxlconfig` files load in
[`mesh-doc voxelize`'s cascade](../../vxl-commands/reference/mesh-doc/voxelize.md#profiles).

## Model files

A model file is a TypeScript module. Its default export lists the model's steps
and parts. `vxl sdf-doc build` puts every name on this page in scope, and a
model imports none of them:

```ts
export default [
  add("block", box([0, 0, 0], [0.1, 0.1, 0.1]), mat.stone),
];
```

Variables, functions, and loops shape the list, but nothing in a model runs per
voxel. Shapes, steps, and patterns only describe what vxl samples later. A model
holds no voxel size.

A larger model can split across files. A file imports another by its relative
path with the `.ts` extension. Every file sees the names on this page.

## Coordinates

The axes are glTF's: +Y up, +X right, and +Z toward the viewer. A model faces
+Z. Positions, distances, and radii count meters, and angles count degrees. A
positive rotation follows the right-hand rule: counterclockwise when seen from
the positive end of its axis.

`vxl sdf-doc voxelize` samples the model on a lattice of cubes one voxel size
across. The cell corners sit on whole multiples of the voxel size. A cell lies
inside a shape when the shape's signed distance reads zero or less at the cell's
center.

1. A box with corners on multiples of the voxel size fills exactly the cells
   between them. At `--voxel-size 0.025`, `box([0, 0, 0], [0.05, 0.25, 0.05])`
   fills 2 x 10 x 2 cells.
2. A shape centered on a cell corner spans an even number of cells, and one
   centered on a cell center spans an odd number. A model symmetric about
   `x = 0` therefore gives its center-line features even widths.
3. The model's `[0, 0, 0]` lands on the document's root node. A model centered
   on x and z with its base at `y = 0` stands on its pivot.
4. Real sizes go in as they are: a seat 45 cm up sits at `y = 0.45`. Details
   sized to the grid take multiples of one voxel size held in a constant such as
   `const v = 0.025;`. The model then reads best at `--voxel-size 0.025`.

## Types

```ts
type Vec2 = [number, number];
type Vec3 = [number, number, number];
type Axis = "x" | "y" | "z";
type Axes = "x" | "y" | "z" | "xy" | "xz" | "yz" | "xyz";
type Side = "+x" | "-x" | "+y" | "-y" | "+z" | "-z";
```

A `Shape` holds a 3D region, and a `Shape2` holds a 2D region in a plane with
axes u and v. A `Material` holds surface properties, a `Pattern` picks a
material per cell, and a `Step` makes one entry of the model's list. A `Part`
groups steps into an object that turns about a joint. These values never change:
a method returns a new value and leaves its receiver alone. One shape can
therefore feed several steps.

## Steps

```ts
function add(name: string, shape: Shape, material: Material | Pattern): Step;
function carve(name: string, shape: Shape): Step;
function paint(name: string, shape: Shape, material: Material | Pattern): Step;
function coat(name: string, material: Material | Pattern, options?: CoatOptions): Step;
function set(name: string, points: Vec3 | Vec3[], material: Material | Pattern): Step;

interface CoatOptions {
  sides?: Side[];
  depth?: number;
  within?: Shape;
}
```

The steps apply in list order to one grid, and a later step wins wherever two
reach the same cell.

1. `add` fills the shape's cells with the material and replaces what they held.
2. `carve` empties the shape's cells.
3. `paint` recolors the live cells inside the shape and fills none.
4. `coat` recolors a live cell when one of the `depth` cells beyond it toward a
   listed side is empty. The coat reads the grid as the earlier steps left it.
   `sides` defaults to all six. `depth` counts cells and defaults to 1. `within`
   limits the coat to a shape's cells.
5. `set` fills the cell holding each point and places detail too small for a
   shape.

Every step takes a name no other step in the model shares. The report and errors
use the name. The grid grows to hold every `add` and `set`, and `carve`,
`paint`, and `coat` never grow it.

## Parts

```ts
function part(name: string, options: { pivot: Vec3 }, steps: (Step | Part)[]): Part;
```

A part groups steps into one voxel object that turns about `pivot`. A part in
another part's list becomes that part's child. Arms, wings, hands, wheels, and
lids take parts when the prompt asks for pieces that move or separate objects.

1. The steps run in list order over one grid, and each part's list runs in its
   place. A part changes no step's effect.
2. A cell belongs to the part whose `add` or `set` last filled it. `paint` and
   `coat` recolor cells in any part and keep the cells' owner, and `carve`
   empties cells in any part.
3. The steps outside every part belong to the root part. The root part takes the
   model file's stem as its name and turns about `[0, 0, 0]`.
4. Each part writes its own object under a node at its pivot, and the nodes nest
   as the parts do. Turning a part's node turns the part and its children about
   the joint.

Every part takes a name no other part shares. A function builds symmetric parts:

```ts
// A figure whose arms turn at the shoulders and hands at the wrists.
const arm = (side: string, x: number) =>
  part(`arm.${side}`, { pivot: [x, 0.525, 0] }, [
    add(`${side} arm`, capsule([x, 0.525, 0], [x * 2.5, 0.525, 0], 0.0375), mat.leather),
    part(`hand.${side}`, { pivot: [x * 2.75, 0.525, 0] }, [
      add(`${side} hand`, sphere([x * 3, 0.525, 0], 0.0375), mat.bone),
    ]),
  ]);

export default [
  part("body", { pivot: [0, 0.3, 0] }, [
    add("torso", box([-0.1, 0.3, -0.05], [0.1, 0.55, 0.05]), mat.leather),
    arm("L", 0.1),
    arm("R", -0.1),
  ]),
];
```

## Shapes

### Primitives

```ts
function box(min: Vec3, max: Vec3, options?: { round?: number }): Shape;
function boxFrame(min: Vec3, max: Vec3, thickness: number): Shape;
function sphere(center: Vec3, radius: number): Shape;
function ellipsoid(center: Vec3, radii: Vec3): Shape;
function cylinder(a: Vec3, b: Vec3, radius: number, options?: { round?: number }): Shape;
function cone(a: Vec3, b: Vec3, radiusA: number, radiusB: number): Shape;
function capsule(a: Vec3, b: Vec3, radius: number): Shape;
function roundCone(a: Vec3, b: Vec3, radiusA: number, radiusB: number): Shape;
function torus(center: Vec3, ringRadius: number, tubeRadius: number, options?: { axis?: Axis; from?: number; to?: number }): Shape;
function octahedron(center: Vec3, radius: number): Shape;
function pyramid(baseCenter: Vec3, width: number, height: number): Shape;
function halfSpace(side: Side, at: number): Shape;
```

1. `box` spans its two corners. `round` rounds its edges and corners by that
   radius inside the same corners.
2. `boxFrame` keeps the twelve edges of the box between its corners as square
   bars `thickness` across that stay inside the corners. Crates, cages, bed
   frames, and lantern frames start from a `boxFrame`.
3. `cylinder`, `cone`, `capsule`, and `roundCone` run from `a` to `b` at any
   angle. A `cone` takes `radiusA` at `a` and `radiusB` at `b`, and a radius of
   0 makes a point. `round` rounds a cylinder's rims. A `roundCone` caps each
   end with a sphere of that end's radius and tapers between the spheres. A
   round cone suits limbs, branches, horns, and bottle necks.
4. A `torus` rings the y axis unless `axis` picks another. `ringRadius` reaches
   from the center to the middle of the tube, and `tubeRadius` sets the tube's
   radius. `from` and `to` cut the ring to an arc with round ends. The angles
   run as an `arc`'s do in the plane `extrude` maps across `axis`. A cut torus
   makes handles, hooks, and horseshoes.
5. `octahedron` reaches `radius` along each axis from its center and makes a
   cut-gem silhouette at small sizes.
6. `pyramid` stands on a square base `width` across centered on `baseCenter`,
   with its apex `height` above it along +Y.
7. `halfSpace` covers everything past `at` toward `side`:
   `halfSpace("+y", 0.25)` holds every point above `y = 0.25`. A half space has
   no bounds. An `add` takes a half space only inside an `intersect` or as a
   cutter. `carve`, `paint`, and `coat` take one bare.

### 2D shapes

```ts
function circle(center: Vec2, radius: number): Shape2;
function rect(min: Vec2, max: Vec2, options?: { chamfer?: number; round?: number }): Shape2;
function ellipse(center: Vec2, radii: Vec2): Shape2;
function ngon(center: Vec2, sides: number, radius: number): Shape2;
function star(center: Vec2, points: number, outerRadius: number, innerRadius: number): Shape2;
function polygon(points: Vec2[]): Shape2;
function polyline(points: Vec2[], width: number): Shape2;
function arc(center: Vec2, radius: number, fromDegrees: number, toDegrees: number, width: number, options?: { caps?: "flat" | "round" }): Shape2;
function sector(center: Vec2, radius: number, fromDegrees: number, toDegrees: number): Shape2;
function vesica(a: Vec2, b: Vec2, width: number): Shape2;
function arch(min: Vec2, max: Vec2): Shape2;
```

Angles in the plane run from +u toward +v.

1. `rect` takes `round` or `chamfer`. A chamfer cuts each corner at 45 degrees
   with legs that long.
2. `ngon` sets every side `radius` from its center with one side facing -v.
   `ngon(c, 4, r)` makes the square from `c - r` to `c + r`. An octagon's sides
   run along the axes and the diagonals.
3. `star` puts a point at +v.
4. `polygon` closes its outline back to the first point.
5. `polyline` and `arc` stroke a line `width` across for trim, filigree, and
   lettering. An arc's ends come round unless `caps` makes them flat.
6. `sector` cuts a pie slice from a disk.
7. `vesica` makes a pointed lens `width` across from `a` to `b` for leaves,
   blades, and flames.
8. `arch` tops the rectangle between its corners with a half circle as wide as
   the rectangle. Doors, windows, and gravestones start from an `arch`.

### From 2D to 3D

```ts
function extrude(profile: Shape2, options: { axis?: Axis; from: number; to: number }): Shape;
function revolve(profile: Shape2, options?: { axis?: Axis; center?: Vec3 }): Shape;
function lathe(points: Vec2[], options?: { axis?: Axis; center?: Vec3 }): Shape;
```

1. `extrude` pushes the profile from `from` to `to` along the z axis unless
   `axis` picks another. The profile's u and v map to the other two axes in x,
   y, z order: (x, y) for `"z"`, (x, z) for `"y"`, and (y, z) for `"x"`.
2. `revolve` spins the profile around the y axis through `center` unless `axis`
   picks another. The profile's u measures the distance from the axis, and v
   measures the position along it. Only the half at u >= 0 counts.
3. `lathe` revolves the outline its `[radius, height]` points trace and closes
   the outline back along the axis. A `lathe` turns legs, balusters, vases, and
   finials.

### Booleans

```ts
function union<S extends Shape | Shape2>(...shapes: S[]): S;
function intersect<S extends Shape | Shape2>(...shapes: S[]): S;
function subtract<S extends Shape | Shape2>(base: S, ...cutters: S[]): S;
function smoothUnion<S extends Shape | Shape2>(radius: number, ...shapes: S[]): S;
function smoothIntersect<S extends Shape | Shape2>(radius: number, ...shapes: S[]): S;
function smoothSubtract<S extends Shape | Shape2>(radius: number, base: S, ...cutters: S[]): S;
```

The booleans work in 2D and 3D alike. A smooth union fills the inside corners
where its shapes meet with a fillet that reaches `radius` along each surface.
Shapes closer than `radius / 2` fuse. The other smooth variants round their
edges over the same reach.

Shapes carry no material. Geometry meets material only in a step, and one step's
shape can union many shapes under one material.

### Transforms

```ts
interface Shape {
  translate(offset: Vec3): Shape;
  rotate(axis: Axis, degrees: number, pivot?: Vec3): Shape;
  orient(from: Vec3, to: Vec3, pivot?: Vec3): Shape;
  scale(factor: number | Vec3, pivot?: Vec3): Shape;
  mirror(axes: Axes, center?: Vec3): Shape;
  repeat(step: Vec3, count: Vec3): Shape;
  repeatPolar(axis: Axis, count: number, center?: Vec3): Shape;
}
```

1. Transforms apply in call order and in world coordinates. `pivot` and `center`
   default to `[0, 0, 0]`.
2. `orient` turns the shape about `pivot` until the direction `from` points
   along `to`. A `torus` or a `lathe` can then point along any line.
3. `mirror` adds the shape's reflection across the plane through `center` for
   each axis in `axes`. `leg.mirror("xz")` makes four legs from one. A shape
   crossing a plane keeps both halves.
4. `repeat` makes `count` copies along each axis and offsets the copy at
   `[i, j, k]` by `[i * step[0], j * step[1], k * step[2]]`. The copy at
   `[0, 0, 0]` stays put.
5. `repeatPolar` makes `count` copies turned evenly around `axis` through
   `center`.

A `Shape2` takes the same methods in the plane, plus the [modifiers](#modifiers)
`offset` and `shell`:

```ts
interface Shape2 {
  translate(offset: Vec2): Shape2;
  rotate(degrees: number, pivot?: Vec2): Shape2;
  scale(factor: number | Vec2, pivot?: Vec2): Shape2;
  mirror(axes: "u" | "v" | "uv", center?: Vec2): Shape2;
  repeat(step: Vec2, count: Vec2): Shape2;
  repeatPolar(count: number, center?: Vec2): Shape2;
  offset(distance: number): Shape2;
  shell(thickness: number): Shape2;
}
```

### Modifiers

```ts
interface Shape {
  offset(distance: number): Shape;
  shell(thickness: number): Shape;
  elongate(lengths: Vec3, center?: Vec3): Shape;
  twist(axis: Axis, degreesPerMeter: number, center?: Vec3): Shape;
  bend(along: Axis, toward: Side, radius: number, pivot?: Vec3): Shape;
  displace(options: { amplitude: number; scale: number; octaves?: number; seed: number }): Shape;
}
```

1. `offset` grows the shape by `distance` and shrinks it when `distance` is
   negative.
2. `shell` keeps the outer `thickness` and hollows the rest. A shell stays
   closed until a `carve` or a `subtract` cuts an opening.
3. `elongate` cuts the shape through `center` across each axis, moves the halves
   apart by `lengths`, and fills the gap with the cut's cross-section. A `torus`
   stretched along x makes a chain link. A pattern stretches with the shape.
4. `twist` turns each slice across `axis` by `degreesPerMeter` times the slice's
   distance from `center` along the axis.
5. `bend` curls the shape's `along` axis into an arc of `radius` curving toward
   `toward`. The slice through `pivot` stays put, and the arc keeps every length
   along the axis.
6. `displace` roughens the surface with fractal noise up to `amplitude` deep
   whose features run about `scale` across. `octaves` sets how many layers the
   noise sums and defaults to 4.

`offset` and `shell` measure true thickness on:

1. Every primitive but `ellipsoid` and `boxFrame`
2. Extrusions and revolutions of 2D primitives
3. An `intersect` or a `subtract` of such shapes
4. Such shapes moved, rotated, or scaled uniformly

A `union` or a `smoothUnion` measures each cell's depth to the nearest face of
any operand. That face can lie buried inside another operand. A `shell` or a
negative `offset` of a union therefore leaves walls or gaps where its operands
meet. A hollow form takes one shape without buried faces: a `box`, a `lathe`, or
an `extrude` of a `polygon`.

On an `ellipsoid`, a non-uniform scale, or an `elongate` along more than one
axis, the distance reads short. There a `shell` comes out thicker than asked,
and an `offset` moves farther. After `twist`, `bend`, or `displace`, the
distance can read long. A `shell` can then come out thinner and open holes.

### Pattern frames

A pattern reads each cell's position in the frame its step's shape was built in,
before the transform methods moved it. Grain built on an upright leg runs along
the leg after a `rotate` lays it flat. Every copy from `mirror` or `repeat`
shows the same pattern.

A primitive's arguments create no frame: a `cylinder` from `a` to `b` reads
world coordinates. Inside a union, a cell takes the frame of the operand it sits
deepest in, and the earlier operand wins a tie. Inside an `intersect` or a
`subtract`, a cell takes the frame of the first operand. `coat` and `set` read
world coordinates.

## Materials

```ts
const mat: Record<LibraryName, Material>;

function material(properties: Properties): Material;
function int(value: number | number[]): IntValue;
function json(value: unknown): JsonValue;
function shades(base: Material, options?: { count?: number; spread?: number }): Material[];

interface Properties {
  baseColor: string;
  emissiveColor?: string;
  emissiveStrength?: number;
  ior?: number;
  metallic?: number;
  occlusionStrength?: number;
  roughness?: number;
  transmission?: number;
  [name: string]: Value | undefined;
}

type Value = boolean | number | string | number[] | IntValue | JsonValue;
```

1. `mat` holds the [library](#library) by name, such as `mat.oak` or `mat.ruby`.
2. `material` makes a material from the key/value properties a voxj palette
   holds. The eight listed properties follow voxj's
   [glTF conventions](../../../../../projects/voxel-formats/voxj/docs/voxel-json-file-format.md#gltf-conventions)
   for their meanings, ranges, and defaults, except that `metallic` defaults
   to 0. The colors take sRGB hex: `#RRGGBB`, or `#RRGGBBAA` for a `baseColor`
   with alpha.
3. Any other name adds a custom property whose kind follows its value: a
   boolean, a string, a float, or a vector of 2 to 4 floats. `int` marks whole
   numbers and vectors of them, and `json` carries any other JSON value. A
   material without a custom property that another material sets takes the
   kind's empty value there: 0, a zero vector, `false`, `""`, or `null`.
4. `shades` returns `count` versions of `base` that run from darkest to lightest
   with the original in the middle. Neighboring shades differ in perceived
   lightness by `spread`, and the other properties carry over. `count` defaults
   to 3 and `spread` to 0.08. A shade past black or white errors.

```ts
const glass = material({ baseColor: "#D8F0FF", roughness: 0.05, transmission: 0.9 });
const rune = material({ baseColor: "#3A3F44", emissiveColor: "#40C0FF", emissiveStrength: 4 });
const chest = material({ baseColor: "#8A5A2B", lootTier: int(3), locked: true });
```

### Library

| Group  | Names                                                                   |
| ------ | ----------------------------------------------------------------------- |
| Wood   | `bark`, `birch`, `ebony`, `mahogany`, `oak`, `pine`, `walnut`           |
| Stone  | `brick`, `clay`, `granite`, `marble`, `sandstone`, `slate`, `stone`     |
| Metal  | `brass`, `bronze`, `copper`, `gold`, `iron`, `silver`, `steel`          |
| Gem    | `amethyst`, `diamond`, `emerald`, `ruby`, `sapphire`, `topaz`           |
| Nature | `bone`, `dirt`, `grass`, `ice`, `leaf`, `moss`, `sand`, `snow`, `water` |
| Craft  | `leather`, `rope`, `straw`                                              |
| Light  | `ember`, `flame`, `glow`, `lava`                                        |

The metals are fully metallic, the gems are smooth and saturated, and the light
group glows. The gems, `ice`, and `water` also transmit light at their own
`ior`. Velvet, painted wood, and anything else the library lacks take a custom
`material`. vxl holds the library's values. A `.vxlconfig` can add a name or
replace one at `sdfDoc.library`.

### Patterns

```ts
function bands(materials: Material[], options: { axis: Axis; period?: number; warp?: number; seed?: number }): Pattern;
function grain(base: Material | Material[], options: { axis: Axis; period?: number; warp?: number; seed: number }): Pattern;
function gradient(materials: Material[], options: { axis: Axis; from: number; to: number; warp?: number; seed?: number }): Pattern;
function noise(materials: Material[], options: { scale: number; octaves?: number; seed: number }): Pattern;
function cells(materials: Material[], options: { size: number; seed: number; border?: Material }): Pattern;
function speckle(base: Material, accents: Material | Material[], options: { density: number; seed: number }): Pattern;
function checker(materials: Material[], options?: { size?: number }): Pattern;
```

A pattern picks one of its materials per cell and never blends them. The palette
then stays as small as the material lists. A seed always gives the same picks. A
length a pattern leaves out counts cells at any voxel size.

1. `bands` slices space across `axis` into slabs `period` thick and cycles
   through `materials` in order. `warp` bends the slabs by up to that much
   noise. `period` defaults to 1 cell and `warp` to 0.
2. `grain` slices the same way and gives each slab a random pick. Its `period`
   defaults to 2 cells and its `warp` to 1.5 cells. A single `base` stands for
   `shades(base)`. The `axis` runs along the wood's length.
3. `gradient` splits `from` to `to` along `axis` into equal spans that take the
   materials in order. Cells before `from` take the first material and cells
   past `to` the last.
4. `noise` splits fractal noise with features about `scale` across into value
   ranges that take the materials in order. Each material covers about an equal
   share of the cells.
5. `cells` breaks space into irregular cells about `size` across that each take
   a random pick. `border` fills a seam one grid cell wide between the cells for
   cobblestone and mortar.
6. `speckle` gives each grid cell a `density` chance of a random accent and
   leaves the rest `base`. Because `speckle` reads grid cells, its copies
   differ.
7. `checker` alternates its materials over cubes `size` wide. `size` defaults to
   1 cell.

## Report

`--report` prints a line for the model, then a line per step in list order. The
[example](#example) chair reports:

```
chair.voxj  1904 voxels of 0.025 m  1 piece  20x41x20  [-0.25, 0, -0.25] .. [0.25, 1.025, 0.25]
  add    legs      400 cells  400 kept  336 exposed  20x17x20  [-0.25, 0, -0.25] .. [0.25, 0.425, 0.25]
  add    seat      800 cells  800 kept  772 exposed  20x2x20   [-0.25, 0.425, -0.25] .. [0.25, 0.475, 0.25]
  add    posts     234 cells  234 kept  208 exposed  20x13x3   [-0.25, 0.475, -0.25] .. [0.25, 0.8, -0.175]
  add    spindles   52 cells   52 kept   52 exposed  10x13x1   [-0.125, 0.475, -0.225] .. [0.125, 0.8, -0.2]
  add    rail      360 cells  270 kept  196 exposed  20x6x3    [-0.25, 0.8, -0.25] .. [0.25, 0.95, -0.175]
  paint  gilt       60 cells   58 kept   56 exposed  20x1x3    [-0.25, 0.925, -0.25] .. [0.25, 0.95, -0.175]
  add    finials    30 cells   30 kept   24 exposed  20x4x3    [-0.25, 0.925, -0.25] .. [0.25, 1.025, -0.175]
  add    jewels     60 cells   60 kept   24 exposed  14x3x4    [-0.175, 0.825, -0.225] .. [0.175, 0.9, -0.125]
```

1. A step's cells are the cells it wrote as it ran: every cell an `add` or a
   `set` filled, every live cell a `carve` emptied, and every live cell a
   `paint` or a `coat` recolored.
2. The step keeps the cells no later step changed. `exposed` counts the kept
   cells touching empty space across a face. A `carve` line has no `exposed`
   count.
3. A size counts cells along x, y, and z. Bounds run in meters from the min
   cell's min corner to the max cell's max corner and can feed a `box` directly.
   A step's size and bounds cover the cells it wrote.
4. A step with `0 cells` wrote nothing: an `add` or a `set` fell between the
   cell centers, or a `carve`, `paint`, or `coat` met no live cell. A step with
   cells and `0 kept` was undone by later steps.

The rail keeps 270 of its 360 cells because the gilt and the jewels take the
rest. The jewels keep all 60 cells and show 24 of them because half of each
jewel sits inside the rail. The spindles come out one cell thick.

With parts, each part gets a line ahead of its list. The list's lines indent one
level past the part's line. A part line counts the cells the part owns. A part
reads `detached` when none of its cells shares a face with its parent's. A part
reading `0 voxels` writes its node with no object. The report for a model in
more than one face-connected piece ends with a line per piece. The piece lines
run from the largest piece down and list the steps behind each piece's cells.

With its hands at `x * 3.25`, the [figure](#parts) leaves a one-voxel gap at
each wrist:

```
figure.voxj  392 voxels of 0.025 m  3 pieces  28x10x4  [-0.35, 0.3, -0.05] .. [0.35, 0.55, 0.05]
  part  body  312 voxels  8x10x4  [-0.1, 0.3, -0.05] .. [0.1, 0.55, 0.05]
    add  torso  320 cells  312 kept  216 exposed  8x10x4  [-0.1, 0.3, -0.05] .. [0.1, 0.55, 0.05]
    part  arm.L  32 voxels  8x2x2  [0.075, 0.5, -0.025] .. [0.275, 0.55, 0.025]
      add  L arm  32 cells  32 kept  30 exposed  8x2x2  [0.075, 0.5, -0.025] .. [0.275, 0.55, 0.025]
      part  hand.L  8 voxels  2x2x2  [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]  detached
        add  L hand  8 cells  8 kept  8 exposed  2x2x2  [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]
    part  arm.R  32 voxels  8x2x2  [-0.275, 0.5, -0.025] .. [-0.075, 0.55, 0.025]
      add  R arm  32 cells  32 kept  30 exposed  8x2x2  [-0.275, 0.5, -0.025] .. [-0.075, 0.55, 0.025]
      part  hand.R  8 voxels  2x2x2  [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]  detached
        add  R hand  8 cells  8 kept  8 exposed  2x2x2  [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]
  piece 1  376 voxels  22x10x4  [-0.275, 0.3, -0.05] .. [0.275, 0.55, 0.05]  from torso, L arm, R arm
  piece 2    8 voxels  2x2x2    [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]   from R hand
  piece 3    8 voxels  2x2x2    [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]     from L hand
```

A second piece usually means a shape floats.

## Errors

Each command stops at its first error. The message reports the failing step and
value. A step that writes no cell stops nothing and reads `0 cells` in the
report. The [checks](model-evaluation.md#checks) list every error.

## Resolution

Most voxel models sit 16 to 64 voxels across. At that size the grid shapes
everything:

1. A shape thinner than a voxel can miss every cell center and vanish. A thin
   feature takes whole cells: a one-voxel post is the `box` from `[x, y0, z]` to
   `[x + v, y1, z + v]`, with `x` and `z` on multiples of `v`.
2. A sphere or cylinder under a radius of about 3 voxels reads as a plus sign or
   a block. Small round details read better as boxes or single `set` cells.
3. A large `add` placed after a detail buries it because later steps win. Large
   forms go first and details after.
4. Copies from `mirror` and `repeat` repeat their pattern exactly. A distinct
   look per copy takes its own step with its own seed.
5. A model holds its details at the voxel size they were sized for. A coarser
   sampling can drop details thinner than its cells, and a finer one draws each
   detail with more cells without adding any.

## Recipes

Each recipe shows shapes as constants and steps as entries of the list, sized
for `--voxel-size 0.025`.

```ts
// Four turned legs from one profile.
const leg = lathe([[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]]).translate([0.2, 0, 0.2]);
add("legs", leg.mirror("xz"), grain(mat.oak, { axis: "y", seed: 1 })),

// A jewel set into a surface. The later add takes the cells the two share.
add("jewel", octahedron([0, 0.875, -0.175], 0.0625), mat.ruby),

// A gilded top edge.
paint("gilt", intersect(rail.shell(0.025), halfSpace("+y", 0.925)), mat.gold),

// An open bowl.
add("bowl", subtract(sphere([0, 0.15, 0], 0.15).shell(0.025), halfSpace("+y", 0.2)), mat.clay),

// Twelve studs around a rim.
add("studs", box([0.225, 0.075, -0.025], [0.275, 0.125, 0.025]).repeatPolar("y", 12), mat.iron),

// Floor planks in alternating shades.
add("floor", box([-0.4, 0, -0.4], [0.4, 0.025, 0.4]), bands(shades(mat.pine), { axis: "x", period: 0.1 })),

// A cobbled wall with an arched doorway through it.
add("wall", box([0, 0, 0], [0.8, 0.4, 0.075]), cells(shades(mat.stone), { size: 0.15, seed: 7, border: mat.dirt })),
carve("doorway", extrude(arch([0.325, 0], [0.475, 0.25]), { from: 0, to: 0.075 })),

// A tapered branch.
add("branch", roundCone([0, 0.25, 0], [0.15, 0.45, 0.05], 0.05, 0.025), mat.bark),

// A lantern cage.
add("cage", boxFrame([-0.1, 0, -0.1], [0.1, 0.25, 0.1], 0.025), mat.iron),

// Snow on every upward face.
coat("snow", mat.snow, { sides: ["+y"] }),

// A rough boulder.
add("boulder", sphere([0, 0.125, 0], 0.15).displace({ amplitude: 0.0375, scale: 0.1, seed: 3 }), mat.granite),
```

## Example

```ts
// An oak chair facing +z with three rubies set in its top rail, sized for
// --voxel-size 0.025.
const oak = (axis: Axis, seed: number) => grain(mat.oak, { axis, seed });

const leg = lathe([[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]]).translate([0.2, 0, 0.2]);
const post = box([0.175, 0.475, -0.25], [0.25, 0.8, -0.175]);
const spindle = box([-0.125, 0.475, -0.225], [-0.1, 0.8, -0.2]).repeat([0.075, 0, 0], [4, 1, 1]);
const rail = box([-0.25, 0.8, -0.25], [0.25, 0.95, -0.175], { round: 0.025 });
const finial = sphere([0.2125, 0.975, -0.2125], 0.0375).mirror("x");
const jewels = union(...[-0.125, 0, 0.125].map((x) => octahedron([x, 0.875, -0.175], 0.0625)));

export default [
  add("legs", leg.mirror("xz"), oak("y", 1)),
  add("seat", box([-0.25, 0.425, -0.25], [0.25, 0.475, 0.25]), oak("x", 2)),
  add("posts", post.mirror("x"), oak("y", 3)),
  add("spindles", spindle, oak("y", 4)),
  add("rail", rail, oak("x", 5)),
  paint("gilt", intersect(rail.shell(0.025), halfSpace("+y", 0.925)), mat.gold),
  add("finials", finial, mat.gold),
  add("jewels", jewels, mat.ruby),
];
```
