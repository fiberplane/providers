use crate::constants::*;
use anyhow::{bail, Context};
use clap::Parser;
use console::style;
use duct::cmd;
use fiberplane_ci::utils::*;
use fiberplane_ci::TaskResult;

/// Publishable crates in order of publication.
const CRATE_DIRS: &[&str] = &["fiberplane-pdk-macros", "fiberplane-pdk"];

#[derive(Parser)]
pub struct PublishArgs {
    /// Do not actually publish the release(s).
    ///
    /// Note that publication will fail if you try to dry-run publication for
    /// multiple crates that depend on one another. This is because later crates
    /// need their updated dependencies to be really published to be publishable
    /// themselves.
    #[clap(long)]
    pub dry_run: bool,
}

pub async fn handle_publish_command(args: &PublishArgs) -> TaskResult {
    eprintln!("{WORKING}Detecting crates that need publication...");

    let workspace_version = TomlNode::from_file("Cargo.toml")?
        .get_string("workspace.package.version")
        .context("Cannot determine workspace version")?;

    let mut unpublished_crate_dirs = Vec::new();
    let all_crate_dirs = get_publishable_workspace_crate_dirs(".")?;
    for crate_dir in all_crate_dirs.iter() {
        let cargo_toml_path = format!("{crate_dir}/Cargo.toml");
        let crate_cargo_toml = TomlNode::from_file(&cargo_toml_path)?;
        let version = if crate_cargo_toml
            .get_bool("package.version.workspace")
            .unwrap_or_default()
        {
            workspace_version.clone()
        } else {
            match crate_cargo_toml.get_string("package.version") {
                Some(package_version) => package_version,
                None => {
                    eprintln!("{WARN}Cannot determine package version in {cargo_toml_path}.");
                    continue;
                }
            }
        };

        let crate_name = get_crate_name_from_dir(crate_dir);
        if is_published(CRATES_IO_INDEX_URL, crate_name, &version).await? {
            continue;
        }

        eprintln!(
            " - {crate_name} => {version}.",
            version = style(&version).bold()
        );

        unpublished_crate_dirs.push(crate_dir.as_str());
    }

    if unpublished_crate_dirs.is_empty() {
        eprintln!("{SUCCESS}No crates need publishing.");
        return Ok(());
    }

    eprintln!("{CHECK}Unpublished crates detected. Starting publication...");

    publish_crates(args)?;

    eprintln!("{SUCCESS}All crates published.");

    Ok(())
}

fn publish_crates(args: &PublishArgs) -> TaskResult {
    for crate_dir in CRATE_DIRS {
        let mut cargo_args = vec!["publish", "--allow-dirty"];
        if args.dry_run {
            cargo_args.push("--dry-run");
        }

        let output = cmd("cargo", &cargo_args).dir(crate_dir).unchecked().run()?;

        if output.status.code() != Some(0) {
            eprintln!(
                "{WARN}cargo {args} failed with exit code {code:?}.",
                args = cargo_args.join(" "),
                code = output.status.code().unwrap_or(-1),
            );
            bail!("Cargo publication failed in {crate_dir}")
        }

        eprintln!(
            "{CHECK}{crate_name} published.",
            crate_name = style(get_crate_name_from_dir(crate_dir)).bold()
        );
    }

    Ok(())
}
