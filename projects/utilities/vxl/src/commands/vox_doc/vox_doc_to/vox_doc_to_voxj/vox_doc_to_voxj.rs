use crate::{Dependencies, ObjectSelection, Result, VoxelInput, VoxjOutput, commands::convert};
use clap::Parser;

/// Converts a voxel file to the Voxel JSON format.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxj")]
pub struct VoxDocToVoxj {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl VoxDocToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let (to, output) = self.output.resolve(&self.input.path);

        convert(
            &dependencies,
            &self.input.path,
            from,
            &output,
            &to,
            &self.selection,
        )
    }
}
