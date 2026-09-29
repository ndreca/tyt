#![deny(rustdoc::broken_intra_doc_links)]

//! The render contract for voxel scenes and its CPU reference renderer.
//!
//! The contract types describe an image of a scene: the materials, the
//! lights, the views, and the output buffer. Every renderer consumes them.
//! The CPU renderer behind the default `cpu` feature follows the contract
//! literally and makes the images every other renderer is tested against.
//! Every entity has a brand, so nothing is addressed by a bare integer. The
//! only integers are pixel coordinates.

// Public API

mod b_render_light;
mod b_render_material;
mod b_render_placement;
mod b_render_view;
mod error;
mod render_image;
mod render_light;
mod render_material;
mod render_object;
mod render_occlusion;
mod render_placement;
mod render_projection;
mod render_scene;
mod render_shadow;
mod render_view;
mod result;

pub use b_render_light::*;
pub use b_render_material::*;
pub use b_render_placement::*;
pub use b_render_view::*;
pub use error::*;
pub use render_image::*;
pub use render_light::*;
pub use render_material::*;
pub use render_object::*;
pub use render_occlusion::*;
pub use render_placement::*;
pub use render_projection::*;
pub use render_scene::*;
pub use render_shadow::*;
pub use render_view::*;
pub use result::*;
