#![deny(rustdoc::broken_intra_doc_links)]

//! The TypeScript builder that records a voxel model as an SDF Json (`.sdfj`)
//! document, embedded for a Rust host to run. A run writes
//! [`SDFJ_BUILDER_FILES`] under one directory and a `library.json` beside
//! `ts/main.ts`, then starts `ts/main.ts` under Node, Bun, or Deno.

// Public API

mod sdfj_builder_file;
mod sdfj_builder_files;

pub use sdfj_builder_file::*;
pub use sdfj_builder_files::*;
