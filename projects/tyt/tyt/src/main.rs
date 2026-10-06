use clap::{CommandFactory, Parser, Subcommand};
use std::process;
use ty_clap::{Completion, DependenciesImpl as ClapDependenciesImpl};
use tyt::{DependenciesImpl, Tyt};

/// Tyleo's tools — a collection of command-line utilities for working with
/// files, images, materials, and more.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with tyt.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    Tyt(Tyt),
}

/// The commands that set up other tools to work with tyt.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Integration {
    /// Prints and installs shell completions.
    #[command(name = "completion", subcommand)]
    Completion(Completion),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Integration(Integration::Completion(completion)) => {
            if let Err(e) = completion.execute(&ClapDependenciesImpl, Cli::command(), "tyt") {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }

        Command::Tyt(tyt) => {
            if let Err(e) = tyt.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
