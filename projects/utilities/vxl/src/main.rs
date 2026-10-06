use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use vxl::{DependenciesImpl, Error, Vxl, commands::IntegrationSkill};

const COMPLETION_INSTALL_HELP: &str = "\
Installing:
  bash        vxl integration completion print bash > ~/.local/share/bash-completion/completions/vxl
  elvish      echo 'eval (vxl integration completion print elvish | slurp)' >> ~/.config/elvish/rc.elv
  fish        vxl integration completion print fish > ~/.config/fish/completions/vxl.fish
  powershell  Add-Content $PROFILE 'vxl integration completion print powershell | Out-String | Invoke-Expression'
  zsh         vxl integration completion print zsh > ~/.zsh/completions/_vxl

Each target directory has to exist. zsh also needs
fpath=(~/.zsh/completions $fpath) before compinit in ~/.zshrc. A new shell then
completes vxl on Tab.";

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with vxl.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    Vxl(Box<Vxl>),
}

/// The commands that set up other tools to work with vxl.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Integration {
    /// Prints shell completions.
    #[command(name = "completion", subcommand)]
    Completion(Completion),

    /// Lists, prints, and installs agent skills.
    #[command(name = "skill")]
    Skill(IntegrationSkill),
}

/// The commands for shell completions.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Completion {
    /// Prints the completions for a shell.
    #[command(name = "print", after_help = COMPLETION_INSTALL_HELP)]
    Print {
        /// The shell to print completions for.
        #[arg(value_name = "shell")]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Integration(Integration::Completion(Completion::Print { shell })) => {
            clap_complete::generate(shell, &mut Cli::command(), "vxl", &mut io::stdout());
            Ok(())
        }

        Command::Integration(Integration::Skill(skill)) => skill.execute(DependenciesImpl),

        Command::Vxl(cmd) => cmd.execute(DependenciesImpl),
    };

    if let Err(e) = result {
        match e {
            Error::Usage(clap_error) => clap_error.exit(),

            e => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Cli;
    use clap::{Parser, error::ErrorKind};

    #[test]
    fn version_prints_the_crate_version() {
        let error = Cli::try_parse_from(["vxl", "--version"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayVersion);
        assert_eq!(
            error.to_string(),
            format!("vxl {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}
