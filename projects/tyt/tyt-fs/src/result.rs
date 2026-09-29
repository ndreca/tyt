use crate::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is a tyt-fs [`Error`].
pub type Result<T> = StdResult<T, Error>;
