use const_format::formatcp;
pub use fiberplane_models::providers::{
    CELLS_MIME_TYPE, EVENTS_MIME_TYPE, SUGGESTIONS_MIME_TYPE, TIMESERIES_MIME_TYPE,
};

pub const PROVIDER_TYPE: &str = "cloudwatch";

pub const GRAPH_METRIC_QUERY_TYPE: &str = "x-graph-metric";
pub const LIST_METRICS_QUERY_TYPE: &str = "x-list-metrics";
pub const DESCRIBE_LOG_GROUPS_QUERY_TYPE: &str = "x-describe-log-groups";
pub const DESCRIBE_QUERIES_QUERY_TYPE: &str = "x-describe-queries";
pub const START_LOG_QUERY_QUERY_TYPE: &str = "x-start-query";
pub const GET_QUERY_RESULTS_QUERY_TYPE: &str = "x-get-query-results";
pub const GET_LOG_RECORD_QUERY_TYPE: &str = "x-get-log-record";

pub const SUGGESTIONS_MSGPACK_MIME_TYPE: &str = formatcp!("{SUGGESTIONS_MIME_TYPE}+msgpack");
pub const TIMESERIES_MSGPACK_MIME_TYPE: &str = formatcp!("{TIMESERIES_MIME_TYPE}+msgpack");
pub const CELLS_MSGPACK_MIME_TYPE: &str = formatcp!("{CELLS_MIME_TYPE}+msgpack");
pub const EVENTS_MSGPACK_MIME_TYPE: &str = formatcp!("{EVENTS_MIME_TYPE}+msgpack");
pub const EVENTS_JSON_MIME_TYPE: &str = formatcp!("{EVENTS_MIME_TYPE}+json");
pub const LIST_METRICS_MIME_TYPE: &str = "application/json";
pub const QUERY_DATA_MIME_TYPE: &str = "application/x-www-form-urlencoded";
pub const QUERY_RESULTS_MIME_TYPE: &str =
    "application/vnd.fiberplane.providers.cloudwatch.query-results";

pub const TIME_RANGE_PARAM_NAME: &str = "time_range";
pub const EXPRESSION_PARAM_NAME: &str = "expression";
pub const LABEL_PARAM_NAME: &str = "label";
pub const PERIOD_PARAM_NAME: &str = "timeperiod";
pub const TAG_KEY_PARAM_NAME: &str = "tag_key";
pub const TAG_VALUE_PARAM_NAME: &str = "tag_value";
pub const DUMMY_PARAM_NAME: &str = "dummy";
pub const LOG_GROUP_PARAM_NAME: &str = "log_group";
pub const QUERY_ID_PARAM_NAME: &str = "query_id";
pub const QUERY_PARAM_NAME: &str = "query";
pub const LOG_RECORD_POINTER_PARAM_NAME: &str = "ptr";

// https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CWL_AnalyzeLogData-discoverable-fields.html
// The pairs are matched with a description that can be used in Auto Suggestions
pub const TS_KEY: (&str, &str) = ("@timestamp", "Timestamp of the event");
pub const INGESTION_TS_KEY: (&str, &str) = ("@ingestionTime", "Time of ingestion in Cloudwatch");
pub const BODY_KEY: (&str, &str) = ("@message", "Raw document stored with all keys");
pub const LOG_KEY: (&str, &str) = ("@log", "Log group that holds the entry");
pub const TRACE_KEY: (&str, &str) = ("@xrayTraceId", "XRay: Trace ID from when available");
pub const XRAY_SPAN_KEY: (&str, &str) = ("@xraySpanId", "XRay: Span ID from when available");
pub const SPAN_KEY: (&str, &str) = ("@requestId", "Lambda: Request ID");
pub const LAMBDA_BILLED_KEY: (&str, &str) = ("@billedDuration", "Lambda: Billed Duration");
pub const LAMBDA_DURATION_KEY: (&str, &str) = ("@duration", "Lambda: Duration");
pub const LOG_STREAM_KEY: (&str, &str) = ("@logStream", "Log stream that holds the entry");
pub const PTR_KEY: (&str, &str) = (
    "@ptr",
    "Pointer to query more information (Automatically included)",
);

pub const DISCOVERABLE_FIELDS: [(&str, &str); 11] = [
    TS_KEY,
    INGESTION_TS_KEY,
    BODY_KEY,
    LOG_KEY,
    TRACE_KEY,
    SPAN_KEY,
    XRAY_SPAN_KEY,
    LAMBDA_DURATION_KEY,
    LAMBDA_BILLED_KEY,
    LOG_STREAM_KEY,
    PTR_KEY,
];
