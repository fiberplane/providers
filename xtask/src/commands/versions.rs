use clap::Parser;
use fiberplane_ci::{commands::versions::*, TaskResult};

#[derive(Parser)]
pub struct VersionArgs {
    #[clap(subcommand)]
    sub_command: VersionCommand,
}

#[derive(Parser)]
pub enum VersionCommand {
    /// Sets the version for the `fiberplane-pdk` crates.
    #[clap()]
    Set(SetVersionArgs),
}

pub fn handle_version_command(args: &VersionArgs) -> TaskResult {
    match &args.sub_command {
        VersionCommand::Set(args) => handle_set_version(args),
    }
}

fn handle_set_version(args: &SetVersionArgs) -> TaskResult {
    set_version("Cargo.toml", args)?;

    Ok(())
}
