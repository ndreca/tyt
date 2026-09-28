#[allow(clippy::module_inception)]
mod palette_quantize;

pub use palette_quantize::*;

mod internal;
pub(crate) use internal::*;
