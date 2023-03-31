use crate::constants::*;
use clap::Parser;
use console::style;
use duct::cmd;
use fiberplane_ci::TaskResult;
use std::fs;

#[derive(Parser)]
pub struct BuildArgs {
    /// Keep debugging information in the built provider(s).
    #[clap(short, long)]
    pub debug: bool,

    /// Provider to build.
    #[clap(default_value = "all")]
    provider: String,
}

pub(crate) fn handle_build_command(args: BuildArgs) -> TaskResult {
    fs::create_dir_all("artifacts")?;

    let providers = if args.provider == "all" {
        PROVIDERS.iter().cloned().map(str::to_owned).collect()
    } else {
        vec![args.provider]
    };

    for provider in providers {
        println!(
            "{BUILD}Building {} provider...",
            style(&provider).cyan().bold()
        );

        let mut cargo_args = vec!["build"];
        if !args.debug {
            cargo_args.push("--release")
        }

        cmd("cargo", cargo_args)
            .dir(format!("providers/{provider}"))
            .stdout_to_stderr()
            .stderr_capture()
            .run()?;

        let artifact_path = format!("artifacts/{provider}.wasm");
        let target_path = "target/wasm32-unknown-unknown";

        if args.debug {
            let bundle_path = format!("{target_path}/debug/{provider}_provider.wasm");
            fs::copy(bundle_path, artifact_path)?;
        } else {
            println!(
                "{OPTIMIZE}Optimizing {} provider...",
                style(&provider).cyan().bold()
            );

            let bundle_path = format!("{target_path}/release/{provider}_provider.wasm");
            wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
                .set_converge()
                .run(bundle_path, artifact_path)?;
        }
    }

    println!("{SUCCESS}Done.");

    Ok(())
}
