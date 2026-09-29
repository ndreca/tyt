use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
use tyt_common::ExecFailed;

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    /// Neither `.tytconfig` nor `.tytusrconfig` exists in the git root.
    ConfigNotFound,

    /// An I/O operation failed.
    IO(IOError),

    /// No `fs.rel` key matches the given base name.
    RelBaseNotFound(String),

    /// `rg` failed.
    Rg(ExecFailed),

    /// `fs.move-to-scratch.scratchDir` is missing from `.tytconfig`.
    ScratchDirNotConfigured,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::ConfigNotFound => {
                write!(f, "no .tytconfig or .tytusrconfig found in the git root")
            }

            Error::IO(e) => e.fmt(f),

            Error::RelBaseNotFound(name) => write!(
                f,
                "no `fs.rel` entry named `{name}` found in .tytconfig or .tytusrconfig"
            ),

            Error::Rg(ExecFailed {
                exit_code,
                stdout,
                stderr,
            }) => {
                match exit_code {
                    Some(code) => write!(f, "rg exited with code {code}")?,
                    None => write!(f, "rg killed by signal")?,
                }
                if !stdout.is_empty() {
                    write!(f, "\nstdout:\n{stdout}")?;
                }
                if !stderr.is_empty() {
                    write!(f, "\nstderr:\n{stderr}")?;
                }
                Ok(())
            }

            Error::ScratchDirNotConfigured => write!(
                f,
                "scratchDir is not configured; add {{\"fs\": {{\"move-to-scratch\": {{\"scratchDir\": \"<path>\"}}}}}} to .tytconfig"
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::ConfigNotFound
            | Error::RelBaseNotFound(_)
            | Error::Rg(_)
            | Error::ScratchDirNotConfigured => None,
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
