use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A data-source represents all the configuration for a specific component or
/// service. It will be used by provider.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde()]
pub enum DataSource {
    Prometheus(PrometheusDataSource),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FetchError {
    #[serde(rename_all = "camelCase")]
    RequestError { payload: RequestError },
    #[serde(rename_all = "camelCase")]
    DataError { message: String },
    #[serde(rename_all = "camelCase")]
    Other { message: String },
}

/// A single data point in time, with meta-data about the metric it was taken
/// from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instant {
    pub metric: Metric,
    pub point: Point,
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

/// A data-source for Prometheus. Currently only requires a url. This should be
/// a full URL starting with http:// or https:// the domain, and optionally a
/// port and a path.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusDataSource {
    pub url: String,
}

/// Options to specify which instant should be fetched.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInstantOptions {
    pub data_source: DataSource,
    pub time: Timestamp,
}

/// Options to specify what series should be fetched.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySeriesOptions {
    pub data_source: DataSource,
    pub time_range: TimeRange,
}

/// HTTP request options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub url: String,
    pub method: RequestMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", with = "serde_bytes")]
    pub body: Option<Vec<u8>>,
}

/// Possible errors that may happen during an HTTP request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    #[serde(rename_all = "camelCase")]
    ServerError {
        status_code: u16,
        #[serde(with = "serde_bytes")]
        response: Vec<u8>,
    },
    #[serde(rename_all = "camelCase")]
    Other { reason: String },
}

/// HTTP request method.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestMethod {
    Delete,
    Get,
    Head,
    Post,
}

/// Response to an HTTP request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}

/// A series of data-points in time, with meta-data about the metric it was
/// taken from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub metric: Metric,
    pub points: Vec<Point>,
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
