use crate::{Dependencies, Result, commands::SdfDocBuild};
use clap::Subcommand;

/// The `sdf-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum SdfDocCommand {
    #[command(name = "build")]
    SdfDocBuild(SdfDocBuild),
}

impl SdfDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            SdfDocCommand::SdfDocBuild(build) => build.execute(dependencies),
        }
    }
}
