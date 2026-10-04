use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use tyt_vmax::{DependenciesImpl, TytVMax};

/// Commands for working with Voxel Max.
#[derive(Clone, Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Prints files for other tools.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    TytVMax(TytVMax),
}

/// The commands that print a file for another tool.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Integration {
    /// Prints shell completions.
    #[command(name = "completion", subcommand)]
    Completion(Completion),
}

/// The commands for shell completions.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Completion {
    /// Prints the completions for a shell.
    #[command(name = "print")]
    Print {
        /// The shell to print completions for.
        #[arg(value_name = "shell")]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Integration(Integration::Completion(Completion::Print { shell })) => {
            clap_complete::generate(shell, &mut Cli::command(), "vmax", &mut io::stdout());
        }

        Command::TytVMax(cmd) => {
            if let Err(e) = cmd.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
