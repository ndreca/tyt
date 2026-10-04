use crate::{
    operations::sdf_doc::{SdfGrid, SdfMaterialProperties, SdfPlace},
    utilities::VoxelFrame,
};
use branded_id::IdVec;
use sdfcore::BSdfMaterial;

/// A model's parts sampled into grids of cells.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfSampling {
    /// The edge length of one cell, in meters.
    pub voxel_size: f64,

    /// The frame the grids were sampled in.
    pub frame: VoxelFrame,

    /// Every place of every part, each before its children, from the first
    /// root part.
    pub places: Vec<SdfPlace>,

    /// The grids the places index.
    pub grids: Vec<SdfGrid>,

    /// The properties of each material of the model.
    pub materials: IdVec<BSdfMaterial, SdfMaterialProperties>,
}
