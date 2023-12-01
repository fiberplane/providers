use super::{constants::*, prometheus::*};
use fiberplane_models::autometrics::PrometheusResponse;
use fiberplane_models::blobs::Blob;
use fiberplane_models::providers::Metric;
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, QuerySchema)]
pub(crate) struct InstantsQuery {
    #[pdk(label = "Enter your Prometheus query", supports_suggestions)]
    query: String,

    #[pdk(label = "Specify a time")]
    time: Option<Timestamp>,
}

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

pub async fn query_instants(query: InstantsQuery, config: Config) -> Result<Blob> {
    let body = Blob::from({
        let mut form_data = form_urlencoded::Serializer::new(String::new());
        form_data.append_pair("query", &query.query);
        form_data.append_pair(
            "time",
            &query.time.unwrap_or_else(Timestamp::now_utc).to_string(),
        );
        form_data
    });

    let response: PrometheusResponse<Vec<InstantVector>> =
        query_direct_and_proxied(&config, "prometheus", "api/v1/query", Some(body)).await?;

    let instants = response
        .data
        .into_iter()
        .map(InstantVector::into_instant)
        .collect::<Result<Vec<_>>>()?;

    Instants(instants).to_blob()
}
