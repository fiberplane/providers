#!/usr/bin/env python3

import argparse
import subprocess
import sys
from typing import List
import json

# Using urllib instead of requests to _only_ require
# Python 3 stdlib to work
import urllib.request


ALL_CRATES_IN_ORDER = ["fiberplane-pdk-macros", "fiberplane-pdk"]
ALL_CRATES = "all"
CRATES_IO = "crates-io"


def index_url_path(crate: str) -> str:
    if len(crate) == 1:
        return f"1/{crate}"
    elif len(crate) == 2:
        return f"2/{crate}"
    elif len(crate) == 3:
        return f"3/{crate[0]}/{crate}"
    else:
        return f"{crate[0:2]}/{crate[2:4]}/{crate}"


def crates_io_published_versions(crate: str) -> List[str]:
    """
    Fetches the current list of all published versions of a crate on crates-io.

    Notably, this _includes_ the yanked versions
    """
    index_url = f"https://index.crates.io/{index_url_path(crate)}"
    print(f"Requesting {index_url}")
    request = urllib.request.Request(
        index_url, headers={"User-Agent": "Fiberplane/Release worker/1.0"}
    )
    with urllib.request.urlopen(request) as response:
        # We ignore anything that comes after the first newline
        data = json.loads(response.read().decode("utf-8").split("\n", 1)[0])
        # The response can be either a single json object or an array of json object
        if isinstance(data, dict):
            return [data["vers"]]
        else:
            return [published["vers"] for published in data]


def publish(crate: str, version: str, registry: str):
    """
    Publish a crate on crates-io.

    You must be already logged in in order to have this function working.
    """
    print(f"Publishing {crate} crate on {registry} in version {version}...", end=" ")
    try:
        subprocess.check_output(
            f"cargo publish --registry {registry} -p {crate}",
            stderr=subprocess.STDOUT,
            shell=True,
        )
    except subprocess.CalledProcessError as e:
        print(f"Error during publication:\n{e.output}")
        sys.exit(1)
    print("OK!")


def is_version_published_on_crates_io(crate: str, version: str) -> bool:
    """
    Return true if VERSION of CRATE is already on crates-io.
    """
    published_versions = crates_io_published_versions(crate)
    print(f"Published versions are {published_versions}")
    return version in published_versions


def current_version(crate: str) -> str:
    """
    Return the current version of CRATE, according to Cargo reading
    its own metadata.
    """
    try:
        cargo_metadata = json.loads(
            subprocess.check_output(
                f"cargo metadata --format-version 1 --no-deps",
                shell=True,
            )
        )
        current_crate_meta = next(
            meta for meta in cargo_metadata["packages"] if meta["name"] == crate
        )
        return current_crate_meta["version"]
    except subprocess.CalledProcessError as e:
        print(f"Cargo error while fetching the current version:\n{e.output}")
        sys.exit(1)


def handle_single_crate_with_crates_io(crate: str):
    crate_version = current_version(crate)
    if not is_version_published_on_crates_io(crate, crate_version):
        publish(crate, crate_version, CRATES_IO)


def main(crate: str, registry: str):
    if registry != CRATES_IO:
        print("Only crates-io registry is supported now")
        sys.exit(1)

    if crate == ALL_CRATES:
        for crate in ALL_CRATES_IN_ORDER:
            handle_single_crate_with_crates_io(crate)
        sys.exit(0)

    handle_single_crate_with_crates_io(crate)
    sys.exit(0)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="publish_crates",
        description="Publish a crate to the given registry",
        epilog="Created by Fiberplane <info@fiberplane.com>",
    )
    parser.add_argument(
        "-p",
        "--package",
        default=ALL_CRATES,
        help=f"Name of the package to publish, omit or use '{ALL_CRATES}' to do all packages",
    )
    parser.add_argument(
        "-r",
        "--registry",
        help=f"Registry to publish to, defaults to '{CRATES_IO}'",
        default=CRATES_IO,
    )

    args = parser.parse_args()
    main(args.package, args.registry)
