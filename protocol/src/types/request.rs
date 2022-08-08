use fiberplane::protocols::blobs::Blob;
use fp_bindgen::prelude::Serializable;
use rmpv::Value;

#[non_exhaustive]
#[derive(Debug, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct ProviderRequest {
    /// Query type that is part of the
    /// [Intent](https://www.notion.so/fiberplane/RFC-45-Provider-Protocol-2-0-Revised-4ec85a0233924b2db0010d8cdae75e16#c8ed5dfbfd764e6bbd5c5b79333f9d6e)
    /// through which the provider is invoked.
    pub query_type: String,

    /// Query data.
    ///
    /// This is usually populated from the [ProviderCell::query_data] field,
    /// meaning the MIME type will be `"application/x-www-form-urlencoded"`
    /// when produced by Studio's Query Builder.
    pub query_data: Blob,

    /// Configuration for the data source.
    pub config: Value,

    /// Optional response from a previous invocation.
    /// May be used for implementing things like filtering without additional
    /// server roundtrip.
    #[fp(default, skip_serializing_if = "Option::is_none")]
    pub previous_response: Option<Blob>,
}
