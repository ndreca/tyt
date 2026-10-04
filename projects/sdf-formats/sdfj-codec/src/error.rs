use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding or encoding an SDF Json (`.sdfj`) document.
#[derive(Debug)]
pub enum Error {
    /// The document JSON could not be parsed or written.
    Json(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Json(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {}
