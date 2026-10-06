use crate::{Dependencies, Error, Result};
use clap::Parser;
use std::{ffi::OsString, process};

/// Runs `claude` with `CLAUDE_CONFIG_DIR` set to the active profile's path.
///
/// The active profile is `--profile <name>` when given, otherwise the
/// cascade-resolved `claude.active` from `.tytusrconfig`, `.tytconfig`, or
/// `~/.tytconfig`, the most local winning.
#[derive(Clone, Debug, Parser)]
#[command(name = "run", trailing_var_arg = true)]
pub struct Run {
    /// Overrides the active profile for this invocation only.
    #[arg(value_name = "name", short = 'p', long = "profile")]
    pub profile: Option<String>,

    /// Arguments forwarded verbatim to `claude`.
    #[arg(value_name = "claude-args", allow_hyphen_values = true, num_args = 0..)]
    pub args: Vec<OsString>,
}

impl Run {
    /// Runs `claude` under the profile and exits with its code.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let resolved = dependencies.claude_prefs()?;
        let name = match self.profile {
            Some(ref name) => name.as_str(),
            None => resolved.active.as_deref().ok_or(Error::NoActiveProfile)?,
        };
        let path = resolved
            .profiles
            .get(name)
            .ok_or_else(|| Error::ProfileNotFound {
                name: name.to_string(),
            })?;
        let env = vec![(OsString::from("CLAUDE_CONFIG_DIR"), OsString::from(path))];
        let code = dependencies.exec_claude_with_env(&env, &self.args)?;
        // `run` is a process passthrough: forward the child's exit code
        // verbatim instead of letting `main` always exit 1 on Err.
        process::exit(code);
    }
}
