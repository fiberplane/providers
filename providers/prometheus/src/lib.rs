mod auto_suggest;
mod constants;
mod prometheus;
mod timeseries;

use auto_suggest::query_suggestions;
use constants::*;
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde_json::Value;
use std::env;
use timeseries::{create_graph_cell, query_series};

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[pdk_export]
async fn get_supported_query_types(_config: ProviderConfig) -> Vec<SupportedQueryType> {
    vec![
        SupportedQueryType::new(TIMESERIES_QUERY_TYPE)
            .with_schema(vec![
                TextField::new()
                    .with_name(QUERY_PARAM_NAME)
                    .with_label("Enter your Prometheus query")
                    .required()
                    .with_suggestions()
                    .into(),
                DateTimeRangeField::new()
                    .with_name(TIME_RANGE_PARAM_NAME)
                    .with_label("Specify a time range")
                    .required()
                    .into(),
                CheckboxField::new()
                    .with_name(LIVE_PARAM_NAME)
                    .with_label("Enable live mode")
                    .with_value("true")
                    .into(),
            ])
            .supporting_mime_types(&[TIMESERIES_MIME_TYPE]),
        SupportedQueryType::new(SUGGESTIONS_QUERY_TYPE)
            .supporting_mime_types(&[SUGGESTIONS_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob> {
    log(format!(
        "Prometheus provider (commit: {}, built at: {}) invoked for query type \"{}\" and query data \"{:?}\"",
        COMMIT_HASH, BUILD_TIMESTAMP, request.query_type, request.query_data
    ));

    let config: Config = serde_json::from_value(request.config).map_err(|err| Error::Config {
        message: format!("Error parsing config: {err:?}"),
    })?;

    match request.query_type.as_str() {
        TIMESERIES_QUERY_TYPE => query_series(request.query_data, config).await,
        SUGGESTIONS_QUERY_TYPE => query_suggestions(request.query_data, config).await,
        STATUS_QUERY_TYPE => check_status(config).await,
        _ => Err(Error::UnsupportedRequest),
    }
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>> {
    log(format!("Creating cells for query type: {query_type}"));

    match query_type.as_str() {
        TIMESERIES_QUERY_TYPE => create_graph_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
}

async fn check_status(config: Config) -> Result<Blob> {
    // Send a fake query to the query endpoint to check if we can connect to the Prometheus
    // instance. We should get a 200 response even though it won't return any data.
    query_direct_and_proxied::<Value>(
        &config,
        "prometheus",
        "api/v1/query?query=fiberplane_check_status",
        None,
    )
    .await?;

    Ok(Blob::builder()
        .mime_type(STATUS_MIME_TYPE.to_owned())
        .data("ok")
        .build())
}
