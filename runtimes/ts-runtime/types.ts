// ============================================= //
// Types for WebAssembly runtime                 //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

/**
 * A rich-text annotation.
 *
 * Annotations are typically found inside a `Formatting` vector.
 */
export type Annotation =
    | { type: "start_bold" }
    | { type: "end_bold" }
    | { type: "start_code" }
    | { type: "end_code" }
    | { type: "start_highlight" }
    | { type: "end_highlight" }
    | { type: "start_italics" }
    | { type: "end_italics" }
    | { type: "start_link"; url: string }
    | { type: "end_link" }
    | { type: "mention" } & Mention
    | { type: "start_strikethrough" }
    | { type: "end_strikethrough" }
    | { type: "start_underline" }
    | { type: "end_underline" };

/**
 * Newtype representing `(offset, Annotation)` tuples.
 *
 * Used inside the `Formatting` vector.
 */
export type AnnotationWithOffset = {
    offset: number;
} & Annotation;

/**
 * Binary blob for passing data in arbitrary encodings.
 *
 * Binary blobs are both consumed and produced by providers. Note that for many
 * use-cases, we use agreed on MIME types as defined in
 * [RFC 47](https://www.notion.so/fiberplane/RFC-47-Data-Model-for-Providers-2-0-0b5b1716dbc8450f882d33effb388c5b).
 * Providers are able to use custom MIME types if they desire.
 *
 * We can also store blobs in cells, but for this we use [EncodedBlob] to allow
 * JSON serialization.
 */
export type Blob = {
    /**
     * Raw data.
     */
    data: ArrayBuffer;

    /**
     * MIME type to use for interpreting the raw data.
     *
     * We keep track of this, so that we can elide unnecessary calls to
     * `extract_data()`, and are able to perform migrations on data specified
     * in any of the `application/vnd.fiberplane.*` types. For other types of
     * data, providers are responsible for migrations, and they are able to
     * include version numbers in their MIME type strings, if desired.
     */
    mimeType: string;
};

export type ButtonField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display on the button.
     */
    label: string;

    /**
     * Value of the button as it will be included in the encoded query. By
     * checking whether the field with the given `name` has this `value`,
     * providers may know which button was pressed.
     */
    value: string;
};

/**
 * Representation of a single notebook cell.
 */
export type Cell =
    | { type: "checkbox" } & CheckboxCell
    | { type: "code" } & CodeCell
    | { type: "discussion" } & DiscussionCell
    | { type: "divider" } & DividerCell
    | { type: "elasticsearch" } & ElasticsearchCell
    | { type: "graph" } & GraphCell
    | { type: "heading" } & HeadingCell
    | { type: "image" } & ImageCell
    | { type: "list_item" } & ListItemCell
    | { type: "log" } & LogCell
    | { type: "loki" } & LokiCell
    | { type: "prometheus" } & PrometheusCell
    | { type: "provider" } & ProviderCell
    | { type: "table" } & TableCell
    | { type: "text" } & TextCell;

export type CheckboxCell = {
    id: string;
    checked: boolean;
    content: string;

    /**
     * Optional formatting to be applied to the cell's content.
     */
    formatting?: Formatting;
    level?: number;
    readOnly?: boolean;
};

export type CheckboxField = {
    /**
     * Whether the checkbox should be initially checked if no query data is
     * present.
     */
    checked: boolean;

    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the checkbox.
     */
    label: string;

    /**
     * Value of the field as it will be included in the encoded query. Note
     * that only checked checkboxes will be included.
     */
    value: string;
};

export type CodeCell = {
    id: string;
    content: string;
    readOnly?: boolean;

    /**
     * Optional MIME type to use for syntax highlighting.
     */
    syntax?: string;
};

/**
 * Defines a field that produces a date value in `YYYY-MM-DD` format.
 */
export type DateField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field.
     */
    label: string;

    /**
     * Whether a value is required.
     */
    required: boolean;
};

/**
 * Defines a field that produces a date-time value that is valid RFC 3339 as
 * well as valid ISO 8601-1:2019.
 */
export type DateTimeField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field.
     */
    label: string;

    /**
     * Whether a value is required.
     */
    required: boolean;
};

/**
 * Defines a field that produces two `DateTime` values, a "from" and a "to"
 * value, separated by a space.
 */
export type DateTimeRangeField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field.
     */
    label: string;

    /**
     * Whether a value is required.
     */
    required: boolean;
};

export type DiscussionCell = {
    id: string;
    threadId: string;
    readOnly?: boolean;
};

export type DividerCell = {
    id: string;
    readOnly?: boolean;
};

export type ElasticsearchCell = {
    id: string;
    content: string;
    readOnly?: boolean;
};

