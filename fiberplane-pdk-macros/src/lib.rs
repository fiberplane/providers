mod config_schema;
mod field_attrs;
mod query_schema;
mod schema_field;
mod schema_generator;
mod schema_struct;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

/// Used to automatically generate a config schema for a given struct.
///
/// The macro extends the struct to which it is applied with a static `parse()`
/// method and it will automatically implement the provider's
/// `get_config_schema()` function for you.
///
/// `parse()` takes an untyped `ProviderConfig` object and parses it into an
/// instance of the struct.
///
/// Example:
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// #[derive(ConfigSchema)]
/// struct MyConfig {
///     #[pdk(label = "Specify your endpoint")]
///     pub endpoint: String,
/// }
///
/// #[pdk_export]
/// async fn invoke2(request: ProviderRequest) -> Result<Blob, Error> {
///     let config = MyConfig::parse(request.config)?;
///     todo!("Do something with the request");
/// }
/// ```
#[proc_macro_derive(ConfigSchema, attributes(pdk))]
#[proc_macro_error]
pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    config_schema::derive_config_schema(input)
}

/// Used to automatically generate a query schema for a given struct.
///
/// The macro extends the struct to which it is applied with static `schema()`
/// and `parse()` methods.
///
/// `schema()` will return the generated query schema, while `parse()` will take
/// form-encoded query data and parse it into an instance of the struct.
///
/// Example:
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// #[derive(QuerySchema)]
/// struct MyQueryData {
///     #[pdk(label = "Enter your query")]
///     pub query: String,
/// }
///
/// #[pdk_export]
/// async fn get_supported_query_types(_config: ProviderConfig) -> Vec<SupportedQueryType> {
///     vec![
///         SupportedQueryType::new(MY_QUERY_TYPE)
///             .with_schema(MyQueryData::schema())
///             .supporting_mime_types(&[MY_MIME_TYPE]),
///     ]
/// }
///
/// #[pdk_export]
/// async fn invoke2(request: ProviderRequest) -> Result<Blob, Error> {
///     match request.query_type.as_str() {
///         MY_QUERY_TYPE => {
///             let data = MyQueryData::parse(request.query_data)?;
///             todo!("Do something with data...")
///         }
///         _ => Err(Error::UnsupportedRequest),
///     }
/// }
/// ```
#[proc_macro_derive(QuerySchema, attributes(pdk))]
#[proc_macro_error]
pub fn derive_query_schema(input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    query_schema::derive_query_schema(input)
}

/// Exports a provider function to make it available to the provider runtime.
///
/// Example usage of implementing the `invoke2` function:
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// #[pdk_export]
/// async fn invoke2(request: ProviderRequest) -> Result<Blob, Error> {
///     todo!("Your code goes here...")
/// }
/// ```
#[proc_macro_attribute]
pub fn pdk_export(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    let ts: proc_macro2::TokenStream = input.into();
    (quote! {
        #[fiberplane_pdk::fp_export_impl(fiberplane_pdk::bindings)]
        #ts
    })
    .into()
}