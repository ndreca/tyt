/// Renders the source for a leaf command, a `Parser` struct whose `execute`
/// prints a greeting.
pub fn command_file_template(name: &str, command: &str, description: &str) -> String {
    format!(
        r#"use crate::{{Dependencies, Result}};
use clap::Parser;

/// {description}
#[derive(Clone, Debug, Parser)]
#[command(name = "{command}")]
pub struct {name} {{}}

impl {name} {{
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {{
        dependencies.write_stdout(b"Hello from {command}!\n")?;
        Ok(())
    }}
}}
"#
    )
}
