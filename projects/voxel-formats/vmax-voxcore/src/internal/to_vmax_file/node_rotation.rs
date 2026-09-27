use crate::{ROTATION_TOLERANCE, VMaxExtNode, decode_axis_angle, encode_axis_angle};
use ty_math::TyTransformF64;

/// The axis-angle a node writes: the preserved spelling while it still
/// decodes to the node's rotation, which keeps a loaded document byte for
/// byte, or the live rotation encoded afresh once the node was rotated after
/// the load. `transform` places the node on Voxel Max's Z-up axes.
pub(crate) fn node_rotation(ext_node: &VMaxExtNode, transform: &TyTransformF64) -> [f64; 4] {
    let stored = decode_axis_angle(ext_node.rotation);
    let live = transform.rotation;
    if stored.abs_diff_eq(live, ROTATION_TOLERANCE) || stored.abs_diff_eq(-live, ROTATION_TOLERANCE)
    {
        return ext_node.rotation;
    }
    encode_axis_angle(live)
}
