# Changelog

All notable changes to this project will be documented in this file.

The format of this file is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Please note that while we use [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
for the `providers` repository as a whole, individual crates published from this
repository may skip versions to stay in lockstep with the other crates. This
means that individual crates do not strictly follow _SemVer_ although their
versioning remains _compatible with_ SemVer, i.e. they will not contain breaking
changes if the major version hasn't changed.

## [Unreleased]

### Added

- Added provider for Parseable.

## [1.0.0-beta.3] - 2023-06-14

- Prometheus provider: decrease step size in order to add more details to lines in graphs.

### Added
- Added provider for Parseable.
- Added support for `ArrayField` schema specification. As long as `T` has a
  `QuerySchema` derive, you can use `Vec<T>` in structs that derive
  `QuerySchema`. Serialization to URL-encoded query uses the ["bracket"
  notation](https://docs.rs/serde-querystring/0.2.1/serde_querystring/index.html#brackets-mode).
  (#31)

### Changed

- The Elasticsearch and Loki providers have been upgraded to the latest provider
  protocol.
- Use less confusing placeholders for the configuration schema of the Cloudwatch
  provider.
- Rename Event in the providers module to ProviderEvent (#28)
- Providers set up their own panic handlers to provide better information in
  case of panics (instead of letting WASM runtime default panic handling
  setup) (#35)

### Fixed

- Fixed required fields in schemas generated using the `QuerySchema` macro.
- Fixed support for the `checked_by_default` and `supports_suggestions`
  annotations in the `ConfigSchema` and `QuerySchema` macros.
- Fixed lacking `create_cells()` implementation for Elasticsearch and Loki providers

## [1.0.0-beta.1] - 2023-02-14

### Added

- Initial release of the Fiberplane Providers and PDK.
- Amazon Web Services (AWS) Cloudwatch provider.
