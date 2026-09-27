use crate::{
    Dependencies, Result,
    commands::{ObjectRemove, ObjectReorder, ObjectSet, ObjectTrim},
};
use clap::Subcommand;

/// The `object` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ObjectCommand {
    #[command(name = "remove")]
    ObjectRemove(ObjectRemove),
    #[command(name = "reorder")]
    ObjectReorder(ObjectReorder),
    #[command(name = "set")]
    ObjectSet(ObjectSet),
    #[command(name = "trim")]
    ObjectTrim(ObjectTrim),
}

impl ObjectCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ObjectCommand::ObjectRemove(remove) => remove.execute(dependencies),
            ObjectCommand::ObjectReorder(reorder) => reorder.execute(dependencies),
            ObjectCommand::ObjectSet(set) => set.execute(dependencies),
            ObjectCommand::ObjectTrim(trim) => trim.execute(dependencies),
        }
    }
}
