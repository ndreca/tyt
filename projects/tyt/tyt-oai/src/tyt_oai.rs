use crate::{Dependencies, Result, commands::Img};
use clap::Subcommand;

/// Commands for working with the OpenAI API.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytOAI {
    #[command(name = "img")]
    Img(Img),
}

impl TytOAI {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytOAI::Img(img) => img.execute(dependencies),
        }
    }
}
