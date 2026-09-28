use crate::{Dependencies, Result, commands::ProfilePaletteShow};
use clap::Subcommand;

/// The `profile palette` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfilePaletteCommand {
    #[command(name = "show")]
    ProfilePaletteShow(ProfilePaletteShow),
}

impl ProfilePaletteCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfilePaletteCommand::ProfilePaletteShow(show) => show.execute(dependencies),
        }
    }
}
