use crate::{
    Dependencies, Result,
    commands::{IntegrationSkillInstall, IntegrationSkillList, IntegrationSkillPrint},
};
use clap::Subcommand;

/// The `integration skill` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum IntegrationSkillCommand {
    #[command(name = "install")]
    IntegrationSkillInstall(IntegrationSkillInstall),

    #[command(name = "list")]
    IntegrationSkillList(IntegrationSkillList),

    #[command(name = "print")]
    IntegrationSkillPrint(IntegrationSkillPrint),
}

impl IntegrationSkillCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            IntegrationSkillCommand::IntegrationSkillInstall(install) => {
                install.execute(dependencies)
            }

            IntegrationSkillCommand::IntegrationSkillList(list) => list.execute(dependencies),

            IntegrationSkillCommand::IntegrationSkillPrint(print) => print.execute(dependencies),
        }
    }
}
