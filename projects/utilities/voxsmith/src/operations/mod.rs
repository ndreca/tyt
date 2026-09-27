//! One module per vxl command group, each behind a feature of the same name.

#[cfg(feature = "hierarchy")]
pub mod hierarchy;

#[cfg(feature = "mesh_doc")]
pub mod mesh_doc;

#[cfg(feature = "node")]
pub mod node;

#[cfg(feature = "object")]
pub mod object;

#[cfg(feature = "object_voxels")]
pub mod object_voxels;

#[cfg(feature = "palette")]
pub mod palette;

#[cfg(feature = "profile")]
pub mod profile;

#[cfg(feature = "vox_doc")]
pub mod vox_doc;
