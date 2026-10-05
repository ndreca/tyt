use crate::operations::sdf_doc::{FACE_OFFSETS, SdfGrid};
use ty_math::TyVector3I32;

/// Whether `cell` meets an empty cell of `grid` across a face. A cell outside
/// the grid counts as empty.
pub fn has_open_face(grid: &SdfGrid, cell: TyVector3I32) -> bool {
    FACE_OFFSETS.iter().any(|face| {
        grid.cell(cell + *face)
            .is_none_or(|neighbor| neighbor.material.is_none())
    })
}
