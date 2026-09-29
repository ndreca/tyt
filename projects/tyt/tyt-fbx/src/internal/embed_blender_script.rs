/// Builds a [`Script`](crate::Script) whose content embeds the file at
/// `rel_path` under `src/blender/`.
macro_rules! embed_blender_script {
    ($rel_path:literal) => {
        $crate::Script {
            relative_file_path: $rel_path,
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/blender/",
                $rel_path,
            )),
        }
    };
}

pub(crate) use embed_blender_script;
