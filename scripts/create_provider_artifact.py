#!/usr/bin/env python3

import argparse
import subprocess
import sys
from pathlib import Path


ALL_PROVIDERS = ["loki", "elasticsearch", "cloudwatch", "https", "sentry", "prometheus"]


def compile_provider(provider: str):
    print(f"Compiling {provider} provider...", end=" ")
    try:
        cwd = Path(".") / "providers"
        subprocess.check_output(
            f"cargo build --release -p {provider}-provider",
            cwd=cwd,
            stderr=subprocess.STDOUT,
            shell=True,
        )
    except subprocess.CalledProcessError as e:
        print(f"Error during compilation:\n{e.output}")
        sys.exit(1)
    print("OK!")


def optimize_provider(provider: str, destination_dir: Path):
    print(f"Optimizing {provider} provider...", end=" ")
    try:
        output_path = destination_dir / f"{provider}.wasm"
        input_path = (
            Path(".")
            / "target"
            / "wasm32-unknown-unknown"
            / "release"
            / f"{provider}_provider.wasm"
        )
        subprocess.check_output(
            f'wasm-opt -Oz -c -o "{output_path}" {input_path}',
            stderr=subprocess.STDOUT,
            shell=True,
        )
    except subprocess.CalledProcessError as e:
        print(f"Error during optimization:\n{e.output}")
        sys.exit(1)
    print("OK!")


def single_provider(provider: str, destination_dir: Path):
    compile_provider(provider)
    optimize_provider(provider, destination_dir)


def main(provider_name: str, destination: Path):
    if provider_name.lower() == "all":
        for provider in ALL_PROVIDERS:
            single_provider(provider, destination)
    else:
        single_provider(provider_name, destination)

    sys.exit(0)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="create_provider_artifact",
        description="Create a release-ready WebAssembly artifact for a provider.",
        epilog="Created by Fiberplane <info@fiberplane.com>",
    )
    parser.add_argument(
        "provider_name", help="Name of the provider, use 'all' to do all providers"
    )
    parser.add_argument(
        "-d",
        "--destination",
        help="Destination directory of the provider",
        default=".",
        type=Path,
    )

    args = parser.parse_args()
    main(args.provider_name, args.destination)
