//! Payloads read and returned by the cloudwatch API
//!
//! Cloudwatch: https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference

use super::Timestamp;
use serde::{Deserialize, Serialize};

// The values of the constants are per API, and for Cloudwatch they were found on
// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/making-api-requests.html

/// The version of the API matching the module.
/// The version must be included in the canonical request of all requests
pub const VERSION: &str = "2010-08-01";
/// The prefix to use for the x-amz-target header value in POST requests
pub const X_AMZ_TARGET_PREFIX: &str = "GraniteServiceVersion20100801";
/// The value of the content-type header value to set in POST requests
pub const POST_CONTENT_TYPE: &str = "application/json";
/// The value of the content-encoding header value to set in POST requests
pub const POST_CONTENT_ENCODING: &str = "amz-1.0";

/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_ListMetrics.html
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMetricsResponse {
    /// Inner Result
    pub list_metrics_result: ListMetricsResult,
}

/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_ListMetrics.html
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMetricsResult {
    /// List of Metric objects
    pub metrics: Vec<Metric>,
    /// Next Token to use for pagination in subsequent calls
    pub next_token: Option<String>,
    /// If you are using this operation in a monitoring account, this array
    /// contains the account IDs of the source accounts where the metrics in the
    /// returned data are from.
    ///
    /// This field is a 1:1 mapping between each metric that is returned and the ID
    /// of the owning account.
    pub owning_accounts: Option<Vec<String>>,
}

