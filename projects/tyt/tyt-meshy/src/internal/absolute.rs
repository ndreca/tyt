use std::path::{Path, PathBuf};

/// Resolves `path` against `base` when it is relative, leaving absolute paths
/// unchanged.
pub fn absolute(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}
