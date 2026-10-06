use crate::{Dependencies, Result, commands};
use clap::Subcommand;

/// Works with images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytImage {
    #[command(name = "pixelate")]
    Pixelate(commands::Pixelate),

    #[command(name = "square-image")]
    SquareImage(commands::SquareImage),
}

impl TytImage {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytImage::Pixelate(cmd) => cmd.execute(dependencies),
            TytImage::SquareImage(cmd) => cmd.execute(dependencies),
        }
    }
}
