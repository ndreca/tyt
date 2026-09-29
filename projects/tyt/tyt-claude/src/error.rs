use crate::Scope;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    /// `claude` is not on `PATH`.
    ClaudeNotFound,

    /// An I/O operation failed.
    IO(IOError),

    /// No profile was passed and none is active.
    NoActiveProfile,

    /// A repo scope was requested outside a git repository.
    NoGitRoot,

    /// The user home directory could not be determined.
    NoUserHome,

    /// The target config file already defines the profile.
    ProfileAlreadyExists {
        /// The profile name.
        name: String,

        /// The scope whose file defines it.
        scope: Scope,
    },

    /// No config file in the cascade defines the named profile.
    ProfileNotFound {
        /// The profile name.
        name: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::ClaudeNotFound => f.write_str(
                "could not find 'claude' on PATH; install Claude Code or adjust your PATH",
            ),

            Error::IO(e) => e.fmt(f),

            Error::NoActiveProfile => f.write_str(
                "no active claude profile; run 'tyt claude set-profile <name>' or pass --profile",
            ),

            Error::NoGitRoot => f.write_str(
                "not inside a git repository; --scope=repo and --scope=repo-user require one",
            ),

            Error::NoUserHome => f.write_str("could not determine the user home directory"),

            Error::ProfileAlreadyExists { name, scope } => {
                write!(f, "profile '{name}' already exists in --scope={scope}")
            }

            Error::ProfileNotFound { name } => write!(f, "profile '{name}' is not defined"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ClaudeNotFound
            | Error::NoActiveProfile
            | Error::NoGitRoot
            | Error::NoUserHome
            | Error::ProfileAlreadyExists { .. }
            | Error::ProfileNotFound { .. } => None,

            Error::IO(e) => Some(e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
