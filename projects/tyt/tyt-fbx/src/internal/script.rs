/// Script content to be executed by Blender.
pub struct Script<'a> {
    /// The script's path within the temp directory.
    pub relative_file_path: &'a str,

    /// The script's source text.
    pub content: &'a str,
}
