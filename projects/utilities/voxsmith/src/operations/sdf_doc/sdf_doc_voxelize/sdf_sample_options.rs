use crate::utilities::{GridResolution, VoxelFrame};

/// The options [`sample`](crate::operations::sdf_doc::sample()) samples under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfSampleOptions {
    /// How the voxel size is chosen.
    pub resolution: GridResolution,

    /// The frame each part's grid is built in. Under
    /// [`VoxelFrame::World`] each place of a part samples on its own, and
    /// under [`VoxelFrame::Local`] each part samples once for all its places.
    pub frame: VoxelFrame,
}
