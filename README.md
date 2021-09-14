# Providers

This repository contains the SDK you need to create _providers_. Providers are
plugins used to fetch data from arbitrary data sources.

In addition, you will find Fiberplane's first-party providers that you can build
yourself and/or use for inspiration when creating your own provider, as well as
the _protocol_ that defines the API between providers and host environments.

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

## Protocol

The protocol defines the API between providers and host environments and can be
found in the `protocol/` directory. Running `cargo run` will regenerate the
bindings used by the SDK library.

Developers that wish to build their own provider runtimes can use
`cargo run -- --runtime` to generate the runtime bindings as well.
