# Voxel glass plan

Status: **open.** Opened 2026-10-04 after the glass review. The steps live
in [checklist.md](checklist.md). Code-level choices are logged in
[implementation-decisions.md](implementation-decisions.md). The
[contract](../../../ref/render/contract.md) holds the render rules. The
[follow-ups plan](../voxel-rendering-followups/README.md) points here.

## Goal

Make voxel glass read as glass, and keep its tint in a transparent PNG. The
review of the integer walk found two faults:

1. A slab shows only the face a ray enters, so a block of glass reads as a
   flat tinted shape with no far side
2. The transparent output takes one alpha from the most transmitted
   channel. Colored glass with nothing behind it vanishes on a light page,
   and the red pillar in `glass.voxj` loses its top

The review compared prototype renders of each candidate over white, gray,
dark, black, and patterned backdrops, and this plan builds the ones it
picked. It revisits two choices of the closed
[transparency plan](../../closed/voxel-transparency/README.md): front faces
only, its decision 1, and the output's single alpha.

## Exits

Today a ray hits a surface only where it enters a material. Glass gains its
far side as an exit:

1. Leaving a cell for an empty cell, for the outside of the grid, or for a
   live cell of another material that is not opaque is an exit. The exit is
   a hit through the face the ray leaves by, of the material it leaves
2. Leaving for an opaque cell is no exit. The opaque cell's entry is the
   next hit, at the same distance
3. An exit comes before the entry at the same face
4. A ray that starts inside a cell leaves that cell's material without an
   exit, as it leaves it without a hit today. A view from inside a voxel
   still sees out of it, and a shadow ray that starts inside glass still
   starts with its pass

A pane shows its near and far walls. Touching voxels of one material still
have no seam.

## Shading an exit

An exit shades as an entry does, with its normal facing back along the ray.
Lights reach it through the glass, so its shadow rays start inside the
material and take its pass. Its corner occlusion counts the opaque cells on
the glass side. The pixel adds it at its coverage and multiplies the
throughput by its pass, so every wall tints and a slab tints twice. Shadow
rays meet exits too, so a slab shadows by both walls. A half-alpha voxel
covers half at each wall, three quarters in all.

## Fresnel by angle

Today the transmitted share loses the reflectance at normal incidence, F0,
at every angle. The hemisphere light reflects F0 of the ambient, mixed by
the normal. Real glass reflects more and passes less toward grazing angles,
which gives its faces their edges. The transmitted share takes Schlick's
Fresnel on the cosine between the ray and the face normal, with the rise
capped by the roughness:

```
F = F0 + (max(1 - roughness, F0) - F0) * (1 - cos)^5
pass = (1 - alpha) + alpha * transmission * (1 - metallic) * (1 - F) * baseColor
```

What the pass loses, the transmitted share reflects. The hemisphere's
reflection off that share mixes sky and ground by the reflected direction's
+Y and scales by F. The rest of the surface keeps today's hemisphere term.
An opaque material has no transmitted share and shades as today.

The pixel walk takes the view ray's cosine and a shadow walk takes its own.
A ray that starts inside a material takes that material's pass at normal
incidence, since the face it leaves by is not yet known. Opacity stays the
pass at normal incidence being zero, for the occlusion and the mesher alike.

## Output

The output keeps today's coverage and light layer and changes how the
transparent PNG folds the transmittance into one alpha:

1. The coverage is today's alpha: the larger of one minus the peak
   transmittance and the peak of the light, clamped to one. The layer is the
   light over the coverage, tonemapped, times the coverage
2. Under a background color the pixel is the layer plus the background
   scaled by the transmittance, as today
3. Under `transparent` the PNG is solved to be exact over white for a viewer
   that blends sRGB values, as browsers and image editors do

Per channel, `W` is the sRGB encoding of the layer plus the transmittance,
clamped to one: the pixel over white. The PNG's transparency `t` is the
smaller of the peak transmittance and the least channel of `W`:

```
alpha = 1 - t
color = (W - t) / alpha, in sRGB values, or 0 at a zero alpha
```

The rule matches today's wherever the transmittance is zero: an opaque
pixel has a `t` of zero and the layer's color at full alpha. A miss has a
`t` of one. Over white in a browser every pixel matches `--background
white`. Colorless transmittance, as in clear glass and bloom halos, moves by
a level or two. Colored glass keeps its tint. Over a dark backdrop it reads
brighter than it should, because one alpha cannot be exact over two
backdrops.

## The mesh

The `glass` profile writes `doubleSided` on its blended material, so a glTF
viewer draws a glass block's far walls as the reference does. Where two
glass materials touch, the mesh keeps both faces back to back, and both
draw.

## Decisions

1. The far wall is an exit hit of the material the ray leaves. Its shade,
   coverage, and pass are the material's, so one shading path serves both
   walls.
2. Every wall tints. One rule for every surface a ray crosses keeps the
   pixel walk, the shadow walk, and a GPU shader free of a special case.
   Existing glass gets deeper, since a pane passes its pass squared. An
   author lightens a base color for the old look.
3. No exit into an opaque cell. The mesh drops a glass face against an
   opaque neighbor, and the rule keeps two surfaces off one plane.
4. Fresnel by angle applies to the transmitted share only. Opaque materials,
   and every golden without glass, hold.
5. The PNG is exact over white. Previews are mostly read on light pages, and
   colored glass that reads too bright over a dark page still reads as
   colored glass, where today's rule loses the geometry. The solve is for
   sRGB blending because browsers composite that way. A solve in linear
   light leaves colored glass 55 to 65 levels off over white in a browser.
6. `--background` is unchanged. It composites per channel and stays exact.
7. No tint by thickness. Without refraction, tint by path length fades a
   block's edges to fog.

## Out of scope

1. Refraction and total internal reflection
2. Volume absorption
3. Fresnel by angle on opaque materials
4. Rough transmission
