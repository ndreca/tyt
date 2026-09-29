use crate::PositiveF64;
use serde::Deserialize;

/// A profile's point light transform. Its `kind` takes a `--light-frame`
/// value or `orbit`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PositionTransformEntry {
    World {
        position: [f64; 3],
    },

    Subject {
        position: [f64; 3],
    },

    Camera {
        position: [f64; 3],
    },

    /// Angles in degrees and a distance in meters.
    Orbit {
        azimuth: f64,

        elevation: f64,

        distance: PositiveF64,
    },
}
