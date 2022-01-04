use super::{HttpRequestError, Timestamp};
use fp_bindgen::prelude::Serializable;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

#[non_exhaustive]
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ProviderResponse {
    // Note that enum variants must be structs because
    // we are using serde's internally tagged representation
    Error { error: Error },
    Instant { instants: Vec<Instant> },
    Series { series: Vec<Series> },
    AutoSuggest { suggestions: Vec<Suggestion> },
    LogRecords { log_records: Vec<LogRecord> },
}

// TODO derive Error trait
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Error {
    // Note that enum variants must be structs because
    // we are using serde's internally tagged representation
    UnsupportedRequest,
    Http { error: HttpRequestError },
    Data { message: String },
    Deserialization { message: String },
    Config { message: String },
    Other { message: String },
}

/// A single data-point in time.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct Point {
    pub timestamp: Timestamp,
    pub value: f64,
}

/// A single data point in time, with meta-data about the metric it was taken
/// from.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct Instant {
    pub metric: Metric,
    pub point: Point,
}

/// A series of data-points in time, with meta-data about the metric it was
/// taken from.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct Series {
    pub metric: Metric,
    pub points: Vec<Point>,
}

/// Meta-data about a metric.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct Metric {
    pub name: String,
    pub labels: HashMap<String, String>,
}

#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct Suggestion {
    /// Suggested text.
    text: String,

    /// Optional description to go along with this suggestion.
    description: Option<String>,
}

/// An individual log record
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct LogRecord {
    pub timestamp: Timestamp,
    pub body: String,
    pub attributes: HashMap<String, String>,
    pub resource: HashMap<String, String>,
    // TODO these should really be [u8; 16], but arrays are
    // not currently supported by fp-bindgen
    pub trace_id: Option<ByteBuf>,
    pub span_id: Option<ByteBuf>,
}
