use clap::{CommandFactory, Parser, Subcommand};
use std::process;
use ty_clap::{Completion, DependenciesImpl as ClapDependenciesImpl};
use tyt_cubemap::{DependenciesImpl, TytCubemap};

/// Works with cubemap images.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with cubemap.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    TytCubemap(TytCubemap),
}

/// The commands that set up other tools to work with cubemap.
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
            if let Err(e) = completion.execute(&ClapDependenciesImpl, Cli::command(), "cubemap") {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }

        Command::TytCubemap(cubemap) => {
            if let Err(e) = cubemap.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
