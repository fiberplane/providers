#![allow(dead_code)]

mod error;
mod http;
mod legacy;
mod request;
mod schema;

pub use error::*;
pub use fiberplane::protocols::blobs::Blob;
use fp_bindgen::prelude::Serializable;
pub use http::*;
pub use legacy::{LegacyProviderRequest, LegacyProviderResponse};
pub use request::*;
pub use rmpv::Value;
pub use schema::*;

/// Defines which query types are supported by a provider.
#[derive(Debug, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct SupportedQueryType {
    /// The query type supported by the provider.
    ///
    /// There are predefined query types, such as "table" and "log", but
    /// providers may also implement custom query types, which it should prefix
    /// with "x-".
    query_type: String,

    /// The query schema defining the format of the `query_data` to be submitted
    /// with queries of this type.
    schema: QuerySchema,

    /// MIME types supported for extraction. Any MIME type specified here should
    /// be valid as an argument to `extract_data()` when passed a response from
    /// queries of this type.
    ///
    /// E.g.:
    /// ```
    /// vec![
    ///     "application/vnd.fiberplane.events",
    ///     "application/vnd.fiberplane.metrics"
    /// ];
    /// ```
    mime_types: Vec<String>,
}

/// Timestamp specified in seconds since the UNIX epoch, with subsecond
/// precision.
pub type Timestamp = f64;
