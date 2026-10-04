use sdfcore::SdfStepMaterial;

/// One cell of a grid.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdfCell {
    /// What the cell holds, or `None` for an empty cell.
    pub material: Option<SdfStepMaterial>,

    /// The index in the object's step list of the last step that changed the
    /// cell, or `None` for a cell no step changed.
    pub step: Option<u32>,
}
