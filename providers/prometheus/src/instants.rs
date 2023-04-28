use super::{constants::*, prometheus::*};
use fiberplane_models::blobs::Blob;
use fiberplane_models::providers::Metric;
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, ProviderData, Serialize)]
#[pdk(mime_type = INSTANTS_MIME_TYPE)]
pub struct Instants(pub Vec<Instant>);

/// A single data-point in time, with meta-data about the metric it was taken from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instant {
    pub name: String,
    pub labels: BTreeMap<String, String>,
    pub metric: Metric,
}

pub async fn query_instants(request: ProviderRequest) -> Result<Blob> {
    let response: PrometheusResponse = query_direct_and_proxied(
        &Config::parse(request.config)?,
        "prometheus",
        "api/v1/query",
        Some(request.query_data),
    )
    .await?;

    let PrometheusData::Vector(instants) = response.data else {
        return Err(Error::Data {
            message: "Expected a vector of instants".to_string(),
        })
    };

    instants
        .into_iter()
        .map(InstantVector::into_instant)
        .collect::<Result<Vec<_>>>()
        .and_then(|instants| Instants(instants).to_blob())
}
