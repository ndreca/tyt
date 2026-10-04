use crate::operations::sdf_doc::SdfCellBounds;
use ty_math::TyVector3I32;

/// What one step wrote as it ran.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdfStepRecord {
    /// How many cells the step wrote: every cell an `add` or a `set` filled,
    /// every live cell a `carve` emptied, and every live cell a `paint` or a
    /// `coat` recolored.
    pub written: u64,

    /// The box around the cells the step wrote, or `None` when it wrote none.
    pub written_bounds: Option<SdfCellBounds>,
}

impl SdfStepRecord {
    /// The record with `cell` written as well.
    pub fn with(self, cell: TyVector3I32) -> Self {
        let bounds = SdfCellBounds {
            min: cell,
            max: cell,
        };

        Self {
            written: self.written + 1,
            written_bounds: Some(
                self.written_bounds
                    .map_or(bounds, |written| written.with(cell)),
            ),
        }
    }
}
