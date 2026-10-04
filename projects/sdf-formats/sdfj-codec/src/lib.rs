#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes SDF Json `.sdfj` documents over the data types the `sdfj`
//! crate defines. The JSON coding comes in through the
//! [`dependencies`] traits.

// Public API

pub mod dependencies;

mod error;
mod from_sdfj_file_bytes;
mod result;
mod to_sdfj_file_bytes;
mod to_sdfj_pretty_file_bytes;

pub use dependencies::*;
pub use error::*;
pub use from_sdfj_file_bytes::*;
pub use result::*;
pub use to_sdfj_file_bytes::*;
pub use to_sdfj_pretty_file_bytes::*;
