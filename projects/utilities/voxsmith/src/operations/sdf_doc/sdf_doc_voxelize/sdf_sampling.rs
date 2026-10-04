use crate::operations::sdf_doc::{SdfGrid, SdfPlace};

/// A model's parts sampled into grids of cells.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfSampling {
    /// The edge length of one cell, in meters.
    pub voxel_size: f64,

    /// Every place of every part, each before its children, from the first
    /// root part.
    pub places: Vec<SdfPlace>,

    /// The grids the places index.
    pub grids: Vec<SdfGrid>,
}
