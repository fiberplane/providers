#![allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, collections::HashMap};

pub use fiberplane::protocols::formatting::Annotation;
pub use fiberplane::protocols::formatting::AnnotationWithOffset;
pub use fiberplane::protocols::blobs::Blob;
pub use fiberplane::protocols::core::Cell;
pub use fiberplane::protocols::core::CheckboxCell;
pub use fiberplane::protocols::core::CodeCell;
pub use fiberplane::protocols::core::DividerCell;
pub use fiberplane::protocols::core::ElasticsearchCell;
pub use fiberplane::protocols::blobs::EncodedBlob;
pub use fiberplane::protocols::core::GraphCell;
pub use fiberplane::protocols::core::GraphType;
pub use fiberplane::protocols::core::HeadingCell;
pub use fiberplane::protocols::core::HeadingType;
pub use fiberplane::protocols::core::ImageCell;
pub use fiberplane::protocols::core::Instant;
pub use fiberplane::protocols::core::ListItemCell;
pub use fiberplane::protocols::core::ListType;
pub use fiberplane::protocols::core::LogCell;
pub use fiberplane::protocols::core::LogRecord;
pub use fiberplane::protocols::core::LokiCell;
pub use fiberplane::protocols::formatting::Mention;
pub use fiberplane::protocols::core::Metric;
pub use fiberplane::protocols::core::Point;
pub use fiberplane::protocols::core::PrometheusCell;
pub use fiberplane::protocols::core::ProviderCell;
pub use fiberplane::protocols::core::Series;
pub use fiberplane::protocols::core::StackingType;
pub use fiberplane::protocols::core::TableCell;
pub use fiberplane::protocols::core::TextCell;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display on the button.
    pub label: String,

    /// Value of the button as it will be included in the encoded query. By
    /// checking whether the field with the given `name` has this `value`,
    /// providers may know which button was pressed.
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckboxField {
    /// Whether the checkbox should be initially checked if no query data is
    /// present.
    pub checked: bool,

    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the checkbox.
    pub label: String,

    /// Value of the field as it will be included in the encoded query. Note
    /// that only checked checkboxes will be included.
    pub value: String,
}

/// Defines a field that produces a date value in `YYYY-MM-DD` format.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field.
    pub label: String,

    /// Whether a value is required.
    pub required: bool,
}

/// Defines a field that produces a date-time value that is valid RFC 3339 as
/// well as valid ISO 8601-1:2019.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateTimeField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field.
    pub label: String,

    /// Whether a value is required.
    pub required: bool,
}

/// Defines a field that produces two `DateTime` values, a "from" and a "to"
/// value, separated by a space.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateTimeRangeField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field.
    pub label: String,

    /// Whether a value is required.
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCell {
    pub id: String,
    pub thread_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    UnsupportedRequest,
    #[serde(rename_all = "camelCase")]
    Http { error: HttpRequestError },
    #[serde(rename_all = "camelCase")]
    Data { message: String },
    #[serde(rename_all = "camelCase")]
    Deserialization { message: String },
    #[serde(rename_all = "camelCase")]
    Config { message: String },
    #[serde(rename_all = "camelCase")]
    Other { message: String },
}

/// Defines a field that allows files to be uploaded as part of the query data.
///
/// Note that query data that includes files will be encoded as
/// "multipart/form-data".
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field.
    pub label: String,

    /// Whether multiple files may be uploaded.
    pub multiple: bool,

    /// Whether a file is required.
    pub required: bool,
}

pub type Formatting = Vec<AnnotationWithOffset>;

/// HTTP request options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpRequestMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_bytes::ByteBuf>,
}

/// Possible errors that may happen during an HTTP request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HttpRequestError {
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    ResponseTooBig,
    #[serde(rename_all = "camelCase")]
    ServerError { status_code: u16, response: serde_bytes::ByteBuf },
    #[serde(rename_all = "camelCase")]
    Other { reason: String },
}

/// HTTP request method.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpRequestMethod {
    Delete,
    Get,
    Head,
    Post,
}

/// Response to an HTTP request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    pub body: serde_bytes::ByteBuf,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}

/// Defines a field that allows labels to be selected.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field (not to be confused with
    /// labels to be selected).
    pub label: String,

    /// Whether multiple labels may be selected.
    pub multiple: bool,

    /// Whether a value is required.
    pub required: bool,
}

/// An individual log record
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyLogRecord {
    pub timestamp: Timestamp,
    pub body: String,
    pub attributes: HashMap<String, String>,
    pub resource: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<serde_bytes::ByteBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<serde_bytes::ByteBuf>,
}

/// Legacy `ProviderRequest` from the Provider 1.0 protocol.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LegacyProviderRequest {
    Instant(QueryInstant),
    Series(QueryTimeRange),
    Proxy(ProxyRequest),
    /// Requests a list of auto-suggestions. Note that these are
    /// context-unaware.
    AutoSuggest,
    Logs(QueryLogs),
    /// Check data source status, any issue will be returned as `Error`
    Status,
}

