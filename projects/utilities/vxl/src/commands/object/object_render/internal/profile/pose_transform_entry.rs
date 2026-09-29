use crate::commands::{DistanceEntry, RotationEntry};
use serde::Deserialize;

/// A profile's view transform. Its `kind` takes a `--view-frame` value or
/// `orbit`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PoseTransformEntry {
    World {
        position: [f64; 3],

        rotation: RotationEntry,
    },

    Subject {
        position: [f64; 3],

        rotation: RotationEntry,
    },

    /// Angles in degrees. `distance` defaults to `fit`.
    Orbit {
        azimuth: f64,

        elevation: f64,

        distance: Option<DistanceEntry>,
    },
}
