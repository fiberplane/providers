use super::instants::Instant;
use fiberplane_models::MaybeSerializable;
use fiberplane_pdk::prelude::Timestamp;
use fiberplane_pdk::providers::*;
use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    num::ParseFloatError,
    time::{Duration, SystemTime},
};

#[derive(Clone, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "crate::prometheus")]
#[serde(tag = "resultType", content = "result", rename_all = "snake_case")]
pub enum PrometheusData {
    Vector(Vec<InstantVector>),
    Matrix(Vec<RangeVector>),
}

impl MaybeSerializable for PrometheusData {}

#[derive(Clone, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "crate::prometheus")]
pub struct InstantVector {
    pub metric: BTreeMap<String, String>,
    pub value: PrometheusPoint,
}

impl MaybeSerializable for InstantVector {}

impl InstantVector {
    pub fn into_instant(self) -> Result<Instant, Error> {
        let mut labels = self.metric;
        let name = labels.remove("__name__").unwrap_or_else(|| "".to_owned());
        let metric = self.value.to_metric()?;
        Ok(Instant {
            name,
            labels,
            metric,
        })
    }
}

#[derive(Deserialize)]
pub struct Metadata {
    pub help: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusMetadataResponse {
    pub data: BTreeMap<String, Vec<Metadata>>,
}

#[derive(Clone, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "crate::prometheus")]
pub struct PrometheusPoint(f64, String);

impl MaybeSerializable for PrometheusPoint {}

impl PrometheusPoint {
    pub fn to_metric(&self) -> Result<Metric, ParseFloatError> {
        let time = SystemTime::UNIX_EPOCH + Duration::from_millis((self.0 * 1000.0) as u64);
        Ok(Metric::builder()
            .time(Timestamp::from(time))
            .value(self.1.parse()?)
            .otel(OtelMetadata::default())
            .build())
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "crate::prometheus")]
pub struct RangeVector {
    pub metric: BTreeMap<String, String>,
    pub values: Vec<PrometheusPoint>,
}

impl MaybeSerializable for RangeVector {}

impl RangeVector {
    pub fn into_series(self) -> Result<Timeseries, Error> {
        let mut labels = self.metric;
        let name = labels.remove("__name__").unwrap_or_else(|| "".to_owned());
        let metrics = self
            .values
            .into_iter()
            .map(|value| value.to_metric())
            .collect::<Result<_, _>>()?;
        Ok(Timeseries::builder()
            .name(name)
            .labels(labels)
            .metrics(metrics)
            .otel(OtelMetadata::default())
            .visible(true)
            .build())
    }
}
