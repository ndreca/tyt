# voxrender

The render contract for voxel scenes and its CPU reference renderer. The
contract types describe an image of a scene. The CPU renderer, behind the
default `cpu` feature, follows them literally and makes the images every
other renderer is tested against. The planned `voxrender-wgpu` draws the
same scene in realtime.

## Entities

A `U32Id` over one brand addresses each kind of entity:

1. Objects: `BVoxObject`, the id of the voxcore object each mirrors
2. Voxels: `BVoxVoxel`, voxcore's id for the cell's position
3. Materials: `BRenderMaterial`
4. Placements: `BRenderPlacement`
5. Lights: `BRenderLight`
6. Views: `BRenderView`

## The scene

A `RenderScene` is a document flattened for drawing. Objects are dense
grids whose cells each hold one material of the scene's table or nothing.
Placements put an object into world meters under a `TyTransformF64`, one
per root-to-object path of the hierarchy. Lights and views sit in world
space. Every mutation checks the cross-references it could break, so a
placement never points at a released object and a voxel never samples a
released material. `from_vox_main`
flattens a voxcore document: one material per live voxel by the effective
palette, deduplicated into the table, and one placement per path with the
grid origin and the voxel size folded in. `subject_bounds` gives the world
bounds of the live voxels under a set of placements, which the `fit` rule
frames.

```rust
let scene = RenderScene::from_vox_main(&main, &[object_id], 0.1)?;

let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();
let bounds = scene.subject_bounds(&placement_ids)?;
```

## Materials

A `RenderMaterial` carries the shaded properties in linear light: the base
color, metalness, roughness, the emissive color and its strength, and the
occlusion strength. Every voxel is opaque. The default is glTF's: opaque
white, fully metallic and rough, with no emission.

```rust
let brass = RenderMaterial {
    base_color: TyLinSrgbF64::new(0.8, 0.6, 0.2),
    roughness: 0.4,
    ..RenderMaterial::default()
};
```

## Lights

A `RenderLight` is one of three kinds, each carrying the part of a pose it
reads. A directional light is a rotation and shines down its -Z. A point
light is a position in meters with an inverse-square falloff and an
optional range. A hemisphere light is a sky color above and a ground color
below, mixed by a normal's world +Y. The two shadowing kinds carry a
`RenderShadow`: none, one ray per pixel, one per face, or one per face
corner.

```rust
let sun = RenderLight::Directional {
    rotation: TyQuaternionF64::IDENTITY,
    color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
    strength: 3.0,
    shadow: RenderShadow::PerCorner,
};
```

## Views

A `RenderView` is a `TyPoseF64` looking down its -Z with +Y up, and a
`RenderProjection`: perspective with a vertical field of view in radians,
or orthographic with the world units across the shorter image axis.

```rust
let view = RenderView {
    pose: TyPoseF64::new(TyVector3F64::new(0.0, 0.0, 10.0), TyQuaternionF64::IDENTITY),
    projection: RenderProjection::Perspective { fov: 35f64.to_radians() },
};
```

## Images

A `RenderImage` is a linear-light RGBA `f32` buffer, rows top to bottom.
`RenderOcclusion` picks whether a render shades with the corner occlusion
`voxsurface` computes.

```rust
let mut image = RenderImage::new(1024, 1024);

image.set_pixel(0, 0, TyLinSrgbaF32::new(1.0, 1.0, 1.0, 1.0));
```
