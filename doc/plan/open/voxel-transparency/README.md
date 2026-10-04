# Voxel transparency plan

Status: **open.** The design was settled 2026-10-02. The steps live in
[checklist.md](checklist.md), checked off as they land. Code-level choices are logged
in [implementation-decisions.md](implementation-decisions.md). The
[contract](../../../ref/render/contract.md) gains the rules below as the
steps land. The [follow-ups plan](../voxel-rendering-followups/README.md)
points here.

## Goal

Give `baseColor` alpha, `transmission`, and `ior` an effect in the render
contract and the CPU reference. The vocabulary already carries all three:
voxj stores the alpha as opacity, the vmax bridge fills `transmission` and
`ior` from a material's dispersion block, and voxelize fills them from glTF.
The render drops the alpha and never reads the other two.

Touching voxels of one glass read as one pane. A thick wall of glass shows
one face and throws the shadow a thin one throws.

## The surface

The contract's surface today is the boundary between live and non-live
cells. With transparency it is the boundary between materials, read along a
ray:

1. In each placement, the ray remembers the material of the cell it is
   inside. Before it enters the grid, and after it enters an empty cell, it
   is inside nothing
2. Entering a cell whose material differs from the one the ray is inside is
   a surface hit. The hit's face is the face the ray entered through
3. Entering a cell of the same material is internal and shades nothing
4. Entering an empty cell shades nothing. There are no back faces
5. A ray that starts inside a cell is inside its material, so it leaves that
   material before it can hit anything

A pane two voxels thick shows one face, two glass colors touching show
their boundary, and an opaque wall behind a window shows its lit face
through it. Placements walk independently and their hits merge by distance.

The ray never bends. Transmission is glTF's thin-surface model without the
volume extension: light passes straight through, tinted once per surface.

## Materials

A `RenderMaterial` gains the three values:

1. The base color becomes `TyLinSrgbaF64`. Its alpha is coverage, `0..1`
2. `transmission` is the share of the diffuse term that passes through,
   `0..1`, `0` by default
3. `ior` is `0` or `1..`, `1.5` by default

The dielectric reflectance at normal incidence comes from `ior` as
`((ior - 1) / (ior + 1))^2`. At the default `1.5` it is the `0.04` the
reference uses today, so every existing image holds. At `0` it is `1`, the
full Fresnel glTF's `KHR_materials_ior` specifies for that value.

## Shading

A hit shades as today, with the diffuse term scaled by `1 - transmission`
in both the direct and the hemisphere light. The specular term, the
emission, and the occlusion are unchanged.

What continues past the hit is one factor per channel, the material's pass:

```
pass = (1 - alpha) + alpha * transmission * (1 - metallic) * (1 - F0) * baseColor
```

The first term is the part of the pixel the surface does not cover. The
second is the light the covered part transmits: the dielectric share, less
what reflects at normal incidence, tinted by the base color. An opaque
material's pass is zero.

A pixel walks its ray front to back. It starts with a throughput of one. At
each hit it adds `throughput * alpha * shade` to its light and
`throughput * alpha * emission` to the bloom buffer, then multiplies the
throughput by the pass. The walk ends at a zero throughput or when the ray
leaves every grid. What remains is the pixel's transmittance.

## Shadows

A shadow ray walks the same surface and multiplies the passes of the hits
it meets until the throughput is zero or it reaches the light. The result
is a color, so a red pane throws a red shadow. `per-corner` blends the four
corners per channel. A pane two voxels thick shadows like a thin one because
its seam is internal.

## Occlusion and bloom

The corner occlusion keeps reading every live cell as solid, the rule
`voxsurface` has and `object mesh` bakes. A transparent
voxel's emission adds at its coverage and blooms as any emission does.

## Output

A pixel carries two colors: the light that reached it, in linear radiance,
and its transmittance, the fraction of what lies behind the scene that
passes through per channel. A miss carries no light and full transmittance.
An opaque hit carries its shade and none. The bloom halo adds to the light
and scales the transmittance by one minus its peak channel, which is the
coverage the halo has today.

The output derives one alpha from the two: the larger of one minus the peak
transmittance and the peak of the light, clamped to one. The color is the
light over that alpha. The peak of the light keeps a highlight on clear
glass from vanishing into a tiny alpha. Under `transparent` the PNG stores
that pair. Under a background color the pixel is the tonemapped color at
that alpha plus the background scaled by the transmittance, so red glass
over white is red and clear glass over white stays white.

The rules reduce to today's output everywhere transparency is absent. An
opaque hit has an alpha of one. A miss is the background or transparent. A
halo on a miss has the halo's peak as its alpha and composites over the
background as it does today. The background never passes through the
tonemap, which would turn a white background grey behind clear glass.

## Decisions

1. Front faces only. The slab rule shades a material boundary once, the
   look a voxel pane should have. Back faces would stack two layers on a
   block of glass.
2. No refraction. The thin-surface model needs no bent ray and no exit
   event. A volume model with absorption and bending is a later plan if a
   reason appears.
3. `ior` sets the dielectric reflectance and nothing else. The formula
   gives the reference's current `0.04` at the default, so no golden moves.
4. Alpha is coverage and transmission is tinted transmission, glTF's split.
   A half-alpha voxel shows half of what is behind it untinted. A
   transmissive voxel keeps its highlights and tints what is behind it.
5. One pass factor serves the pixel walk and the shadow walk, so a surface
   shadows exactly as it transmits.
6. Shadows are colors. A scalar shadow would grey a stained-glass window.
7. Transparent cells occlude like solid ones, matching the mesh bake.
8. The image carries light and transmittance, not a straight alpha. A
   single alpha cannot carry a tint, and compositing the background before
   the tonemap would recolor it. The straight alpha is an output rule.
9. The ray walk is a `voxrender` iterator over surface hits, one DDA per
   placement merged by distance. `cast_ray` stays as its first hit.
10. No new flag or profile key. Transparency comes from the palette.

## Out of scope

1. Volume absorption. The vmax bridge's custom `absorption` stays unread.
2. Rough transmission. A rough glass transmits as sharply as a smooth one.
3. MagicaVoxel glass. The mvox bridge keeps a glass material's transparency
   in its unmodeled extra keys, so `.vox` glass stays opaque until that
   bridge maps it. It also binds the raw `_ior` under the vocabulary's `ior`
   name. If MagicaVoxel stores that value offset from the index, reading
   `ior` rejects such files on range. The fix is the bridge's. No `.vox`
   fixture in the repo confirms either way.
4. The mesher. `voxsurface` culls a face against any solid neighbor, so an
   opaque wall behind a glass pane has no face in a mesh. `object mesh` and
   the standalone tier need a transparency-aware cull before they can show
   glass. The follow-ups plan notes it.
