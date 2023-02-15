# Providers

This repository contains our Fiberplane Providers and the PDK (Provider
Development Kit).

We encourage our community to contribute providers. Please see
[CONTRIBUTING.md](CONTRIBUTING.md) for more information.

## Overview

Fiberplane Providers are full-stack WASM plugins for fetching data from
arbitrary data sources. Providers built by Fiberplane, as well as those
contributed by our community, can be found in the [`providers/`](providers/)
directory.

The PDK is a development kit for creating providers. It consists of a Rust crate
that you can link against when creating your own provider. All the providers in
this repository also make us of it. Please see the
[`fiberplane-pdk/`](fiberplane-pdk/) directory for more details.

To learn how to create your own provider, please follow the
[tutorial](https://docs.fiberplane.com/docs/create-a-provider) or have a look at
the [sample provider](providers/sample).

There is also a
[Provider Protocol Reference](https://docs.fiberplane.com/reference/provider-protocol-reference).

## Getting Help

Please see
[COMMUNITY.md](https://github.com/fiberplane/fiberplane/blob/main/COMMUNITY.md)
for ways to reach out to us.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Code of Conduct

See
[CODE_OF_CONDUCT.md](https://github.com/fiberplane/fiberplane/blob/main/CODE_OF_CONDUCT.md).

## License

Our providers and the PDK are distributed under the terms of both the MIT
license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
