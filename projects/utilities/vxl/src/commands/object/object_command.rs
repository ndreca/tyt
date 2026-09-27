use crate::{
    Dependencies, Result,
    commands::{
        ObjectAdd, ObjectDuplicate, ObjectLink, ObjectRemove, ObjectReorder, ObjectSet, ObjectTrim,
        ObjectUnlink,
    },
};
use clap::Subcommand;

/// The `object` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ObjectCommand {
    #[command(name = "add")]
    ObjectAdd(ObjectAdd),
    #[command(name = "duplicate")]
    ObjectDuplicate(ObjectDuplicate),
    #[command(name = "link")]
    ObjectLink(ObjectLink),
    #[command(name = "remove")]
    ObjectRemove(ObjectRemove),
    #[command(name = "reorder")]
    ObjectReorder(ObjectReorder),
    #[command(name = "set")]
    ObjectSet(ObjectSet),
    #[command(name = "trim")]
    ObjectTrim(ObjectTrim),
    #[command(name = "unlink")]
    ObjectUnlink(ObjectUnlink),
}

impl ObjectCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ObjectCommand::ObjectAdd(add) => add.execute(dependencies),
            ObjectCommand::ObjectDuplicate(duplicate) => duplicate.execute(dependencies),
            ObjectCommand::ObjectLink(link) => link.execute(dependencies),
            ObjectCommand::ObjectRemove(remove) => remove.execute(dependencies),
            ObjectCommand::ObjectReorder(reorder) => reorder.execute(dependencies),
            ObjectCommand::ObjectSet(set) => set.execute(dependencies),
            ObjectCommand::ObjectTrim(trim) => trim.execute(dependencies),
            ObjectCommand::ObjectUnlink(unlink) => unlink.execute(dependencies),
        }
    }
}
