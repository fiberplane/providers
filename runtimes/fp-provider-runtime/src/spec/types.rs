use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    UnsupportedRequest,
    #[serde(rename_all = "camelCase")]
    Http { error: HttpRequestError },
    #[serde(rename_all = "camelCase")]
    Data { message: String },
    #[serde(rename_all = "camelCase")]
    Deserialization { message: String },
    #[serde(rename_all = "camelCase")]
    Config { message: String },
    #[serde(rename_all = "camelCase")]
    Other { message: String },
}

/// HTTP request options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpRequestMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_bytes::ByteBuf>,
}

/// Possible errors that may happen during an HTTP request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HttpRequestError {
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    #[serde(rename_all = "camelCase")]
    ServerError { status_code: u16, response: serde_bytes::ByteBuf },
    #[serde(rename_all = "camelCase")]
    Other { reason: String },
}

/// HTTP request method.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpRequestMethod {
    Delete,
    Get,
    Head,
    Post,
}

/// Response to an HTTP request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    pub body: serde_bytes::ByteBuf,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}

/// A single data point in time, with meta-data about the metric it was taken
/// from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instant {
    pub metric: Metric,
    pub point: Point,
}

/// An individual log record
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRecord {
    pub timestamp: Timestamp,
    pub body: String,
    pub attributes: HashMap<String, String>,
    pub resource: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<serde_bytes::ByteBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<serde_bytes::ByteBuf>,
}

/// Meta-data about a metric.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub name: String,
    pub labels: HashMap<String, String>,
}

/// A single data-point in time.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub timestamp: Timestamp,
    pub value: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderRequest {
    Instant(QueryInstant),
    Series(QueryTimeRange),
    Proxy(ProxyRequest),
    /// Requests a list of auto-suggestions. Note that these are
    /// context-unaware.
    AutoSuggest,
    Logs(QueryLogs),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderResponse {
    #[serde(rename_all = "camelCase")]
    Error { error: Error },
    #[serde(rename_all = "camelCase")]
    Instant { instants: Vec<Instant> },
    #[serde(rename_all = "camelCase")]
    Series { series: Vec<Series> },
    #[serde(rename_all = "camelCase")]
    AutoSuggest { suggestions: Vec<Suggestion> },
    #[serde(rename_all = "camelCase")]
    LogRecords { log_records: Vec<LogRecord> },
}

/// Relays requests for a data-source to a proxy server registered with the API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyRequest {
    /// ID of the proxy as known by the API.
    pub proxy_id: String,

    /// Name of the data source exposed by the proxy.
    pub data_source_name: String,

    /// Request data to send to the proxy
    pub request: serde_bytes::ByteBuf,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInstant {
    pub query: String,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryLogs {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    pub time_range: TimeRange,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTimeRange {
    pub query: String,
    pub time_range: TimeRange,
}

/// A series of data-points in time, with meta-data about the metric it was
/// taken from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub metric: Metric,
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// Suggested text.
    pub text: String,

    /// Optional description to go along with this suggestion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A range in time from a given timestamp (inclusive) up to another timestamp
/// (exclusive).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub from: Timestamp,
    pub to: Timestamp,
}

pub type Timestamp = f64;
