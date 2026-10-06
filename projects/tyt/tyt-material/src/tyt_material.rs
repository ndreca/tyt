use crate::{Dependencies, Result, commands};
use clap::Subcommand;

/// Works with material textures.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMaterial {
    #[command(name = "create-mse")]
    CreateMse(commands::CreateMse),
}

impl TytMaterial {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytMaterial::CreateMse(create_mse) => create_mse.execute(dependencies),
        }
    }
}