/// A Metric as returned by AWS API
///
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_Metric.html
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Metric {
    /// The list of dimension associated to the metric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Vec<Dimension>>,
    /// The name of the metric
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_name: Option<String>,
    /// The namespace of the metric (e.g. "AWS/EC2", "AWS/RDS", ...)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// A Metric-specific key-value metadata pair
///
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_Dimension.html
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Dimension {
    /// Name of the dimension
    pub name: String,
    /// Value of the dimension
    pub value: String,
}

/// A filter based on metric dimensions
///
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_DimensionFilter.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DimensionFilter {
    /// The dimension name to be matched.
    /// Length Constraints: Minimum length of 1. Maximum length of 255.
    name: String,
    /// The value of the dimension to be matched.
    /// Length Constraints: Minimum length of 1. Maximum length of 1024.
    value: Option<String>,
}

/// A request to get metric data as timeseries
///
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_GetMetricData.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetMetricDataRequest {
    /// The value specified is exclusive; results include data points up to the
    /// specified time stamp.
    ///
    /// For better performance, specify StartTime and EndTime values that align
    /// with the value of the metric's Period and sync up with the beginning and
    /// end of an hour. For example, if the Period of a metric is 5 minutes,
    /// specifying 12:05 or 12:30 as EndTime can get a faster response from
    /// CloudWatch than setting 12:07 or 12:29 as the EndTime.
    pub end_time: Timestamp,
    /// This structure includes the Timezone parameter, which you can use to
    /// specify your time zone so that the labels of returned data display the
    /// correct time for your time zone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_options: Option<LabelOptions>,
    /// The maximum number of data points the request should return before
    /// paginating. If you omit this, the default of 100,800 is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_datapoints: Option<u64>,
    /// The metric queries to be returned. A single GetMetricData call can
    /// include as many as 500 MetricDataQuery structures. Each of these
    /// structures can specify either a metric to retrieve, a Metrics Insights
    /// query, or a math expression to perform on retrieved data.
    pub metric_data_queries: Vec<MetricDataQuery>,
    /// Next Token to use for pagination in subsequent calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    /// Specify the order of data points in the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_by: Option<ScanOrder>,
    /// The time stamp indicating the earliest data to be returned.
    ///
    /// The value specified is inclusive; results include data points with the
    /// specified time stamp.
    ///
    /// CloudWatch rounds the specified time stamp as follows:
    ///
    ///  - Start time less than 15 days ago - Round down to the nearest whole
    ///    minute. For example, 12:32:34 is rounded down to 12:32:00.
    ///
    ///  - Start time between 15 and 63 days ago - Round down to the nearest
    ///    5-minute clock interval. For example, 12:32:34 is rounded down to 12:30:00.
    ///
    ///  - Start time greater than 63 days ago - Round down to the nearest 1-hour
    ///    clock interval. For example, 12:32:34 is rounded down to 12:00:00.
    ///
    /// If you set Period to 5, 10, or 30, the start time of your request is rounded
    /// down to the nearest time that corresponds to even 5-, 10-, or 30-second
    /// divisions of a minute. For example, if you make a query at (HH:mm:ss)
    /// 01:05:23 for the previous 10-second period, the start time of your request
    /// is rounded down and you receive data from 01:05:10 to 01:05:20. If you make
    /// a query at 15:07:17 for the previous 5 minutes of data, using a period of 5
    /// seconds, you receive data timestamped between 15:02:15 and 15:07:15.
    ///
    /// For better performance, specify StartTime and EndTime values that align with
    /// the value of the metric's Period and sync up with the beginning and end of
    /// an hour. For example, if the Period of a metric is 5 minutes, specifying
    /// 12:05 or 12:30 as StartTime can get a faster response from CloudWatch than
    /// setting 12:07 or 12:29 as the StartTime.
    pub start_time: Timestamp,
}

/// A set of options for custom labelling of series
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LabelOptions {
    /// The time zone to use for metric data return in this operation. The
    /// format is + or - followed by four digits. The first two digits indicate
    /// the number of hours ahead or behind of UTC, and the final two digits are
    /// the number of minutes. For example, +0130 indicates a time zone that is
    /// 1 hour and 30 minutes ahead of UTC. The default is +0000.
    pub timezone: Option<String>,
}

/// A part of a query of metric data.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MetricDataQuery {
    /// The ID of the account where the metrics are located.
    ///
    /// If you are performing a GetMetricData operation in a monitoring account,
    /// use this to specify which account to retrieve this metric from.
    ///
    /// If you are performing a PutMetricAlarm operation, use this to specify which
    /// account contains the metric that the alarm is watching.
    ///
    /// Length Constraints: Minimum length of 1. Maximum length of 255.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// This field can contain either a Metrics Insights query, or a metric math
    /// expression to be performed on the returned data. For more information
    /// about Metrics Insights queries, see [Metrics Insights query components
    /// and
    /// syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch-metrics-insights-querylanguage.html)
    /// in the Amazon CloudWatch User Guide.
    ///
    /// A math expression can use the Id of the other metrics or queries to
    /// refer to those metrics, and can also use the Id of other expressions to
    /// use the result of those expressions. For more information about metric
    /// math expressions, see [Metric Math Syntax and
    /// Functions](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/using-metric-math.html#metric-math-syntax)
    /// in the Amazon CloudWatch User Guide.
    ///
    /// Within each MetricDataQuery object, you must specify either Expression or
    /// MetricStat but not both.
    ///
    /// Length Constraints: Minimum length of 1. Maximum length of 2048.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
    /// A short name used to tie this object to the results in the response.
    /// This name must be unique within a single call to GetMetricData. If you
    /// are performing math expressions on this set of data, this name
    /// represents that data and can serve as a variable in the mathematical
    /// expression. The valid characters are letters, numbers, and underscore.
    /// The first character must be a lowercase letter.
    ///
    /// Length Constraints: Minimum length of 1. Maximum length of 255.
    pub id: String,
    /// A human-readable label for this metric or expression. This is especially
    /// useful if this is an expression, so that you know what the value
    /// represents. If the metric or expression is shown in a CloudWatch
    /// dashboard widget, the label is shown. If Label is omitted, CloudWatch
    /// generates a default.
    ///
    /// You can put dynamic expressions into a label, so that it is more
    /// descriptive. For more information, see [Using Dynamic
    /// Labels](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/graph-dynamic-labels.html).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The metric to be returned, along with statistics, period, and units. Use
    /// this parameter only if this object is retrieving a metric and not
    /// performing a math expression on returned data.
    ///
    /// Within one MetricDataQuery object, you must specify either Expression or
    /// MetricStat but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_stat: Option<MetricStat>,
    /// The granularity, in seconds, of the returned data points. For metrics
    /// with regular resolution, a period can be as short as one minute (60
    /// seconds) and must be a multiple of 60. For high-resolution metrics that
    /// are collected at intervals of less than one minute, the period can be 1,
    /// 5, 10, 30, 60, or any multiple of 60. High-resolution metrics are those
    /// metrics stored by a PutMetricData operation that includes a
    /// StorageResolution of 1 second.
    ///
    /// Valid range: Minimum value of 1.
    #[serde(rename = "Period")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_secs: Option<usize>,
    /// When used in GetMetricData, this option indicates whether to return the
    /// timestamps and raw data values of this metric. If you are performing
    /// this call just to do math expressions and do not also need the raw data
    /// returned, you can specify false. If you omit this, the default of true
    /// is used.
    ///
    /// When used in PutMetricAlarm, specify true for the one expression result
    /// to use as the alarm. For all other metrics and expressions in the same
    /// PutMetricAlarm operation, specify ReturnData as False.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_data: Option<bool>,
}

/// Order of points in a GetMetricData request
#[derive(Debug, Clone, Serialize, Copy)]
#[allow(missing_docs)]
#[serde(rename_all = "PascalCase")]
pub enum ScanOrder {
    TimestampAscending,
    TimestampDescending,
}

