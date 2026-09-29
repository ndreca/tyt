use crate::commands::RotationEntry;
use serde::Deserialize;

/// A profile's directional light transform. Its `kind` takes a
/// `--light-frame` value.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RotationTransformEntry {
    World { rotation: RotationEntry },

    Camera { rotation: RotationEntry },
}
