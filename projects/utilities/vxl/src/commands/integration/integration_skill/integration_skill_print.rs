use crate::{AgentSkill, Dependencies, Result};
use clap::Parser;

/// Prints a skill as a `SKILL.md`.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "print",
    after_help = "`vxl integration skill install` writes the skill where agents load it."
)]
pub struct IntegrationSkillPrint {
    /// The skill to print.
    #[arg(value_name = "skill")]
    skill: AgentSkill,
}

impl IntegrationSkillPrint {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        Ok(dependencies.write_stdout(self.skill.skill_md().as_bytes())?)
    }
}
