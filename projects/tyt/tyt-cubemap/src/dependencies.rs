use crate::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Dependency injection trait for cubemap operations.
pub trait Dependencies {
    /// Creates a fresh temporary directory and returns its path.
    fn create_temp_dir(&self) -> Result<PathBuf>;

    /// Runs `ffmpeg` with `args` and returns its stdout.
    fn exec_ffmpeg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Runs `magick` with `args` and returns its stdout.
    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Removes `path` and everything under it.
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Renames the file at `from` to `to`.
    fn rename_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()>;

    /// Writes `contents` to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
