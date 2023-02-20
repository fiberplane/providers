<!-- The following is generated by cargo-rdme from lib.rs, and should not be modified manually-->
<!-- cargo-rdme start -->

# Fiberplane Provider Development Kit

This crate provides high-level macros and abstractions for more ergonomic
provider development.

Follow the tutorial: [How to create a provider](https://docs.fiberplane.com/docs/create-a-provider)

## Overview

The main traits and macros you will interact with from this crate are:

* [`pdk_query_types!`](https://docs.rs/fiberplane-pdk-macros/latest/fiberplane_pdk_macros/macro.pdk_query_types.html) -
  Macro for defining your query types and their handlers.
* [`pdk_export!`](https://docs.rs/fiberplane-pdk-macros/latest/fiberplane_pdk_macros/attr.pdk_export.html) -
  Macro to export functions as part of the provider protocol.
* [`ConfigSchema`](https://docs.rs/fiberplane-pdk-macros/latest/fiberplane_pdk_macros/derive.ConfigSchema.html) -
  Macro for deriving a schema from your config struct.
* [`QuerySchema`](https://docs.rs/fiberplane-pdk-macros/latest/fiberplane_pdk_macros/derive.QuerySchema.html) -
  Macro for deriving a schema from your query struct(s).
* [`ProviderData`](https://docs.rs/fiberplane-pdk/latest/fiberplane_pdk/provider_data/trait.ProviderData.html) -
  A trait and derive macro for types that you  wish to pass around as provider data using
  [`Blob`s](https://docs.rs/fiberplane-models/latest/fiberplane_models/blobs/struct.Blob.html).

<!-- cargo-rdme end -->