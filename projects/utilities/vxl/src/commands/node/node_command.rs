use crate::{Dependencies, Result, commands::NodeSet};
use clap::Subcommand;

/// The `node` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum NodeCommand {
    #[command(name = "set")]
    NodeSet(NodeSet),
}

impl NodeCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            NodeCommand::NodeSet(set) => set.execute(dependencies),
        }
    }
}
