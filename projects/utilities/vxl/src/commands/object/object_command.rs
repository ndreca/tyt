use crate::{Dependencies, Result, commands::ObjectRemove};
use clap::Subcommand;

/// The `object` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ObjectCommand {
    #[command(name = "remove")]
    ObjectRemove(ObjectRemove),
}

impl ObjectCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ObjectCommand::ObjectRemove(remove) => remove.execute(dependencies),
        }
    }
}
