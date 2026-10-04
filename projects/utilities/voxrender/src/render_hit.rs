use crate::BRenderPlacement;
use branded_id::U32Id;
use voxcore::BVoxVoxel;
use voxsurface::SurfaceSpan;

/// One surface hit of a ray: the face through which it entered a live
/// cell of a material other than the one it was inside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderHit {
    /// The placement the ray hit.
    pub placement_id: U32Id<BRenderPlacement>,

    /// The cell the ray hit.
    pub voxel_id: U32Id<BVoxVoxel>,

    /// The face the ray entered through, a unit span in the object's grid
    /// units whose normal faces the ray.
    pub face: SurfaceSpan,

    /// How far along the face's `u` and `v` the hit landed, each in
    /// `[0, 1]`.
    pub along: [f64; 2],

    /// The distance from the ray's origin, in meters.
    pub distance: f64,
}
