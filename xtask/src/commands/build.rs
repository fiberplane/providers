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

        let artifact = format!("artifacts/{provider}.wasm");

        if args.debug {
            let input = format!("target/wasm32-unknown-unknown/debug/{provider}_provider.wasm");
            fs::copy(input, artifact)?;
        } else {
            println!(
                "{OPTIMIZE}Optimizing {} provider...",
                style(&provider).cyan().bold()
            );

            let input = format!("target/wasm32-unknown-unknown/release/{provider}_provider.wasm");
            wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
                .set_converge()
                .run(input, artifact)?;
        }
    }

    println!("{SUCCESS}Done.");

    Ok(())
}
