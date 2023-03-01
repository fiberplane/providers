use crate::constants::*;
use clap::{arg, ArgMatches, Command};
use console::style;
use duct::cmd;
use std::fs;

pub(crate) fn build_command() -> Command {
    Command::new("build").args(&[
        arg!(-d --debug "keep debugging information in the built provider(s)"),
        arg!([provider] "provider to build (default: all)"),
    ])
}

pub(crate) fn build_providers(args: &ArgMatches) -> anyhow::Result<()> {
    fs::create_dir_all("artifacts")?;

    let providers = match args.get_one::<String>("provider") {
        Some(provider) if provider != "all" => vec![provider.clone()],
        _ => PROVIDERS.iter().cloned().map(str::to_owned).collect(),
    };

    let is_debug = args.get_flag("debug");

    for provider in providers {
        println!(
            "{BUILD}{}",
            format!("Building {} provider...", style(&provider).cyan().bold())
        );

        let mut args = vec!["build"];
        if !is_debug {
            args.push("--release")
        }

        cmd("cargo", args)
            .dir(format!("providers/{provider}"))
            .stdout_to_stderr()
            .stderr_capture()
            .run()?;

        let artifact = format!("artifacts/{provider}.wasm");

        if is_debug {
            let input = format!("target/wasm32-unknown-unknown/debug/{provider}_provider.wasm");
            fs::copy(input, artifact)?;
        } else {
            println!(
                "{OPTIMIZE}{}",
                format!("Optimizing {} provider...", style(&provider).cyan().bold())
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
