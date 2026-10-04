use crate::commands::SdfDocBuildProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `sdfDoc.build` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SdfDocBuildConfig {
    pub(crate) profiles: BTreeMap<String, SdfDocBuildProfile>,
}
