use crate::Result;
use std::ffi::OsStr;

/// Dependencies for image operations.
pub trait Dependencies {
    /// Runs `magick` with `args` and returns its stdout.
    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Writes `contents` to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
