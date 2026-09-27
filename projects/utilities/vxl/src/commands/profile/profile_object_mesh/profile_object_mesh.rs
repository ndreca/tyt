use crate::{Dependencies, Result, commands::ProfileObjectMeshCommand};
use clap::Parser;

/// Inspects the profiles `object mesh --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "object-mesh")]
pub struct ProfileObjectMesh {
    #[clap(subcommand)]
    pub command: ProfileObjectMeshCommand,
}

impl ProfileObjectMesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
