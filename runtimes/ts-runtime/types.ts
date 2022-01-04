// ============================================= //
// Types for WebAssembly runtime                 //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

export type Config = {
    url?: string;
};

export type Error =
    | { type: "unsupported_request" }
    | { type: "http"; error: HttpRequestError }
    | { type: "data"; message: string }
    | { type: "deserialization"; message: string }
    | { type: "config"; message: string }
    | { type: "other"; message: string };

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
    | { type: "server_error"; statusCode: number;response: ArrayBuffer }
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

/**
 * A single data point in time, with meta-data about the metric it was taken
 * from.
 */
export type Instant = {
    metric: Metric;
    point: Point;
};

/**
 * An individual log record
 */
export type LogRecord = {
    timestamp: Timestamp;
    body: string;
    attributes: Record<string, string>;
    resource: Record<string, string>;
    traceId?: ArrayBuffer;
    spanId?: ArrayBuffer;
};

/**
 * Meta-data about a metric.
 */
export type Metric = {
    name: string;
    labels: Record<string, string>;
};

/**
 * A single data-point in time.
 */
export type Point = {
    timestamp: Timestamp;
    value: number;
};

export type ProviderRequest =
    | { type: "instant" } & QueryInstant
    | { type: "series" } & QueryTimeRange
    | { type: "proxy" } & ProxyRequest
    /**
     * Requests a list of auto-suggestions. Note that these are
     * context-unaware.
     */
    | { type: "auto_suggest" }
    | { type: "logs" } & QueryLogs;

export type ProviderResponse =
    | { type: "error"; error: Error }
    | { type: "instant"; instants: Array<Instant> }
    | { type: "series"; series: Array<Series> }
    | { type: "auto_suggest"; suggestions: Array<Suggestion> }
    | { type: "log_records"; logRecords: Array<LogRecord> };

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

export type QueryInstant = {
    query: string;
    timestamp: Timestamp;
};

export type QueryLogs = {
    query: string;
    limit?: number;
    timeRange: TimeRange;
};

export type QueryTimeRange = {
    query: string;
    timeRange: TimeRange;
};

/**
 * A result that can be either successful (`Ok)` or represent an error (`Err`).
 */
export type Result<T, E> =
    /**
     * Represents a succesful result.
     */
    | { Ok: T }
    /**
     * Represents an error.
     */
    | { Err: E };

/**
 * A series of data-points in time, with meta-data about the metric it was
 * taken from.
 */
export type Series = {
    metric: Metric;
    points: Array<Point>;
};

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
 * A range in time from a given timestamp (inclusive) up to another timestamp
 * (exclusive).
 */
export type TimeRange = {
    from: Timestamp;
    to: Timestamp;
};

export type Timestamp = number;
