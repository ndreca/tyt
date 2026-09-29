use crate::NamedCliValue;
use serde::Deserialize;
use ty_math::TyAngleUnit;

/// A profile's rotation. Its `kind` picks the rotation flag it mirrors.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RotationEntry {
    /// Mirrors `--view-quaternion` and `--light-quaternion`.
    Quaternion { value: [f64; 4] },

    /// Mirrors `--view-euler` and `--light-euler`. `unit` defaults to degrees.
    Euler {
        value: [f64; 3],

        unit: Option<NamedCliValue<TyAngleUnit>>,
    },

    /// Mirrors `--view-look-at` and `--light-look-at`. `target` defaults to
    /// the frame's origin.
    LookAt { target: Option<[f64; 3]> },

    /// Mirrors `--view-angles` and `--light-angles`, in degrees.
    Angles { azimuth: f64, elevation: f64 },
}
