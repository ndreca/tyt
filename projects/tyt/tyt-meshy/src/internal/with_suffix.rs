use std::path::{Path, PathBuf};

/// Appends `suffix` to `base`'s path (e.g. `out/foo` + `.usdz` → `out/foo.usdz`).
pub fn with_suffix(base: &Path, suffix: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}
