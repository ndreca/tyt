use std::{
    fs,
    io::{ErrorKind, Result},
    path::Path,
};

/// Removes a file. A missing file counts as success.
pub fn remove_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
