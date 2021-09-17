export type FetchError =
    | { type: "request_error"; payload: RequestError }
    | { type: "data_error"; message: string }
    | { type: "other"; message: string };

export type Instant = {
    metric: Metric;
    point: Point;
};

export type Metric = {
    name: string;
    labels: Record<string, string>;
};

export type Point = {
    timestamp: Timestamp;
    value: number;
};

export type QueryInstantOptions = {
    time: Timestamp;
};

export type QuerySeriesOptions = {
    timeRange: TimeRange;
};

export type Request = {
    url: string;
    method: RequestMethod;
    headers?: Record<string, string>;
    body?: ArrayBuffer;
};

export type RequestError =
    | { type: "offline" }
    | { type: "no_route" }
    | { type: "connection_refused" }
    | { type: "timeout" }
    | { type: "server_error"; statusCode: number; response: ArrayBuffer }
    | { type: "other"; reason: string };

export type RequestMethod =
    | "DELETE"
    | "GET"
    | "HEAD"
    | "POST";

export type Response = {
    body: ArrayBuffer;
    headers: Record<string, string>;
    statusCode: number;
};

export type Result<T, E> =
    | { Ok: T }
    | { Err: E };

export type Series = {
    metric: Metric;
    points: Array<Point>;
    visible: boolean;
};

export type TimeRange = {
    from: Timestamp;
    to: Timestamp;
};

export type Timestamp = number;
