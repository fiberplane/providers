use super::Timestamp;
use fp_bindgen::prelude::Serializable;
use serde_bytes::ByteBuf;

#[non_exhaustive]
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ProviderRequest {
    // Note that enum variants must be structs because
    // we are using serde's internally tagged representation
    Instant(QueryInstant),
    Series(QueryTimeRange),
    Proxy(ProxyRequest),
    /// Requests a list of auto-suggestions. Note that these are
    /// context-unaware.
    AutoSuggest,
}

#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct QueryInstant {
    pub query: String,
    pub timestamp: Timestamp,
}

#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct QueryTimeRange {
    pub query: String,
    pub time_range: TimeRange,
}

/// Relays requests for a data-source to a proxy server registered with the API.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct ProxyRequest {
    /// ID of the proxy as known by the API.
    pub proxy_id: String,

    /// Name of the data source exposed by the proxy.
    pub data_source_name: String,

    /// Request data to send to the proxy
    pub request: ByteBuf,
}

/// A range in time from a given timestamp (inclusive) up to another timestamp
/// (exclusive).
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct TimeRange {
    pub from: Timestamp,
    pub to: Timestamp,
}
