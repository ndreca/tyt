use clap::{CommandFactory, Parser, Subcommand};
use std::process;
use ty_clap::{Completion, DependenciesImpl as ClapDependenciesImpl};
use tyt_fbx::{DependenciesImpl, TytFbx};

/// Operations on FBX files.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with fbx.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    TytFbx(TytFbx),
}

/// The commands that set up other tools to work with fbx.
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
            if let Err(e) = completion.execute(&ClapDependenciesImpl, Cli::command(), "fbx") {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }

        Command::TytFbx(fbx) => {
            if let Err(e) = fbx.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
