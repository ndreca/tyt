use std::{fs, io::Result, path::Path};

/// Creates or overwrites a file with `contents`.
pub fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    fs::write(path, contents)
}
