use crate::{
    Dependencies, Result,
    commands::{Find, MoveToScratch, Rel},
};
use clap::Subcommand;

/// Works with the filesystem.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFS {
    #[command(name = "find")]
    Find(Find),

    #[command(name = "move-to-scratch")]
    MoveToScratch(MoveToScratch),

    #[command(name = "rel")]
    Rel(Rel),
}

impl TytFS {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytFS::Find(find) => find.execute(dependencies),
            TytFS::MoveToScratch(move_to_scratch) => move_to_scratch.execute(dependencies),
            TytFS::Rel(rel) => rel.execute(dependencies),
        }
    }
}
