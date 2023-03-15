//! Describe Log Groups query handling
use crate::{
    client::cloudwatch_logs::Client,
    config::Config,
    constants::{
        BODY_KEY, EVENTS_MIME_TYPE, EVENTS_MSGPACK_MIME_TYPE, INGESTION_TS_KEY, LOG_KEY,
        LOG_RECORD_POINTER_PARAM_NAME, SPAN_KEY, TRACE_KEY, TS_KEY,
    },
};
use fiberplane_pdk::prelude::{Blob, Cell, Error, LogCell, ProviderRequest};
use fiberplane_pdk::providers::{
    OtelMetadata, OtelSpanId, OtelTraceId, ProviderEvent, FORM_ENCODED_MIME_TYPE,
};
use std::collections::{BTreeMap, HashMap};
use time::{
    format_description::well_known::Rfc3339, macros::format_description, OffsetDateTime,
    PrimitiveDateTime,
};

pub async fn invoke2_handler(config: Config, request: ProviderRequest) -> Result<Blob, Error> {
    let request: LogRecordInput = request.query_data.try_into()?;
    let client = Client::from(&config);

    client
        .get_log_details(request.log_record_pointer, None)
        .await
        .and_then(try_into_blob)
}

pub fn convert_log_entry_to_event(res: HashMap<String, String>) -> Result<ProviderEvent, Error> {
    let time = res
        .get(TS_KEY.0)
        .and_then(|x| {
            if let Ok(ts) = x.parse::<f64>() {
                // ts is in milliseconds on AWS products. It's not "official" as in "a single page on AWS documentation says so",
                // but rather informal as in "every page we've seen mentioning timestamps states milliseconds".
                let nanos: i128 = (ts * 1_000_000.0).round() as i128;
                OffsetDateTime::from_unix_timestamp_nanos(nanos).ok()
            } else {
                PrimitiveDateTime::parse(
                    x,
                    &format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"
                    ),
                )
                .map(PrimitiveDateTime::assume_utc)
                .ok()
            }
        })
        .unwrap_or_else(OffsetDateTime::now_utc);
    let description = res.get(LOG_KEY.0).map(ToString::to_string);
    let title = String::new();
    let labels: BTreeMap<String, String> = res
        .iter()
        .filter_map(|(k, v)| {
            // Properly format ingestion time
            if *k == INGESTION_TS_KEY.0 {
                let ingestion_time = v.parse::<f64>().ok().and_then(|ts| {
                    // ts is in microseconds on AWS products. It's not "official" as in "a single page on AWS documentation says so",
                    // but rather informal as in "every page we've seen mentioning timestamps states microseconds".
                    let nanos: i128 = (ts * 1_000.0).round() as i128;
                    OffsetDateTime::from_unix_timestamp_nanos(nanos).ok()
                });
                let new_v =
                    ingestion_time.map_or_else(|| v.to_string(), |ts| ts.format(&Rfc3339).unwrap());
                Some((k.to_string(), new_v))
            } else if *k != TS_KEY.0 && *k != BODY_KEY.0 {
                Some((k.to_string(), v.to_string()))
            } else {
                None
            }
        })
        .collect();
    let resource = match res.get(LOG_KEY.0) {
        Some(val) => [(LOG_KEY.0.to_string(), val.to_string().into())].into(),
        None => Default::default(),
    };
    let trace_id = res.get(TRACE_KEY.0).map(|t_id| {
        OtelTraceId::new(
            t_id.as_bytes()[0..16]
                .try_into()
                .expect("OtelSpanId wraps a [u8; 16]"),
        )
    });
    let span_id = res.get(SPAN_KEY.0).map(|s_id| {
        OtelSpanId::new(
            s_id.as_bytes()[0..8]
                .try_into()
                .expect("OtelSpanId wraps a [u8; 8]"),
        )
    });
    let otel = OtelMetadata::builder()
        .resource(resource)
        .trace_id(trace_id)
        .span_id(span_id)
        .attributes(
            labels
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string().into()))
                .collect(),
        )
        .build();
    Ok(ProviderEvent::builder()
        .time(time.into())
        .end_time(None)
        .otel(otel)
        .title(title)
        .description(description)
        .severity(None)
        .labels(labels)
        .build())
}

fn try_into_blob(res: HashMap<String, String>) -> Result<Blob, Error> {
    let event = vec![convert_log_entry_to_event(res)?];
    Ok(Blob::builder()
        .data(rmp_serde::to_vec_named(&event)?)
        .mime_type(EVENTS_MSGPACK_MIME_TYPE.to_owned())
        .build())
}

pub fn create_cells_handler(_response: Blob) -> Result<Vec<Cell>, Error> {
    let logs_cell = Cell::Log(
        LogCell::builder()
            .id("query-results".to_string())
            .data_links(vec![format!("cell-data:{EVENTS_MIME_TYPE},self")])
            .hide_similar_values(false)
            .build(),
    );

    Ok(vec![logs_cell])
}

struct LogRecordInput {
    log_record_pointer: String,
}

impl TryFrom<Blob> for LogRecordInput {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type != FORM_ENCODED_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        let mut log_record_pointer = String::new();
        for (key, value) in form_urlencoded::parse(&blob.data) {
            if let LOG_RECORD_POINTER_PARAM_NAME = key.as_ref() {
                log_record_pointer = value.to_string()
            }
        }

        Ok(Self { log_record_pointer })
    }
}
