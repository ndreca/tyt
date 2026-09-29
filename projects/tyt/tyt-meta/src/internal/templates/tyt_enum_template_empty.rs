/// Renders a new crate's root command enum with no commands yet.
pub fn tyt_enum_template_empty(root_enum: &str, description: &str) -> String {
    format!(
        r#"use crate::{{Dependencies, Result}};
use clap::Subcommand;

/// {description}
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum {root_enum} {{}}

impl {root_enum} {{
    /// Runs the command.
    pub fn execute(self, _dependencies: impl Dependencies) -> Result<()> {{
        match self {{}}
    }}
}}
"#
    )
}
