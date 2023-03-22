/*!
# Fiberplane PDK macros

Macro crate to support the [Fiberplane PDK](https://docs.rs/fiberplane-pdk).

*/

mod casing;
mod config_schema;
mod field_attrs;
mod ident_or_string;
mod provider_data;
mod query_schema;
mod query_types;
mod schema_field;
mod schema_generator;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

/// Used to define the query types supported by the provider.
///
/// For every query type that is supported, the following properties may be
/// defined:
///
/// * **handler** - This is the function that will be called for handling
///   requests of the given type. Handlers may have one or two arguments, the
///   types of which are given between parentheses. The first argument is either
///   `ProviderRequest`, in which case the entire request is passed through
///   without processing, or the type of the query data. The second argument,
///   which is optional, specifies a type for the provider's config. Add a
///   `.await` call to the handler if the handler is asynchronous. Providing a
///   handler is mandatory.
/// * **label** - A label that is used when presenting the query type to the
///   user. Some query types are not intended to be user-selected (such as the
///   `status` query type) in which case they should not define a label.
/// * **supported_mime_types** - A list of MIME types that is supported by the
///   provider for data produced when running queries of the given type. This
///   should at least include the MIME type of the `Blob`s returned by
///   `handler()`. If the provider implements the `extract_data()` function this
///   may include other MIME types as well, in which case `extract_data()` has
///   the responsibility of converting from the data format returned by
///   `handler()` to the requested MIME type.
///
/// This macro generates the `invoke2()` and `get_supported_query_types()`
/// functions for you.
///
/// # Example
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// pdk_query_types! {
///     TIMESERIES_QUERY_TYPE => {
///         label: "Timeseries query",
///         handler: query_timeseries(ExampleQueryData, ExampleConfig),
///         supported_mime_types: [TIMESERIES_MIME_TYPE],
///     },
///     STATUS_QUERY_TYPE => {
///         handler: query_status(ProviderRequest).await,
///         supported_mime_types: [STATUS_MIME_TYPE],
///     },
///     "x-custom-query-type" => {
///         label: "My custom query",
///         handler: query_custom(ExampleQueryData).await,
///         supported_mime_types: ["application/vnd.fiberplane.provider.my-provider.custom-data"],
///     }
/// }
///
/// fn query_timeseries(query_data: ExampleQueryData, config: ExampleConfig) -> Result<Blob> {
///     todo!("Implement timeseries query handling")
/// }
///
/// async fn query_status(request: ProviderRequest) -> Result<Blob> {
///     todo!("Implement status checking")
/// }
///
/// async fn query_custom(query_data: ExampleQueryData) -> Result<Blob> {
///     todo!("Implement custom query handling")
/// }
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn pdk_query_types(input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    query_types::define_query_types(input)
}

/// Used to automatically generate a config schema for a given struct.
///
/// The macro extends the struct to which it is applied with a static `parse()`
/// method and it will automatically implement the provider's
/// `get_config_schema()` function for you.
///
/// `parse()` takes an untyped `ProviderConfig` object and parses it into an
/// instance of the struct.
///
/// # Example
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// #[derive(ConfigSchema, Deserialize)]
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
#[proc_macro_derive(ConfigSchema, attributes(pdk, serde))]
#[proc_macro_error]
pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    config_schema::derive_config_schema(input)
}

/// Used to automatically generate conversion methods to convert your data to
/// and from the `Blob` type.
///
/// The macro extends the struct to which it is applied with `parse()` and
/// `serialize()` methods.
///
/// # Example
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// #[derive(ProviderData, Serialize, Deserialize)]
/// #[pdk(mime_type = MY_MIME_TYPE]
/// struct MyData {
///     // specify your fields here...
/// }
/// ```
#[proc_macro_derive(ProviderData, attributes(pdk))]
#[proc_macro_error]
pub fn derive_provider_data(input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    provider_data::derive_provider_data(input)
}

/// Used to automatically generate a query schema for a given struct.
///
/// The macro extends the struct to which it is applied with static `schema()`
/// and `parse()` methods.
///
/// `schema()` will return the generated query schema, while `parse()` will take
/// form-encoded query data and parse it into an instance of the struct.
///
/// # Example
///
/// Note: This examples shows how to use the generated `schema()` and `parse()`
///       methods directly. Using the `pdk_query_types!` macro, this example
///       can be simplified even further.
///
/// ```no_compile
/// use fiberplane_pdk::prelude::*;
///
/// #[derive(Deserialize, QuerySchema)]
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
#[proc_macro_derive(QuerySchema, attributes(pdk, serde))]
#[proc_macro_error]
pub fn derive_query_schema(input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    query_schema::derive_query_schema(input)
}

/// Exports a provider function to make it available to the provider runtime.
///
/// # Example
///
/// Example usage of implementing the `invoke2()` function:
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
