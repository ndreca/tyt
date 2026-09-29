use std::path::Path;
use tyt_common::relativize;

/// Expresses `path` relative to `base` as a string.
pub fn relative(base: &Path, path: &Path) -> String {
    relativize(base, path).to_string_lossy().into_owned()
}
