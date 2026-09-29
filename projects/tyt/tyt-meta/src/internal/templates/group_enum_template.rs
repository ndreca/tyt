/// Renders the source for a parent command's empty subcommand enum.
pub fn group_enum_template(name: &str, description: &str) -> String {
    format!(
        r#"use crate::{{Dependencies, Result}};
use clap::Subcommand;

/// {description}
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum {name}Command {{}}

impl {name}Command {{
    /// Runs the command.
    pub fn execute(self, _dependencies: impl Dependencies) -> Result<()> {{
        match self {{}}
    }}
}}
"#
    )
}
