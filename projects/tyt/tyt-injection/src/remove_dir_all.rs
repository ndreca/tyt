use std::{
    fs,
    io::{ErrorKind, Result},
    path::Path,
};

/// Removes a directory and its contents. A missing directory counts as success.
pub fn remove_dir_all(path: &Path) -> Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
