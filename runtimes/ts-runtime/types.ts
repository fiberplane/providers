// ============================================= //
// Types for WebAssembly runtime                 //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

/**
 * A data-source represents all the configuration for a specific component or
 * service. It will be used by provider.
 */
export type DataSource =
    | { Prometheus: PrometheusDataSource };

export type FetchError =
    | { type: "request_error"; payload: RequestError }
    | { type: "data_error"; message: string }
    | { type: "other"; message: string };

/**
 * A single data point in time, with meta-data about the metric it was taken
 * from.
 */
export type Instant = {
    metric: Metric;
    point: Point;
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

/**
 * A data-source for Prometheus. Currently only requires a url. This should be
 * a full URL starting with http:// or https:// the domain, and optionally a
 * port and a path.
 */
export type PrometheusDataSource = {
    url: string;
};

/**
 * Options to specify which instant should be fetched.
 */
export type QueryInstantOptions = {
    dataSource: DataSource;
    time: Timestamp;
};

/**
 * Options to specify what series should be fetched.
 */
export type QuerySeriesOptions = {
    dataSource: DataSource;
    timeRange: TimeRange;
};

/**
 * HTTP request options.
 */
export type Request = {
    url: string;
    method: RequestMethod;
    headers?: Record<string, string>;
    body?: ArrayBuffer;
};

/**
 * Possible errors that may happen during an HTTP request.
 */
export type RequestError =
    | { type: "offline" }
    | { type: "no_route" }
    | { type: "connection_refused" }
    | { type: "timeout" }
    | { type: "server_error"; statusCode: number;response: ArrayBuffer }
    | { type: "other"; reason: string };

/**
 * HTTP request method.
 */
export type RequestMethod =
    | "DELETE"
    | "GET"
    | "HEAD"
    | "POST";

/**
 * Response to an HTTP request.
 */
export type Response = {
    body: ArrayBuffer;
    headers: Record<string, string>;
    statusCode: number;
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

/**
 * A range in time from a given timestamp (inclusive) up to another timestamp
 * (exclusive).
 */
export type TimeRange = {
    from: Timestamp;
    to: Timestamp;
};

export type Timestamp = number;