/// Legacy `ProviderResponse` from the 1.0 protocol.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LegacyProviderResponse {
    #[serde(rename_all = "camelCase")]
    Error { error: Error },
    #[serde(rename_all = "camelCase")]
    Instant { instants: Vec<Instant> },
    #[serde(rename_all = "camelCase")]
    Series { series: Vec<Series> },
    #[serde(rename_all = "camelCase")]
    AutoSuggest { suggestions: Vec<Suggestion> },
    #[serde(rename_all = "camelCase")]
    LogRecords { log_records: Vec<LegacyLogRecord> },
    StatusOk,
}

/// Defines a field that allows labels to be selected.
///
/// Note that because the value is encoded as a string anyway, and depending on
/// the chosen `step` this field can be used for either integers or floating
/// point numbers, the values in the schema are simply presented as strings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field.
    pub label: String,

    /// Optional maximum value to be selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,

    /// Optional minimal value to be selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,

    /// Whether a value is required.
    pub required: bool,

    /// Specifies the granularity that any specified numbers must adhere to.
    ///
    /// If omitted, `step` defaults to "1", meaning only integers are allowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRequest {
    /// Query type that is part of the
    /// [Intent](https://www.notion.so/fiberplane/RFC-45-Provider-Protocol-2-0-Revised-4ec85a0233924b2db0010d8cdae75e16#c8ed5dfbfd764e6bbd5c5b79333f9d6e)
    /// through which the provider is invoked.
    pub query_type: String,

    /// Query data.
    ///
    /// This is usually populated from the [ProviderCell::query_data] field,
    /// meaning the MIME type will be `"application/x-www-form-urlencoded"`
    /// when produced by Studio's Query Builder.
    pub query_data: Blob,

    /// Configuration for the data source.
    pub config: rmpv::Value,

    /// Optional response from a previous invocation.
    /// May be used for implementing things like filtering without additional
    /// server roundtrip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response: Option<Blob>,
}

/// Relays requests for a data-source to a proxy server registered with the API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyRequest {
    /// ID of the proxy as known by the API.
    pub proxy_id: String,

    /// Name of the data source exposed by the proxy.
    pub data_source_name: String,

    /// Request data to send to the proxy
    pub request: serde_bytes::ByteBuf,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryField {
    Button(ButtonField),
    Checkbox(CheckboxField),
    Date(DateField),
    DateTime(DateTimeField),
    DateTimeRange(DateTimeRangeField),
    File(FileField),
    Label(LabelField),
    Number(NumberField),
    Select(SelectField),
    Text(TextField),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInstant {
    pub query: String,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryLogs {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    pub time_range: TimeRange,
}

pub type QuerySchema = Vec<QueryField>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTimeRange {
    pub query: String,
    pub time_range: TimeRange,
}

/// Defines a field that allows selection from a predefined list of options.
///
/// Note that values to be selected from can be either hard-coded in the schema,
/// or fetched on-demand the same way as auto-suggestions.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the field.
    pub label: String,

    /// Whether multiple values may be selected.
    pub multiple: bool,

    /// A list of options to select from. If empty, the auto-suggest mechanism
    /// is used to fetch options as needed.
    pub options: Vec<String>,

    /// An optional list of fields that should be filled in before allowing the
    /// user to fill in this field. This forces a certain ordering in the data
    /// entry, which enables richer auto-suggestions, as the filled in
    /// prerequisite fields can provide additional context.
    pub prerequisites: Vec<String>,

    /// Whether a value is required.
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// Suggested text.
    pub text: String,

    /// Optional description to go along with this suggestion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Defines which query types are supported by a provider.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedQueryType {
    /// The query type supported by the provider.
    ///
    /// There are predefined query types, such as "table" and "log", but
    /// providers may also implement custom query types, which it should prefix
    /// with "x-".
    pub query_type: String,

    /// The query schema defining the format of the `query_data` to be submitted
    /// with queries of this type.
    pub schema: QuerySchema,

    /// MIME types supported for extraction. Any MIME type specified here should
    /// be valid as an argument to `extract_data()` when passed a response from
    /// queries of this type.
    ///
    /// E.g.:
    /// ```
    /// vec![
    ///     "application/vnd.fiberplane.events",
    ///     "application/vnd.fiberplane.metrics"
    /// ];
    /// ```
    pub mime_types: Vec<String>,
}

/// Defines a free-form text entry field.
///
/// Is commonly used for filter text and query entry. For the latter case,
/// `supports_highlighting` can be set to `true` if the provider supports syntax
/// highlighting for the query language.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextField {
    /// Name of the field as it will be included in the encoded query.
    pub name: String,

    /// Suggested label to display along the form field.
    pub label: String,

    /// Suggests whether multi-line input is useful for this provider.
    pub multiline: bool,

    /// An optional list of fields that should be filled in before allowing the
    /// user to fill in this field. This forces a certain ordering in the data
    /// entry, which enables richer auto-suggestions, as the filled in
    /// prerequisite fields can provide additional context.
    pub prerequisites: Vec<String>,

    /// Whether a value is required.
    pub required: bool,

    /// Whether the provider implements syntax highlighting for this field.
    /// See `highlight_field()` in the protocol definition.
    pub supports_highlighting: bool,
}

/// A range in time from a given timestamp (inclusive) up to another timestamp
/// (exclusive).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub from: Timestamp,
    pub to: Timestamp,
}

pub type Timestamp = f64;
