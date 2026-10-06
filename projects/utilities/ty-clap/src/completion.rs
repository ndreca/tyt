use crate::{CompletionVerb, Dependencies};
use clap::{Command, Subcommand};
use clap_complete::Shell;
use std::io::Result as IOResult;

/// The `integration completion` commands a binary nests under its
/// `integration` group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "shell")]
pub enum Completion {
    /// Bash completions.
    #[command(name = "bash", subcommand)]
    Bash(CompletionVerb),

    /// Elvish completions.
    #[command(name = "elvish", subcommand)]
    Elvish(CompletionVerb),

    /// Fish completions.
    #[command(name = "fish", subcommand)]
    Fish(CompletionVerb),

    /// PowerShell completions.
    #[command(name = "powershell", subcommand)]
    PowerShell(CompletionVerb),

    /// Zsh completions.
    #[command(name = "zsh", subcommand)]
    Zsh(CompletionVerb),
}

impl Completion {
    /// Runs the command for the binary `bin`, which `command` describes.
    pub fn execute(
        self,
        dependencies: &impl Dependencies,
        command: Command,
        bin: &str,
    ) -> IOResult<()> {
        let (shell, verb) = match self {
            Completion::Bash(verb) => (Shell::Bash, verb),
            Completion::Elvish(verb) => (Shell::Elvish, verb),
            Completion::Fish(verb) => (Shell::Fish, verb),
            Completion::PowerShell(verb) => (Shell::PowerShell, verb),
            Completion::Zsh(verb) => (Shell::Zsh, verb),
        };

        verb.execute(dependencies, command, bin, shell)
    }
}
