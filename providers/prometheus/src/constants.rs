use const_format::formatcp;
use fiberplane_pdk::providers::{SUGGESTIONS_MIME_TYPE, TIMESERIES_MIME_TYPE};

pub const SUGGESTIONS_MSGPACK_MIME_TYPE: &str = formatcp!("{SUGGESTIONS_MIME_TYPE}+msgpack");
pub const TIMESERIES_MSGPACK_MIME_TYPE: &str = formatcp!("{TIMESERIES_MIME_TYPE}+msgpack");

pub const QUERY_PARAM_NAME: &str = "query";
pub const TIME_RANGE_PARAM_NAME: &str = "time_range";
pub const LIVE_PARAM_NAME: &str = "live";

pub const ONE_MINUTE: u32 = 60; // seconds
pub const ONE_HOUR: u32 = 60 * ONE_MINUTE; // seconds
