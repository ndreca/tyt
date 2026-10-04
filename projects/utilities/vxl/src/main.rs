use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::{io, process};
use vxl::{AgentSkill, DependenciesImpl, Error, Vxl};

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
    #[command(name = "print", after_help = COMPLETION_INSTALL_HELP)]
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
    #[command(name = "print", after_help = skill_install_help())]
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

fn skill_install_help() -> String {
    let installs: Vec<_> = AgentSkill::value_variants()
        .iter()
        .map(|skill| {
            let value = skill
                .to_possible_value()
                .expect("every skill has a command-line value");
            let name = value.get_name();
            format!(
                "  mkdir -p .claude/skills/{name}\n  \
                 vxl integration skill print {name} > .claude/skills/{name}/SKILL.md"
            )
        })
        .collect();
    format!(
        "Installing in the current project for Claude Code:\n{}\n\n\
         With ~/.claude/skills in place of .claude/skills, the commands install the\n\
         skill for every project. A new Claude Code session loads a skill when a\n\
         prompt asks for what the skill does. The prompt \"make a voxel chair\"\n\
         asks for voxel-modeling. Upgrading vxl takes a reprint.",
        installs.join("\n\n")
    )
}
