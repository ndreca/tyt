use mvox::MVoxRotation;
use ty_math::{TyMatrix4x4F64, TyTransformF64};

/// How far a turned axis may sit from a whole `-1`, `0`, or `1` and still
/// read as one.
const SIGNED_TOLERANCE: f64 = 1e-6;

/// The packed rotation byte of a node's rotation and scale on MagicaVoxel's
/// Z-up axes, or `None` when they are not a signed permutation.
pub fn frame_rotation(transform: &TyTransformF64) -> Option<u8> {
    let turned = transform.yup_to_zup();
    let matrix = TyMatrix4x4F64::from_scale_rotation_translation(
        turned.scale,
        turned.rotation,
        Default::default(),
    );

    let mut signed = [[0i8; 3]; 3];
    for (row, entries) in signed.iter_mut().enumerate() {
        for (column, entry) in entries.iter_mut().enumerate() {
            let value = matrix.col(column)[row];
            let rounded = value.round();
            if (value - rounded).abs() > SIGNED_TOLERANCE || rounded.abs() > 1.0 {
                return None;
            }
            *entry = rounded as i8;
        }
    }

    MVoxRotation::from_matrix(signed).map(|rotation| rotation.0)
}
