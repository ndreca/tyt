use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use vxl::{AgentSkill, DependenciesImpl, Error, Vxl};

/// A command-line tool for working with voxels.
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
    Vxl(Box<Vxl>),
}

/// The commands that print a file for another tool.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Integration {
    /// Prints shell completions.
    #[command(name = "completion", subcommand)]
    Completion(Completion),

    /// Prints agent skills.
    #[command(name = "skill", subcommand)]
    Skill(Skill),
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

/// The commands for agent skills.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Skill {
    /// Prints a skill as a `SKILL.md`.
    #[command(name = "print")]
    Print {
        /// The skill to print.
        #[arg(value_name = "skill")]
        skill: AgentSkill,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Integration(Integration::Completion(Completion::Print { shell })) => {
            clap_complete::generate(shell, &mut Cli::command(), "vxl", &mut io::stdout());
        }

        Command::Integration(Integration::Skill(Skill::Print { skill })) => {
            print!("{}", skill.skill_md());
        }

        Command::Vxl(cmd) => {
            if let Err(e) = cmd.execute(DependenciesImpl) {
                match e {
                    Error::Usage(clap_error) => clap_error.exit(),

                    e => {
                        eprintln!("error: {e}");
                        process::exit(1);
                    }
                }
            }
        }
    }
}
