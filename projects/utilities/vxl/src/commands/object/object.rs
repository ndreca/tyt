use crate::{Dependencies, Result, commands::ObjectCommand};
use clap::Parser;

/// Edits objects into Voxel JSON or meshes them into glTF.
#[derive(Clone, Debug, Parser)]
#[command(name = "object")]
pub struct Object {
    #[clap(subcommand)]
    pub command: ObjectCommand,
}

impl Object {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
