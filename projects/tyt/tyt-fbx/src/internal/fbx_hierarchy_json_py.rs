use crate::{Script, embed_blender_script};

/// The Blender script that prints an FBX file's object hierarchy as JSON.
pub const FBX_HIERARCHY_JSON_PY: Script = embed_blender_script!("fbx_hierarchy_json.py");
