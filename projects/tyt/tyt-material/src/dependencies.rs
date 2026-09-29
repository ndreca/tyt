use crate::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// The side effects material commands perform.
pub trait Dependencies {
    /// Copies the file at `from` to `to`.
    fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()>;

    /// Creates a fresh temporary directory and returns its path.
    fn create_temp_dir(&self) -> Result<PathBuf>;

    /// Runs `magick` with `args` and returns its standard output.
    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Returns the path matching `pattern` when exactly one matches.
    fn glob_single_match(&self, pattern: &str) -> Result<PathBuf>;

    /// Removes the directory at `path` and everything under it.
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Writes `contents` to standard output.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
