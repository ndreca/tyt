use crate::{Dimension, Domain, ValueEnvironment, bools, f32s, groupings, strings, u8s, u32s};
use std::collections::HashMap;

/// The lamp: two swatches, a steel base under a glowing glass bulb, two
/// voxels, and ten faces of one voxel each.
pub fn lamp() -> ValueEnvironment {
    let mut environment = ValueEnvironment {
        values: HashMap::new(),
        groupings: groupings(
            &[0, 1],
            &[&[0], &[0], &[0], &[0], &[0], &[1], &[1], &[1], &[1], &[1]],
        ),
    };

    environment.values = [
        (
            "baseColor",
            f32s(
                Domain::Swatch,
                Dimension::Vec4,
                &[0.5, 0.5, 0.5, 1.0, 1.0, 0.9, 0.6, 0.6],
            ),
        ),
        (
            "roughness",
            f32s(Domain::Swatch, Dimension::Vec1, &[0.9, 0.4]),
        ),
        (
            "metallic",
            f32s(Domain::Swatch, Dimension::Vec1, &[1.0, 0.0]),
        ),
        (
            "emissiveColor",
            f32s(
                Domain::Swatch,
                Dimension::Vec3,
                &[0.0, 0.0, 0.0, 1.0, 0.9, 0.6],
            ),
        ),
        (
            "emissiveStrength",
            f32s(Domain::Swatch, Dimension::Vec1, &[0.0, 4.0]),
        ),
        ("tag", strings(Domain::Swatch, &["steel", "glass"])),
        ("flag", bools(Domain::Swatch, &[false, true])),
        ("count", u32s(Domain::Swatch, Dimension::Vec1, &[3, 7])),
        (
            "wide",
            u8s(Domain::Voxel, Dimension::Vec2, &[200, 100, 50, 25]),
        ),
        (
            "voxelPosition",
            u32s(Domain::Voxel, Dimension::Vec3, &[0, 0, 0, 0, 1, 0]),
        ),
        (
            "faceValue",
            f32s(
                Domain::Face,
                Dimension::Vec1,
                &(0..10).map(|face| face as f32).collect::<Vec<_>>(),
            ),
        ),
        (
            "computedOcclusion",
            f32s(
                Domain::Corner,
                Dimension::Vec1,
                &(0..40)
                    .map(|corner| corner as f32 / 40.0)
                    .collect::<Vec<_>>(),
            ),
        ),
        (
            "unit",
            f32s(Domain::Plain, Dimension::Vec3, &[1.0, 0.0, 0.0]),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value))
    .collect();

    environment
}