/// Metric-based data request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MetricStat {
    /// Metric to pull in the request
    pub metric: Metric,
    /// The granularity, in seconds, of the returned data points. For metrics
    /// with regular resolution, a period can be as short as one minute (60
    /// seconds) and must be a multiple of 60. For high-resolution metrics that
    /// are collected at intervals of less than one minute, the period can be 1,
    /// 5, 10, 30, 60, or any multiple of 60. High-resolution metrics are those
    /// metrics stored by a PutMetricData call that includes a StorageResolution
    /// of 1 second.
    ///
    /// If the StartTime parameter specifies a time stamp that is greater than 3
    /// hours ago, you must specify the period as follows or no data points in that
    /// time range is returned:
    ///
    /// - Start time between 3 hours and 15 days ago - Use a multiple of 60 seconds
    ///   (1 minute).
    ///   
    /// - Start time between 15 and 63 days ago - Use a multiple of 300 seconds (5
    ///   minutes).
    ///   
    /// - Start time greater than 63 days ago - Use a multiple of 3600 seconds (1
    ///   hour).
    pub period: usize,
    /// The statistic to return. It can include any CloudWatch statistic or
    /// extended statistic.
    pub stat: String,
    /// When you are using a Put operation, this defines what unit you want to
    /// use when storing the metric.
    ///
    /// In a Get operation, if you omit Unit then all data that was collected with
    /// any unit is returned, along with the corresponding units that were specified
    /// when the data was reported to CloudWatch. If you specify a unit, the
    /// operation returns only data that was collected with that unit specified. If
    /// you specify a unit that does not match the data collected, the results of
    /// the operation are null. CloudWatch does not perform unit conversions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
}

/// Units for metric data series
#[derive(Debug, Clone, Serialize)]
#[allow(missing_docs)]
#[serde(rename_all = "PascalCase")]
pub enum Unit {
    Seconds,
    Microseconds,
    Milliseconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Bits,
    Kilobits,
    Megabits,
    Gigabits,
    Terabits,
    Percent,
    Count,
    #[serde(rename = "Bytes/Second")]
    BytesPerSecond,
    #[serde(rename = "Kilobytes/Second")]
    KilobytesPerSecond,
    #[serde(rename = "Megabytes/Second")]
    MegabytesPerSecond,
    #[serde(rename = "Gigabytes/Second")]
    GigabytesPerSecond,
    #[serde(rename = "Terabytes/Second")]
    TerabytesPerSecond,
    #[serde(rename = "Bits/Second")]
    BitsPerSecond,
    #[serde(rename = "Kilobits/Second")]
    KilobitsPerSecond,
    #[serde(rename = "Megabits/Second")]
    MegabitsPerSecond,
    #[serde(rename = "Gigabits/Second")]
    GigabitsPerSecond,
    #[serde(rename = "Terabits/Second")]
    TerabitsPerSecond,
    #[serde(rename = "Percent/Second")]
    PercentPerSecond,
    #[serde(rename = "Count/Second")]
    CountPerSecond,
    None,
}

/// Response to a GetMetricData request
///
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_GetMetricData.html#API_GetMetricData_ResponseElements
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetMetricDataResponse {
    /// A list of messages with additional information about the data returned.
    pub messages: Vec<MessageData>,
    /// The list of results for the query
    pub metric_data_results: Vec<MetricDataResult>,
    /// Next Token to use for pagination in subsequent calls
    pub next_token: Option<String>,
}

/// A message returned by the GetMetricDataAPI, including a code and a
/// description.
///
/// If a cross-Region GetMetricData operation fails with a code of Forbidden and
/// a value of Authentication too complex to retrieve cross region data, you can
/// correct the problem by running the GetMetricData operation in the same
/// Region where the metric data is.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageData {
    #[allow(missing_docs)]
    pub code: Option<String>,
    #[allow(missing_docs)]
    pub value: Option<String>,
}

/// The result of a Metric Data Query
///
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricDataResult.html
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MetricDataResult {
    /// The short name you specified to represent this metric.
    pub id: Option<String>,
    /// The human-readable label associated with the data.
    pub label: Option<String>,
    /// A list of messages with additional information about the data returned.
    pub messages: Option<Vec<MessageData>>,
    ///The status of the returned data. Complete indicates that all data points
    /// in the requested time range were returned. PartialData means that an
    /// incomplete set of data points were returned. You can use the NextToken
    /// value that was returned and repeat your request to get more data points.
    /// NextToken is not returned if you are performing a math expression.
    /// InternalError indicates that an error occurred. Retry your request using
    /// NextToken, if present.
    pub status_code: Option<StatusCode>,
    /// The timestamps for the data points, formatted in Unix timestamp format.
    /// The number of timestamps always matches the number of values and the
    /// value for Timestamps[x] is Values[x].
    pub timestamps: Option<Vec<Timestamp>>,
    /// The data points for the metric corresponding to Timestamps. The number
    /// of values always matches the number of timestamps and the timestamp for
    /// Values[x] is Timestamps[x].
    pub values: Option<Vec<f64>>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "PascalCase")]
pub enum StatusCode {
    Complete,
    InternalError,
    PartialData,
    Forbidden,
}