/**
 * base64-encoded version of [Blob].
 */
export type EncodedBlob = {
    /**
     * Raw data, encoded using base64 so it can be serialized using JSON.
     */
    data: string;

    /**
     * MIME type to use for interpreting the raw data.
     *
     * See [Blob::mime_type].
     */
    mimeType: string;
};

export type Error =
    | { type: "unsupported_request" }
    | { type: "http"; error: HttpRequestError }
    | { type: "data"; message: string }
    | { type: "deserialization"; message: string }
    | { type: "config"; message: string }
    | { type: "other"; message: string };

/**
 * Defines a field that allows files to be uploaded as part of the query data.
 *
 * Note that query data that includes files will be encoded as
 * "multipart/form-data".
 */
export type FileField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field.
     */
    label: string;

    /**
     * Whether multiple files may be uploaded.
     */
    multiple: boolean;

    /**
     * Whether a file is required.
     */
    required: boolean;
};

export type Formatting = Array<AnnotationWithOffset>;

export type GraphCell = {
    id: string;

    /**
     * Optional formatting to be applied to the cell's title.
     */
    formatting?: Formatting;
    graphType: GraphType;
    stackingType: StackingType;
    readOnly?: boolean;
    sourceIds: Array<string>;
    timeRange?: TimeRange;
    title: string;
    data?: Record<string, Array<Series>>;
};

export type GraphType =
    | "bar"
    | "line";

export type HeadingCell = {
    id: string;
    headingType: HeadingType;
    content: string;

    /**
     * Optional formatting to be applied to the cell's content.
     */
    formatting?: Formatting;
    readOnly?: boolean;
};

export type HeadingType =
    | "h1"
    | "h2"
    | "h3";

/**
 * HTTP request options.
 */
export type HttpRequest = {
    url: string;
    method: HttpRequestMethod;
    headers?: Record<string, string>;
    body?: ArrayBuffer;
};

/**
 * Possible errors that may happen during an HTTP request.
 */
export type HttpRequestError =
    | { type: "offline" }
    | { type: "no_route" }
    | { type: "connection_refused" }
    | { type: "timeout" }
    | { type: "response_too_big" }
    | { type: "server_error"; statusCode: number; response: ArrayBuffer }
    | { type: "other"; reason: string };

/**
 * HTTP request method.
 */
export type HttpRequestMethod =
    | "DELETE"
    | "GET"
    | "HEAD"
    | "POST";

/**
 * Response to an HTTP request.
 */
export type HttpResponse = {
    body: ArrayBuffer;
    headers: Record<string, string>;
    statusCode: number;
};

export type ImageCell = {
    id: string;
    fileId?: string;

    /**
     * Used to indicates the upload progress.
     * If file_id is set this shouldn't be set
     * Also: if no progress is set and no file_id exists
     * it means the cell is in the initial state (ready for upload)
     */
    progress?: number;
    readOnly?: boolean;
    width?: number;
    height?: number;

    /**
     * Will contain a hash to show as a preview for the image
     */
    preview?: string;

    /**
     * URL of the image if it was originally hosted on a remote server.
     * This will not be set if the image was uploaded through the
     * Fiberplane Studio.
     */
    url?: string;
};

/**
 * A single data-point in time, with meta-data about the metric it was taken from.
 */
export type Instant = {
    metric: Metric;
    point: Point;
};

/**
 * Defines a field that allows labels to be selected.
 */
export type LabelField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field (not to be confused with
     * labels to be selected).
     */
    label: string;

    /**
     * Whether multiple labels may be selected.
     */
    multiple: boolean;

    /**
     * Whether a value is required.
     */
    required: boolean;
};

/**
 * An individual log record
 */
export type LegacyLogRecord = {
    timestamp: Timestamp;
    body: string;
    attributes: Record<string, string>;
    resource: Record<string, string>;
    traceId?: ArrayBuffer;
    spanId?: ArrayBuffer;
};

/**
 * Legacy `ProviderRequest` from the Provider 1.0 protocol.
 */
export type LegacyProviderRequest =
    | { type: "instant" } & QueryInstant
    | { type: "series" } & QueryTimeRange
    | { type: "proxy" } & ProxyRequest
    /**
     * Requests a list of auto-suggestions. Note that these are
     * context-unaware.
     */
    | { type: "auto_suggest" }
    | { type: "logs" } & QueryLogs
    /**
     * Check data source status, any issue will be returned as `Error`
     */
    | { type: "status" };

/**
 * Legacy `ProviderResponse` from the 1.0 protocol.
 */
export type LegacyProviderResponse =
    | { type: "error"; error: Error }
    | { type: "instant"; instants: Array<Instant> }
    | { type: "series"; series: Array<Series> }
    | { type: "auto_suggest"; suggestions: Array<Suggestion> }
    | { type: "log_records"; logRecords: Array<LegacyLogRecord> }
    | { type: "status_ok" };

