use crate::{Dependencies, Result, commands::ObjectVoxelsCommand};
use clap::Parser;

/// Moves objects' voxels within their grids, writing Voxel JSON.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxels")]
pub struct ObjectVoxels {
    #[clap(subcommand)]
    pub command: ObjectVoxelsCommand,
}

impl ObjectVoxels {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
