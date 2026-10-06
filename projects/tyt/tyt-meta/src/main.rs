use clap::{CommandFactory, Parser, Subcommand};
use std::process;
use ty_clap::{Completion, DependenciesImpl as ClapDependenciesImpl};
use tyt_meta::{DependenciesImpl, TytMeta};

/// Meta-tools for scaffolding new tyt sub-crates and commands.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with meta.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    TytMeta(TytMeta),
}

/// The commands that set up other tools to work with meta.
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
            if let Err(e) = completion.execute(&ClapDependenciesImpl, Cli::command(), "meta") {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }

        Command::TytMeta(meta) => {
            if let Err(e) = meta.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
