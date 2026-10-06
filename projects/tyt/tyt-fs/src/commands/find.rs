use crate::{Dependencies, Result, find_files};
use clap::Parser;

/// Finds files using gitignore-style patterns.
#[derive(Clone, Debug, Parser)]
#[command(name = "find")]
pub struct Find {
    /// Gitignore-style patterns selecting files.
    #[arg(value_name = "pattern", required = true)]
    patterns: Vec<String>,
}

impl Find {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let stdout = find_files(&dependencies, &self.patterns)?;
        if !stdout.is_empty() {
            dependencies.write_stdout(&stdout)?;
        }
        Ok(())
    }
}
