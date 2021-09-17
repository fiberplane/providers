# Providers

This repository contains the SDK you need to create _providers_. Providers are
plugins used to fetch data from arbitrary data sources.

In addition, you will find Fiberplane's first-party providers that you can build
yourself and/or use for inspiration when creating your own provider, as well as
the _protocol_ that defines the API between providers and host environments.

Finally, you will find the runtimes for providers in the `runtimes/` directory.

## SDK

In order to create your own provider, you should link to the `fp-provider`
crate:

`Cargo.toml`:

```toml
[dependencies]
fp-provider = "1.0"
```

The sources for this crate are found in the `fp-provider/` directory, though
they are auto-generated from the protocol (see below).

## First-party providers

Fiberplane offers several providers out of the box:

- **Prometheus**. This provider can fetch instants and series from a Prometheus
  server. Its implementation can be found in the `providers/prometheus/`
  directory.
- **Proxy**. The proxy provider redirects data requests to a proxy server,
  which in turn delegates the request to another provider. Its implementation
  can be found in the `providers/proxy/` directory.

## Runtimes

The runtimes in which providers can be executed can be found in the `runtimes/`
directory. The Wasmer runtime you can find there is also used as a dependency
for our open-source [Proxy server](https://github.com/fiberplane/proxy).

Please note that most of the code for the runtimes is again generated from the
protocol (see below).

## Protocol

The protocol that defines the API between providers and runtimes can be found in
the `protocol/` directory. Running `cargo run` in that directory will regenerate
the bindings used by the SDK library, as well as those for the runtimes.

## License

This project is licensed under Apache License, version 2.0
([LICENSE.txt](https://github.com/fiberplane/fp-bindgen/blob/main/LICENSE.txt)).
