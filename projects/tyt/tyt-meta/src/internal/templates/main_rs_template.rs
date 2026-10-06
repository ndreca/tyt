/// Renders a new crate's `src/main.rs` that runs `root_enum` beside the
/// `integration completion` commands.
pub fn main_rs_template(module: &str, root_enum: &str, command: &str, description: &str) -> String {
    format!(
        r#"use clap::{{CommandFactory, Parser, Subcommand}};
use std::process;
use ty_clap::{{Completion, DependenciesImpl as ClapDependenciesImpl}};
use {module}::{{DependenciesImpl, {root_enum}}};

/// {description}
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {{
    #[clap(subcommand)]
    command: Command,
}}

#[derive(Clone, Debug, Subcommand)]
enum Command {{
    /// Sets up other tools to work with {command}.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    {root_enum}({root_enum}),
}}

/// The commands that set up other tools to work with {command}.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Integration {{
    /// Prints and installs shell completions.
    #[command(name = "completion", subcommand)]
    Completion(Completion),
}}

fn main() {{
    let cli = Cli::parse();
    match cli.command {{
        Command::Integration(Integration::Completion(completion)) => {{
            if let Err(e) = completion.execute(&ClapDependenciesImpl, Cli::command(), "{command}") {{
                eprintln!("error: {{e}}");
                process::exit(1);
            }}
        }}

        Command::{root_enum}(cmd) => {{
            if let Err(e) = cmd.execute(DependenciesImpl) {{
                eprintln!("error: {{e}}");
                process::exit(1);
            }}
        }}
    }}
}}
"#
    )
}
