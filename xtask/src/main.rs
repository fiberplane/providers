mod commands;
mod constants;
mod errors;

use clap::{Command, Parser, Subcommand};
use commands::{build_command, build_providers};
use console::{style, Emoji};
use errors::TaskError;

static ERROR: Emoji<'_, '_> = Emoji("🤒 ", "");

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Cleans all target folders
    Build,
}

fn main() {
    if let Err(err) = handle_cli() {
        println!("{ERROR}{}", style(format!("Error: {err}")).red());
    }
}

fn handle_cli() -> anyhow::Result<()> {
    let matches = Command::new("xtask")
        .subcommand(build_command())
        .get_matches();

    match matches.subcommand() {
        Some(("build", arguments)) => build_providers(arguments),
        _ => Err(TaskError::UnknownCommand.into()),
    }
}
