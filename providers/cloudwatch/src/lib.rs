//! AWS CloudWatch provider
//!
//! This provider uses AWS credentials to connect to
//! CloudWatch and provide alerts and metric graphs about
//! an instance. To do so it uses:
//! - CloudWatch metrics
//! - CloudWatch logs
//! - Resource groups tagging

mod client;
mod config;
mod constants;
mod panic;
mod queries;
mod types;
pub mod utils;

use config::Config;
use constants::*;
use fiberplane_pdk::prelude::*;
use queries::*;
use std::env;
pub use types::*;

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[pdk_export]
async fn get_supported_query_types(_config: ProviderConfig) -> Vec<SupportedQueryType> {
    panic::init_panic_hook();
    vec![
        SupportedQueryType::new(GRAPH_METRIC_QUERY_TYPE).with_label("AWS: graph metrics")
            .with_schema( vec![
                TextField::new()
                    .with_name(EXPRESSION_PARAM_NAME)
                    .with_label("Query of the metric to graph")
                    .with_placeholder("select avg(CPUUtilization) from schema(\"AWS/EC2\", InstanceId) group by InstanceId")
                    .multiline()
                    .required()
                    .into()
                ,
                TextField::new()
                    .with_name(LABEL_PARAM_NAME)
                    .with_label("Title of the timeseries")
                    .with_placeholder("CPU usage")
                    .multiline()
                    .required()
                .into(),
                TextField::new()
                    .with_name(PERIOD_PARAM_NAME)
                    .with_label("Period of time between points")
                    .with_placeholder("30")
                    .required()
                .into(),
                DateTimeRangeField::new()
                    .with_name(TIME_RANGE_PARAM_NAME)
                    .with_label("Specify a time range")
                    .required()
                .into(),
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(LIST_METRICS_QUERY_TYPE).with_label("AWS: list metrics")
            .with_schema(vec![
                TextField::new()
                    .with_name(TAG_KEY_PARAM_NAME)
                    .with_label("A tag key to filter by")
                    .with_placeholder("Environment")
                    .with_suggestions()
                .into(),
                TextField::new()
                    .with_name(TAG_VALUE_PARAM_NAME)
                    .with_label("A tag value to filter by")
                    .with_placeholder("production")
                    .with_suggestions()
                .into(),
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(DESCRIBE_LOG_GROUPS_QUERY_TYPE)
            .with_label("AWS: list log groups")
            .with_schema(vec![TextField::new()
                .with_name(DUMMY_PARAM_NAME)
                .with_label("Dummy field to get a button")
            .into()])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(START_LOG_QUERY_QUERY_TYPE)
            .with_label("AWS: start a Logs query")
            .with_schema(vec![
                TextField::new()
                    .with_name(QUERY_PARAM_NAME)
                    .with_label("Query for logs")
                    .with_placeholder(&format!("fields {}, {} | sort {} desc | limit 20", TS_KEY.0, BODY_KEY.0, TS_KEY.0))
                    .required()
                    .with_suggestions()
                .into(),
                TextField::new()
                    .with_name(LOG_GROUP_PARAM_NAME)
                    .with_label("Group names to do the query on")
                    .with_placeholder("RDSOSMetrics")
                    .multiline()
                    .required()
                    .with_suggestions()
                .into(),
                DateTimeRangeField::new()
                    .with_name(TIME_RANGE_PARAM_NAME)
                    .with_label("Specify a time range")
                    .required()
                .into(),
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(DESCRIBE_QUERIES_QUERY_TYPE)
            .with_label("AWS: list Logs queries")
            .with_schema(vec![
                TextField::new()
                    .with_name(LOG_GROUP_PARAM_NAME)
                    .with_label("Group name to list the queries for")
                    .with_placeholder("RDSOSMetrics")
                    .with_suggestions()
                .into(),
                // TODO: Add a QueryStatus filter parameter once we can use the Select field (FP-2590)
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(GET_QUERY_RESULTS_QUERY_TYPE)
            .with_label("AWS: display Logs query results")
            .with_schema(vec![TextField::new()
                .with_name(QUERY_ID_PARAM_NAME)
                .with_label("Query Id to look for")
                .with_placeholder("640e6499-5f95-4e53-beb6-64dafb482180")
                .required()
            .into()])
            .supporting_mime_types(&[EVENTS_MIME_TYPE]),
        SupportedQueryType::new(GET_LOG_RECORD_QUERY_TYPE)
            .with_label("AWS: display Logs entry details")
            .with_schema(vec![TextField::new()
                .with_name(LOG_RECORD_POINTER_PARAM_NAME)
                .with_label("Log record pointer to fetch")
                .with_placeholder("123456789")
                .required()
            .into()])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
        SupportedQueryType::new(SUGGESTIONS_QUERY_TYPE).supporting_mime_types(&[SUGGESTIONS_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob> {
    panic::init_panic_hook();

    log(format!(
        "CloudWatch: (commit: {COMMIT_HASH}, built at: {BUILD_TIMESTAMP}) invoked for query type \"{}\" and query data \"{:?}\"",
        request.query_type, request.query_data
    ));

    let config: Config =
        serde_json::from_value(request.config.clone()).map_err(|err| Error::Config {
            message: format!("Error parsing config: {err:?}"),
        })?;

    match request.query_type.as_str() {
        STATUS_QUERY_TYPE => status::check_status(config).await,
        LIST_METRICS_QUERY_TYPE => list_metrics::invoke2_handler(config, request).await,
        GRAPH_METRIC_QUERY_TYPE => graph_metric::invoke2_handler(config, request).await,
        DESCRIBE_LOG_GROUPS_QUERY_TYPE => {
            describe_log_groups::invoke2_handler(config, request).await
        }
        DESCRIBE_QUERIES_QUERY_TYPE => describe_queries::invoke2_handler(config, request).await,
        START_LOG_QUERY_QUERY_TYPE => start_query::invoke2_handler(config, request).await,
        GET_QUERY_RESULTS_QUERY_TYPE => get_query_results::invoke2_handler(config, request).await,
        GET_LOG_RECORD_QUERY_TYPE => get_log_record::invoke2_handler(config, request).await,
        SUGGESTIONS_QUERY_TYPE => auto_suggest::invoke2_handler(request.query_data, config).await,
        _ => Err(Error::UnsupportedRequest),
    }
}

#[pdk_export]
fn create_cells(query_type: String, response: Blob) -> Result<Vec<Cell>> {
    panic::init_panic_hook();

    match query_type.as_str() {
        STATUS_QUERY_TYPE => Ok(Vec::new()),
        LIST_METRICS_QUERY_TYPE => list_metrics::create_cells_handler(response),
        GRAPH_METRIC_QUERY_TYPE => graph_metric::create_cells_handler(response),
        GET_QUERY_RESULTS_QUERY_TYPE => get_query_results::create_cells_handler(response),
        GET_LOG_RECORD_QUERY_TYPE => get_log_record::create_cells_handler(response),
        _ => Err(Error::UnsupportedRequest),
    }
}

#[pdk_export]
fn extract_data(response: Blob, mime_type: String, query: Option<String>) -> Result<Blob> {
    panic::init_panic_hook();

    if response.mime_type.starts_with(QUERY_RESULTS_MIME_TYPE) {
        return get_query_results::extract_data_handler(response, mime_type, query);
    }

    Err(Error::UnsupportedRequest)
}

#[pdk_export]
fn get_config_schema() -> Vec<ConfigField> {
    vec![
        TextField::new()
            .with_name("region")
            .with_label("Region of AWS endpoints to use")
            .with_placeholder("For example, eu-central-2")
            .required()
            .into(),
        TextField::new()
            .with_name("access_key_id")
            .with_label("AWS Access Key ID")
            .with_placeholder("For example, AKIAIOSFODNN7EXAMPLE")
            .required()
            .into(),
        TextField::new()
            .with_name("secret_access_key")
            .with_label("AWS Secret Access Key")
            .with_placeholder("For example, wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
            .required()
            .into(),
    ]
}
