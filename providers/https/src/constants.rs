use const_format::formatcp;

pub use fiberplane_pdk::prelude::{CELLS_MIME_TYPE, STATUS_QUERY_TYPE};

pub const PERFORM_QUERY_TYPE: &str = "x-https-query";

pub const PATH_PARAM_NAME: &str = "path";
pub const HTTP_METHOD_PARAM_NAME: &str = "http-method";

pub const CELLS_MSGPACK_MIME_TYPE: &str = formatcp!("{CELLS_MIME_TYPE}+msgpack");
