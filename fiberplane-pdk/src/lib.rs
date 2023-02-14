/*!
# Fiberplane Provider Development Kit

This crate provides high-level macros and abstractions for more ergonomic
provider development.

Follow the tutorial: [How to create a provider](https://docs.fiberplane.com/docs/tutorial-create-a-provider)

## Overview

The main traits and macros you will interact with from this crate are:

* [`pdk_query_types!`][pdk_query_types] - A macro for defining your query types
  and their handlers.
* [`pdk_export!`][pdk_export] - A macro to export functions as part of the
  provider protocol.
* [`ConfigSchema`][ConfigSchema] - A macro for deriving a schema from your
  config struct.
* [`QuerySchema`][QuerySchema] - A macro for deriving a schema from your query
  struct(s).
* [`ProviderData`][ProviderData] - A trait and derive macro for types that you
  wish to pass around as provider data using [`Blob`s][Blob].

*/

mod parse_query;
pub mod prelude;
pub mod provider_data;
mod types;

// Fiberplane-specific re-exports.
pub use fiberplane_models::providers;
pub use fiberplane_pdk_macros as macros;
pub use fiberplane_provider_bindings as bindings;
pub use fp_bindgen::prelude::fp_export_impl;
pub use parse_query::parse_query;

// Re-exported third-party crates. Provider authors may use these instead of
// depending on them directly, so they always use the same version as the PDK.
pub use serde_json;
