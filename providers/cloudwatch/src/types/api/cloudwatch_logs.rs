//! Payloads read and returned by the cloudwatch logs API
//!
//! Cloudwatch logs: https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/Welcome.html

mod describe_log_groups;
mod describe_log_streams;
mod describe_queries;
mod get_log_record;
mod get_query_results;
mod start_query;

use super::IntTimestamp as Timestamp;
use serde::{Deserialize, Serialize};

pub use describe_log_groups::*;
pub use describe_log_streams::*;
pub use describe_queries::*;
pub use get_log_record::*;
pub use get_query_results::*;
pub use start_query::*;

// The values of the constants are per API, and for Cloudwatch they were found from
// sample requests on
// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_GetLogEvents.html

/// The version of the API matching the module.
/// The version must be included in the canonical request of all requests
pub const VERSION: &str = "2014-03-28";
/// The prefix to use for the x-amz-target header value in POST requests
pub const X_AMZ_TARGET_PREFIX: &str = "Logs_20140328";
/// The value of the content-type header value to set in POST requests
pub const POST_CONTENT_TYPE: &str = "application/x-amz-json-1.1";

/// Log Group
///
/// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_LogGroup.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogGroup {
    /// The Amazon Resource Name (ARN) of the log group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    /// The creation time of the log group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_time: Option<Timestamp>,
    /// Displays whether this log group has a protection policy, or whether it
    /// had one in the past.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_protection_status: Option<DataProtectionStatus>,
    /// The Amazon Resource Name (ARN) of the AWS KMS key to use when encrypting
    /// log data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
    /// The name of the log group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_name: Option<String>,
    /// The number of metric filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_filter_count: Option<u64>,
    /// The number of days to retain the log events in the specified log group.
    /// Possible values are: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365,
    /// 400, 545, 731, 1827, 2192, 2557, 2922, 3288, and 3653.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_in_days: Option<u64>,
    /// The number of bytes stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored_bytes: Option<u64>,
}

/// Status of the data protection policy
///
/// A data protection policy can help safeguard sensitive data that's ingested
/// by the log group by auditing and masking the sensitive log data.
#[derive(Debug, Clone, Deserialize, Copy)]
#[serde(rename = "UPPERCASE")]
pub enum DataProtectionStatus {
    Activated,
    Deleted,
    Archived,
    Disabled,
}

/// Log Stream
///
/// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_LogStream.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogStream {
    /// The Amazon Resource Name (ARN) of the log stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    /// The creation time of the log group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_time: Option<Timestamp>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_event_timestamp: Option<Timestamp>,
    /// The lastEventTime [sic] value updates on an eventual consistency basis.
    /// It typically updates in less than an hour from ingestion, but in rare
    /// situations might take longer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_event_timestamp: Option<Timestamp>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ingestion_time: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_stream_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(
        since = "0.1.0",
        note = "As of June 17, 2019, this parameter is no longer supported for log streams, and is always reported as zero. This change applies only to log streams. The storedBytes parameter for log groups is not affected."
    )]
    pub stored_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_sequence_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Default)]
pub enum QueryStatus {
    Scheduled,
    Running,
    Complete,
    Failed,
    Cancelled,
    Timeout,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInfo {
    pub create_time: Option<Timestamp>,
    pub log_group_name: Option<String>,
    pub query_id: Option<String>,
    pub query_string: Option<String>,
    #[serde(default)]
    pub status: QueryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum OrderStreamsBy {
    LogStreamName,
    LastEventTime,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultField {
    pub field: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryStatistics {
    pub bytes_scanned: Option<f64>,
    pub records_matched: Option<f64>,
    pub records_scanned: Option<f64>,
}
