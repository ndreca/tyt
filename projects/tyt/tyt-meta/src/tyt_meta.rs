use crate::{Dependencies, Result, commands::CreateCommand};
use clap::Subcommand;

/// Scaffolds new tyt sub-crates and commands.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMeta {
    #[command(name = "create-command")]
    CreateCommand(CreateCommand),
}

impl TytMeta {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytMeta::CreateCommand(cmd) => cmd.execute(dependencies),
        }
    }
}
