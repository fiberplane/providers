pub mod auto_suggest;
pub mod describe_log_groups;
pub mod describe_queries;
pub mod get_log_record;
pub mod get_query_results;
pub mod graph_metric;
pub mod list_metrics;
pub mod start_query;
pub mod status;

use crate::constants::CELLS_MSGPACK_MIME_TYPE;
use fiberplane_pdk::prelude::{Blob, Cell, Error, Timestamp};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn serialize_cells(cells: Vec<Cell>) -> Result<Blob, Error> {
    Ok(Blob::builder()
        .data(rmp_serde::to_vec_named(&cells)?)
        .mime_type(CELLS_MSGPACK_MIME_TYPE.to_owned())
        .build())
}

pub fn try_from_iso_date(timestamp: &str) -> Result<Timestamp, Error> {
    OffsetDateTime::parse(timestamp, &Rfc3339)
        .map(Timestamp)
        .map_err(|err| Error::Deserialization {
            message: format!("could not deserialize timestamp '{timestamp}': {err}"),
        })
}
