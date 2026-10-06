use crate::{ColorFormat, Dependencies, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a Voxel Json document into a `.vmax` package directory.
#[derive(Clone, Debug, Parser)]
#[command(name = "from-voxj")]
pub struct FromVoxj {
    /// The input `.voxj` or `.voxjz` document.
    #[arg(value_name = "input-voxj")]
    input_voxj: PathBuf,

    /// The output `.vmax` package directory to write.
    #[arg(value_name = "output-vmax")]
    output_vmax: PathBuf,

    /// Where to store object colors in the package.
    #[arg(value_name = "color-format", long, default_value = "png")]
    color_format: ColorFormat,
}

impl FromVoxj {
    /// Rebuilds the `.vmax` package from the Voxel Json document.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let voxj_bytes = dependencies.read_file(&self.input_voxj)?;
        dependencies.write_vmax_package(&voxj_bytes, &self.output_vmax, self.color_format)?;
        Ok(())
    }
}
