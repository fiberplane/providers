#!/usr/bin/env python3

import argparse
import subprocess
import sys
import re
from pathlib import Path
from typing import List
import json

# Using urllib instead of requests to _only_ require
# Python 3 stdlib to work
import urllib.request


# The name of the provider is included as 'name' key in the match
PROVIDER_MATCHER = re.compile("^(?P<name>.*)-provider$")
USER_AGENT = "Fiberplane/Release worker/1.0"
# We assume this script is only run on Linux 64-bit platforms
BINARYEN_VERSION = "version_112"
WASM_OPT_URL = f"https://github.com/WebAssembly/binaryen/releases/download/{BINARYEN_VERSION}/binaryen-{BINARYEN_VERSION}-x86_64-linux.tar.gz"


def install_dependencies():
    """
    Install dependencies for the script
    """
    check_wasm_opt = subprocess.run("which wasm-opt", shell=True)
    if check_wasm_opt.returncode != 0:
        if not sys.platform.startswith("linux"):
            print(
                "Automatic dependency installation only works on linux for CI purposes, install wasm-opt manually please"
            )
            sys.exit(1)
        print(f"Installing wasm-opt...", end=" ")
        try:
            request = urllib.request.Request(
                WASM_OPT_URL, headers={"User-Agent": USER_AGENT}
            )
            with urllib.request.urlopen(request) as response:
                tarball_path = Path.home() / "binaryen.tar.gz"
                with open(tarball_path, "wb") as f:
                    f.write(response.read())

            subprocess.check_output(
                f"tar -xzf {tarball_path}",
                stderr=subprocess.STDOUT,
                shell=True,
            )

            subprocess.check_output(
                f"mkdir -p ~/.local/bin/",
                stderr=subprocess.STDOUT,
                shell=True,
            )

            subprocess.check_output(
                f"cp binaryen-{BINARYEN_VERSION}/bin/* ~/.local/bin/",
                stderr=subprocess.STDOUT,
                shell=True,
            )

            # This chmod will fail on missing file, so it
            # also acts as a check that the `wasm-opt` binary is in the correct location
            subprocess.check_output(
                f"chmod a+x ~/.local/bin/wasm-opt",
                stderr=subprocess.STDOUT,
                shell=True,
            )
        except Exception as e:
            print(f"Error during installation:\n{e}")
            sys.exit(1)
        print("OK!")


def list_all_providers(*, deny_list: List[str]) -> List[str]:
    """
    List all provider crates in the current Cargo workspace, except
    the ones present in DENY_LIST.

    A "provider crate" is a crate whose name ends with "-provider" in
    the providers/ subdirectory
    """
    # We don't make sure that the provider crate is in the correct subdirectory
    # here.
    try:
        cargo_metadata = json.loads(
            subprocess.check_output(
                f"cargo metadata --format-version 1 --no-deps",
                shell=True,
            )
        )
        result = []
        for meta in cargo_metadata["packages"]:
            match = PROVIDER_MATCHER.match(meta["name"])
            if match and match.group("name") not in deny_list:
                result.append(match.group("name"))
        return result
    except subprocess.CalledProcessError as e:
        print(f"Cargo error while fetching the current version:\n{e.output}")
        sys.exit(1)


def compile_provider(provider: str):
    """
    Compile the provider named PROVIDER in release mode for WASM target.
    """
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
    """
    Optimize the wasm blob for PROVIDER for release, and leave the artifact in DESTINATION_DIR.
    """
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
    """
    Prepare provider PROVIDER for release, leaving the wasm blob in DESTINATION_DIR.
    """
    compile_provider(provider)
    optimize_provider(provider, destination_dir)


def main(provider_name: str, destination: Path):
    install_dependencies()
    if provider_name.lower() == "all":
        for provider in list_all_providers(deny_list=["sample"]):
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
