use std::io::{self, Result, Write};

/// Writes `contents` to stdout.
pub fn write_stdout(contents: &[u8]) -> Result<()> {
    io::stdout().write_all(contents)
}
