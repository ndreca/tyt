use crate::{Dependencies, Result, commands::SdfDocCommand};
use clap::Parser;

/// Works with SDF documents.
#[derive(Clone, Debug, Parser)]
#[command(name = "sdf-doc")]
pub struct SdfDoc {
    #[clap(subcommand)]
    pub command: SdfDocCommand,
}

impl SdfDoc {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
