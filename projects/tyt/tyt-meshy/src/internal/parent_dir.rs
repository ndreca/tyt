use std::path::{Path, PathBuf};

/// Returns the directory containing `path`, treating an empty parent as the
/// current directory.
pub fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}
