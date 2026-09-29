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
mod fit_distance;
mod fit_margin;
mod fit_scale;
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
pub use fit_distance::*;
pub use fit_margin::*;
pub use fit_scale::*;
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

// Optional API

#[cfg(feature = "cpu")]
mod cast_ray;

#[cfg(feature = "cpu")]
pub use cast_ray::*;

#[cfg(feature = "cpu")]
mod render_hit;

#[cfg(feature = "cpu")]
pub use render_hit::*;

#[cfg(feature = "cpu")]
mod render_ray;

#[cfg(feature = "cpu")]
pub use render_ray::*;

#[cfg(feature = "cpu")]
mod render_view_rays;

#[cfg(feature = "cpu")]
pub use render_view_rays::*;

// Internal API

mod fit_radius;

pub(crate) use fit_radius::*;
