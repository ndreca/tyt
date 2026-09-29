use crate::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is a tyt-vmax [`Error`].
pub type Result<T> = StdResult<T, Error>;
