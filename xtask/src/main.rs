mod commands;
mod constants;
mod errors;

use clap::Command;
use commands::{build_command, build_providers};
use console::style;
use constants::ERROR;
use errors::TaskError;

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
