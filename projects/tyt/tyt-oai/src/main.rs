use clap::{CommandFactory, Parser, Subcommand};
use std::process;
use ty_clap::{Completion, DependenciesImpl as ClapDependenciesImpl};
use tyt_oai::{DependenciesImpl, TytOAI};

/// Works with the OpenAI API.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with oai.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    TytOAI(TytOAI),
}

/// The commands that set up other tools to work with oai.
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
            if let Err(e) = completion.execute(&ClapDependenciesImpl, Cli::command(), "oai") {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }

        Command::TytOAI(cmd) => {
            if let Err(e) = cmd.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
