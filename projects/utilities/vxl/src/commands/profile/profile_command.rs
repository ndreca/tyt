use crate::{Dependencies, Result, commands::ProfileObjectMesh};
use clap::Subcommand;

/// The `profile` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileCommand {
    #[command(name = "object-mesh")]
    ProfileObjectMesh(ProfileObjectMesh),
}

impl ProfileCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileCommand::ProfileObjectMesh(object_mesh) => object_mesh.execute(dependencies),
        }
    }
}
