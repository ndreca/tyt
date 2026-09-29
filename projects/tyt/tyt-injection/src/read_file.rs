use std::{fs, io::Result, path::Path};

/// Reads a whole file into bytes.
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path)
}