export type ListItemCell = {
    id: string;
    content: string;

    /**
     * Optional formatting to be applied to the cell's content.
     */
    formatting?: Formatting;
    listType: ListType;
    level?: number;
    readOnly?: boolean;
    startNumber?: number;
};

export type ListType =
    | "ordered"
    | "unordered";

export type LogCell = {
    id: string;
    readOnly?: boolean;
    sourceIds: Array<string>;

    /**
     * Optional formatting to be applied to the cell's title.
     */
    formatting?: Formatting;
    title: string;
    data?: Record<string, Array<LogRecord>>;
    timeRange?: TimeRange;
};

export type LogRecord = {
    timestamp: Timestamp;
    body: string;
    attributes: Record<string, string>;
    resource: Record<string, string>;
    traceId?: string;
    spanId?: string;
};

export type LokiCell = {
    id: string;
    content: string;
    readOnly?: boolean;
};

/**
 * Annotation for the mention of a user.
 *
 * Mentions do not have a start and end offset. Instead, they occur at the
 * start offset only and are expected to run up to the end of the name of
 * the mentioned user. If however, for unforeseen reasons, the plain text
 * being annotated does not align with the name inside the mention, the
 * mention will stop at the first non-matching character. Mentions for
 * which the first character of the name does not align must be ignored in
 * their entirety.
 */
export type Mention = {
    name: string;
    userId: string;
};

export type Metric = {
    name: string;
    labels: Record<string, string>;
};

/**
 * Defines a field that allows labels to be selected.
 *
 * Note that because the value is encoded as a string anyway, and depending on
 * the chosen `step` this field can be used for either integers or floating
 * point numbers, the values in the schema are simply presented as strings.
 */
export type NumberField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field.
     */
    label: string;

    /**
     * Optional maximum value to be selected.
     */
    max?: string;

    /**
     * Optional minimal value to be selected.
     */
    min?: string;

    /**
     * Whether a value is required.
     */
    required: boolean;

    /**
     * Specifies the granularity that any specified numbers must adhere to.
     *
     * If omitted, `step` defaults to "1", meaning only integers are allowed.
     */
    step?: string;
};

export type Point = {
    timestamp: Timestamp;
    value: number;
};

export type PrometheusCell = {
    id: string;
    content: string;
    readOnly?: boolean;
};

export type ProviderCell = {
    id: string;

    /**
     * The intent served by this provider cell.
     *
     * See: https://www.notion.so/fiberplane/RFC-45-Provider-Protocol-2-0-Revised-4ec85a0233924b2db0010d8cdae75e16#c8ed5dfbfd764e6bbd5c5b79333f9d6e
     */
    intent: string;

    /**
     * Query data encoded as "<mime-type>,<data>", where the MIME type is
     * either "application/x-www-form-urlencoded" or "multipart/form-data".
     * This is used for storing data for the Query Builder.
     *
     * Note: The format follows the specification for data URLs, without the
     *       `data:` prefix. See: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs
     */
    queryData?: string;

    /**
     * Optional response data from the provider.
     */
    response?: EncodedBlob;

    /**
     * Optional list of generated output cells.
     */
    output: Array<Cell>;

    /**
     * Optional title to assign the cell.
     */
    title: string;

    /**
     * Optional formatting to apply to the title.
     */
    formatting?: Formatting;
    readOnly?: boolean;
};

export type ProviderRequest = {
    /**
     * Query type that is part of the
     * [Intent](https://www.notion.so/fiberplane/RFC-45-Provider-Protocol-2-0-Revised-4ec85a0233924b2db0010d8cdae75e16#c8ed5dfbfd764e6bbd5c5b79333f9d6e)
     * through which the provider is invoked.
     */
    queryType: string;

    /**
     * Query data.
     *
     * This is usually populated from the [ProviderCell::query_data] field,
     * meaning the MIME type will be `"application/x-www-form-urlencoded"`
     * when produced by Studio's Query Builder.
     */
    queryData: Blob;

    /**
     * Configuration for the data source.
     */
    config: any;

    /**
     * Optional response from a previous invocation.
     * May be used for implementing things like filtering without additional
     * server roundtrip.
     */
    previousResponse?: Blob;
};

/**
 * Relays requests for a data-source to a proxy server registered with the API.
 */
export type ProxyRequest = {
    /**
     * ID of the proxy as known by the API.
     */
    proxyId: string;

    /**
     * Name of the data source exposed by the proxy.
     */
    dataSourceName: string;

    /**
     * Request data to send to the proxy
     */
    request: ArrayBuffer;
};

