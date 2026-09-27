use crate::{Dependencies, Result, commands::NodeCommand};
use clap::Parser;

/// Edits nodes, writing Voxel JSON.
#[derive(Clone, Debug, Parser)]
#[command(name = "node")]
pub struct Node {
    #[clap(subcommand)]
    pub command: NodeCommand,
}

impl Node {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
