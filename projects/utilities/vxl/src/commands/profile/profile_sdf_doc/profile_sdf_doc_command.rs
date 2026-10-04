use crate::{Dependencies, Result, commands::ProfileSdfDocBuild};
use clap::Subcommand;

/// The `profile sdf-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileSdfDocCommand {
    #[command(name = "build")]
    ProfileSdfDocBuild(ProfileSdfDocBuild),
}

impl ProfileSdfDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileSdfDocCommand::ProfileSdfDocBuild(build) => build.execute(dependencies),
        }
    }
}
