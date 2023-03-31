mod build;
mod publish;
mod versions;

pub(crate) use build::*;
use clap::Parser;
pub(crate) use publish::*;
pub(crate) use versions::*;

#[derive(Parser)]
pub enum Command {
    Build(BuildArgs),

    Publish(PublishArgs),

    #[clap(alias = "versions")]
    Version(VersionArgs),
}
