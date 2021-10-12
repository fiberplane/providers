use fp_bindgen::prelude::*;
use std::collections::HashMap;

/// A data-source represents all the configuration for a specific component or
/// service. It will be used by provider.
#[derive(Serializable)]
#[allow(dead_code)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum DataSource {
    Prometheus(PrometheusDataSource),
    Proxy(ProxyDataSource),
    // Elasticsearch
    // Kubernetes
}

/// A data-source for Prometheus. Currently only requires a url. This should be
/// a full URL starting with http:// or https:// the domain, and optionally a
/// port and a path.
#[derive(Serializable)]
pub struct PrometheusDataSource {
    pub url: String,
}

/// Relays requests for a data-source to a proxy server registered with the API.
#[derive(Serializable)]
pub struct ProxyDataSource {
    /// ID of the proxy as known by the API.
    pub proxy_id: String,

    /// Name of the data source exposed by the proxy.
    pub data_source_name: String,
}

/// Options to specify which instant should be fetched.
#[derive(Serializable)]
pub struct QueryInstantOptions {
    pub data_source: DataSource,
    pub time: Timestamp,
}

/// Options to specify what series should be fetched.
#[derive(Serializable)]
pub struct QuerySeriesOptions {
    pub data_source: DataSource,
    pub time_range: TimeRange,
}

/// A range in time from a given timestamp (inclusive) up to another timestamp
/// (exclusive).
#[derive(Serializable)]
pub struct TimeRange {
    pub from: Timestamp,
    pub to: Timestamp,
}

/// Timestamp specified in seconds since the UNIX epoch, with subsecond
/// precision.
pub type Timestamp = f64;

/// A single data-point in time.
#[derive(Serializable)]
pub struct Point {
    pub timestamp: Timestamp,
    pub value: f64,
}

/// A single data point in time, with meta-data about the metric it was taken
/// from.
#[derive(Serializable)]
pub struct Instant {
    pub metric: Metric,
    pub point: Point,
}

/// A series of data-points in time, with meta-data about the metric it was
/// taken from.
#[derive(Serializable)]
pub struct Series {
    pub metric: Metric,
    pub points: Vec<Point>,
}

/// Meta-data about a metric.
#[derive(Serializable)]
pub struct Metric {
    pub name: String,
    pub labels: HashMap<String, String>,
}
