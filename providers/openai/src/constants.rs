use const_format::formatcp;

pub use fiberplane_pdk::prelude::{CELLS_MIME_TYPE, STATUS_QUERY_TYPE};

pub const PERFORM_QUERY_TYPE: &str = "x-openai-query";

pub const PROMPT_PARAM_NAME: &str = "param";

pub const CELLS_MSGPACK_MIME_TYPE: &str = formatcp!("{CELLS_MIME_TYPE}+msgpack");
