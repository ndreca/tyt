use serde::de::DeserializeOwned;
use std::io::{Error, ErrorKind, Result};

/// Deserializes JSON bytes into `T`.
pub fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
