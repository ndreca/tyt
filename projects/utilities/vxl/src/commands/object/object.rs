use crate::{Dependencies, Result, commands::ObjectCommand};
use clap::Parser;

/// Edits objects, writing Voxel JSON.
#[derive(Clone, Debug, Parser)]
#[command(name = "object")]
pub struct Object {
    #[clap(subcommand)]
    pub command: ObjectCommand,
}

impl Object {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
