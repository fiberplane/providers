use super::{HttpRequestError, Timestamp};
use fp_bindgen::prelude::Serializable;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

#[non_exhaustive]
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ProviderResponse {
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
        log_records: Vec<LogRecord>,
    },
    StatusOk,
}

// TODO derive Error trait
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Error {
    UnsupportedRequest,
    #[fp(rename_all = "camelCase")]
    Http {
        error: HttpRequestError,
    },
    #[fp(rename_all = "camelCase")]
    Data {
        message: String,
    },
    #[fp(rename_all = "camelCase")]
    Deserialization {
        message: String,
    },
    #[fp(rename_all = "camelCase")]
    Config {
        message: String,
    },
    #[fp(rename_all = "camelCase")]
    Other {
        message: String,
    },
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
    pub text: String,

    /// Optional description to go along with this suggestion.
    pub description: Option<String>,
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
