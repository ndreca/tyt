use crate::{Script, embed_blender_script};

/// The helper module every Blender script imports.
pub const COMMON_PY: Script = embed_blender_script!("common.py");
