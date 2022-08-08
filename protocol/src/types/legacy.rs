use super::{error::Error, Timestamp};
use fiberplane::protocols::core::{Instant, Series};
use fp_bindgen::prelude::Serializable;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

/// Legacy `ProviderRequest` from the Provider 1.0 protocol.
#[non_exhaustive]
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum LegacyProviderRequest {
    // Note that enum variants must be structs because
    // we are using serde's internally tagged representation
    Instant(QueryInstant),
    Series(QueryTimeRange),
    Proxy(ProxyRequest),
    /// Requests a list of auto-suggestions. Note that these are
    /// context-unaware.
    AutoSuggest,
    Logs(QueryLogs),
    /// Check data source status, any issue will be returned as `Error`
    Status,
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

#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct QueryLogs {
    pub query: String,
    pub limit: Option<u32>,
    pub time_range: TimeRange,
}

/// Legacy `ProviderResponse` from the 1.0 protocol.
#[non_exhaustive]
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum LegacyProviderResponse {
    #[fp(rename_all = "camelCase")]
    Error {
        error: Error,
    },
    #[fp(rename_all = "camelCase")]
    Instant {
        instants: Vec<Instant>,
    },
    #[fp(rename_all = "camelCase")]
    Series {
        series: Vec<Series>,
    },
    #[fp(rename_all = "camelCase")]
    AutoSuggest {
        suggestions: Vec<Suggestion>,
    },
    #[fp(rename_all = "camelCase")]
    LogRecords {
        log_records: Vec<LegacyLogRecord>,
    },
    StatusOk,
}

#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct Suggestion {
    /// Suggested text.
    pub text: String,

    /// Optional description to go along with this suggestion.
    pub description: Option<String>,
}

/// An individual log record
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct LegacyLogRecord {
    pub timestamp: Timestamp,
    pub body: String,
    pub attributes: HashMap<String, String>,
    pub resource: HashMap<String, String>,
    // TODO these should really be [u8; 16], but arrays are
    // not currently supported by fp-bindgen
    pub trace_id: Option<ByteBuf>,
    pub span_id: Option<ByteBuf>,
}
