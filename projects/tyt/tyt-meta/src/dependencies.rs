use crate::Result;
use std::path::{Path, PathBuf};

/// Dependencies for meta operations.
pub trait Dependencies {
    /// Creates `path` and any missing parent directories.
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Immediate child paths of `path`.
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Vec<PathBuf>>;

    /// Reads the file at `path` as UTF-8 text.
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String>;

    /// Writes `contents` to `path`, replacing any existing file.
    fn write<P: AsRef<Path>>(&self, path: P, contents: &str) -> Result<()>;

    /// Writes `contents` to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;

    /// The nearest ancestor of the current directory whose `Cargo.toml`
    /// declares a `[workspace]`.
    fn workspace_root(&self) -> Result<PathBuf>;
}
