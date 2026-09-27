use std::path::{Path, PathBuf};

/// The output one object of a split run writes: `output`'s stem, a hyphen,
/// and `stem`, under `output`'s extension beside it.
pub(crate) fn split_output(output: &Path, stem: &str) -> PathBuf {
    let output_stem = output
        .file_stem()
        .expect("the output path carries a file name")
        .to_string_lossy();

    let mut file_name = format!("{output_stem}-{stem}");

    if let Some(extension) = output.extension() {
        file_name = format!("{file_name}.{}", extension.to_string_lossy());
    }

    output.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use crate::commands::split_output;
    use std::path::{Path, PathBuf};

    #[test]
    fn the_object_joins_the_stem_under_the_extension() {
        assert_eq!(
            split_output(Path::new("out/scene.glb"), "turret"),
            PathBuf::from("out/scene-turret.glb")
        );
        assert_eq!(
            split_output(Path::new("scene"), "turret"),
            PathBuf::from("scene-turret")
        );
    }
}