export type QueryField =
    | { type: "button" } & ButtonField
    | { type: "checkbox" } & CheckboxField
    | { type: "date" } & DateField
    | { type: "date_time" } & DateTimeField
    | { type: "date_time_range" } & DateTimeRangeField
    | { type: "file" } & FileField
    | { type: "label" } & LabelField
    | { type: "number" } & NumberField
    | { type: "select" } & SelectField
    | { type: "text" } & TextField;

export type QueryInstant = {
    query: string;
    timestamp: Timestamp;
};

export type QueryLogs = {
    query: string;
    limit?: number;
    timeRange: TimeRange;
};

export type QuerySchema = Array<QueryField>;

export type QueryTimeRange = {
    query: string;
    timeRange: TimeRange;
};

/**
 * A result that can be either successful (`Ok`) or represent an error (`Err`).
 */
export type Result<T, E> =
    /**
     * Represents a successful result.
     */
    | { Ok: T }
    /**
     * Represents an error.
     */
    | { Err: E };

/**
 * Defines a field that allows selection from a predefined list of options.
 *
 * Note that values to be selected from can be either hard-coded in the schema,
 * or fetched on-demand the same way as auto-suggestions.
 */
export type SelectField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the field.
     */
    label: string;

    /**
     * Whether multiple values may be selected.
     */
    multiple: boolean;

    /**
     * A list of options to select from. If empty, the auto-suggest mechanism
     * is used to fetch options as needed.
     */
    options: Array<string>;

    /**
     * An optional list of fields that should be filled in before allowing the
     * user to fill in this field. This forces a certain ordering in the data
     * entry, which enables richer auto-suggestions, as the filled in
     * prerequisite fields can provide additional context.
     */
    prerequisites: Array<string>;

    /**
     * Whether a value is required.
     */
    required: boolean;
};

/**
 * A series of data-points in time, with meta-data about the metric it was taken from.
 */
export type Series = {
    metric: Metric;
    points: Array<Point>;
    visible: boolean;
};

export type StackingType =
    | "none"
    | "stacked"
    | "percentage";

export type Suggestion = {
    /**
     * Suggested text.
     */
    text: string;

    /**
     * Optional description to go along with this suggestion.
     */
    description?: string;
};

/**
 * Defines which query types are supported by a provider.
 */
export type SupportedQueryType = {
    /**
     * The query type supported by the provider.
     *
     * There are predefined query types, such as "table" and "log", but
     * providers may also implement custom query types, which it should prefix
     * with "x-".
     */
    queryType: string;

    /**
     * The query schema defining the format of the `query_data` to be submitted
     * with queries of this type.
     */
    schema: QuerySchema;

    /**
     * MIME types supported for extraction. Any MIME type specified here should
     * be valid as an argument to `extract_data()` when passed a response from
     * queries of this type.
     *
     * E.g.:
     * ```
     * vec![
     *     "application/vnd.fiberplane.events",
     *     "application/vnd.fiberplane.metrics"
     * ];
     * ```
     */
    mimeTypes: Array<string>;
};

export type TableCell = {
    id: string;
    readOnly?: boolean;
    sourceIds: Array<string>;

    /**
     * Optional formatting to be applied to the cell's title.
     */
    formatting?: Formatting;
    title: string;
    data?: Record<string, Array<Instant>>;
};

export type TextCell = {
    id: string;
    content: string;

    /**
     * Optional formatting to be applied to the cell's content.
     */
    formatting?: Formatting;
    readOnly?: boolean;
};

/**
 * Defines a free-form text entry field.
 *
 * Is commonly used for filter text and query entry. For the latter case,
 * `supports_highlighting` can be set to `true` if the provider supports syntax
 * highlighting for the query language.
 */
export type TextField = {
    /**
     * Name of the field as it will be included in the encoded query.
     */
    name: string;

    /**
     * Suggested label to display along the form field.
     */
    label: string;

    /**
     * Suggests whether multi-line input is useful for this provider.
     */
    multiline: boolean;

    /**
     * An optional list of fields that should be filled in before allowing the
     * user to fill in this field. This forces a certain ordering in the data
     * entry, which enables richer auto-suggestions, as the filled in
     * prerequisite fields can provide additional context.
     */
    prerequisites: Array<string>;

    /**
     * Whether a value is required.
     */
    required: boolean;

    /**
     * Whether the provider implements syntax highlighting for this field.
     * See `highlight_field()` in the protocol definition.
     */
    supportsHighlighting: boolean;
};

/**
 * A range in time from a given timestamp (inclusive) up to another timestamp
 * (exclusive).
 */
export type TimeRange = {
    from: Timestamp;
    to: Timestamp;
};

export type Timestamp = number;
