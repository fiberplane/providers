use const_format::formatcp;
use fiberplane_pdk::providers::{
    NODE_GRAPH_MIME_TYPE, SUGGESTIONS_MIME_TYPE, TIMESERIES_MIME_TYPE,
};

pub const SUGGESTIONS_MSGPACK_MIME_TYPE: &str = formatcp!("{SUGGESTIONS_MIME_TYPE}+msgpack");
pub const TIMESERIES_MSGPACK_MIME_TYPE: &str = formatcp!("{TIMESERIES_MIME_TYPE}+msgpack");
pub const NODE_GRAPH_MSGPACK_MIME_TYPE: &str = formatcp!("{NODE_GRAPH_MIME_TYPE}+msgpack");

pub const QUERY_PARAM_NAME: &str = "query";
pub const FUNCTION_PARAM_NAME: &str = "function";
pub const DEPTH_PARAM_NAME: &str = "depth";

pub const ONE_MINUTE: u32 = 60; // seconds
pub const ONE_HOUR: u32 = 60 * ONE_MINUTE; // seconds
