use crate::{Dependencies, Result, commands::ProfileObject};
use clap::Subcommand;

/// The `profile` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileCommand {
    #[command(name = "object")]
    ProfileObject(ProfileObject),
}

impl ProfileCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileCommand::ProfileObject(object) => object.execute(dependencies),
        }
    }
}
