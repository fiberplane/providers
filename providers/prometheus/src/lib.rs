mod auto_suggest;
mod constants;
mod node_graph;
mod prometheus;
mod timeseries;

use auto_suggest::query_suggestions;
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use node_graph::{create_hierarchy_cell, node_graph, NodeGraphQuery};
use serde_json::Value;
use std::env;
use timeseries::{create_graph_cell, query_series, TimeseriesQuery};

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

pdk_query_types! {
    NODE_GRAPH_QUERY_TYPE => {
        handler: node_graph(NodeGraphQuery, Config).await,
        supported_mime_types: [NODE_GRAPH_MIME_TYPE]
    },
    TIMESERIES_QUERY_TYPE => {
        handler: query_series(TimeseriesQuery, Config).await,
        supported_mime_types: [TIMESERIES_MIME_TYPE]
    },
    STATUS_QUERY_TYPE => {
        handler: check_status(ProviderRequest).await,
        supported_mime_types: [STATUS_MIME_TYPE]
    },
    SUGGESTIONS_QUERY_TYPE => {
        handler: query_suggestions(AutoSuggestRequest, Config).await,
        supported_mime_types: [SUGGESTIONS_MIME_TYPE]
    }
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>> {
    log(format!("Creating cells for query type: {query_type}"));

    match query_type.as_str() {
        TIMESERIES_QUERY_TYPE => create_graph_cell(),
        NODE_GRAPH_QUERY_TYPE => create_hierarchy_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
}

async fn check_status(request: ProviderRequest) -> Result<Blob> {
    let config = Config::parse(request.config)?;

    // Send a fake query to the query endpoint to check if we can connect to the Prometheus
    // instance. We should get a 200 response even though it won't return any data.
    query_direct_and_proxied::<Value>(
        &config,
        "prometheus",
        "api/v1/query?query=fiberplane_check_status",
        None,
    )
    .await?;

    ProviderStatus::builder()
        .status(Ok(()))
        .version(COMMIT_HASH.to_owned())
        .built_at(BUILD_TIMESTAMP.to_owned())
        .build()
        .to_blob()
}
