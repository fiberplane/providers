//! Describe Log Groups query handling
use crate::{
    api::cloudwatch_logs::{QueryStatistics, QueryStatus, ResultField},
    client::cloudwatch_logs::Client,
    config::Config,
    constants::{
        BODY_KEY, EVENTS_JSON_MIME_TYPE, EVENTS_MIME_TYPE, EVENTS_MSGPACK_MIME_TYPE, LOG_KEY,
        PTR_KEY, QUERY_ID_PARAM_NAME, QUERY_RESULTS_MIME_TYPE, SPAN_KEY, TRACE_KEY, TS_KEY,
    },
};
use fiberplane_pdk::prelude::{Blob, Cell, Error, LogCell, ProviderRequest, TextCell};
use fiberplane_pdk::providers::{
    OtelMetadata, OtelSpanId, OtelTraceId, ProviderEvent, FORM_ENCODED_MIME_TYPE,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime};

pub async fn invoke2_handler(config: Config, request: ProviderRequest) -> Result<Blob, Error> {
    let request: QueryResultsInput = request.query_data.try_into()?;
    let client = Client::from(&config);

    // Short version that doesn't fetch everything
    let res = client
        .get_query_results(request.query_id)
        .await
        .map_err(|e| Error::Invocation {
            message: format!("failed to list metrics: {e}"),
        })?;

    let events = LogLines(
        res.results
            .into_iter()
            .map(LogLines::bare_event_from_response)
            .collect(),
    );

    QueryResults {
        events,
        query_status: res.status,
        query_statistics: res.statistics,
    }
    .try_into()
}

pub fn create_cells_handler(response: Blob) -> Result<Vec<Cell>, Error> {
    fn display_f64(opt: Option<f64>) -> String {
        match opt {
            Some(val) => val.to_string(),
            None => "N/A".to_string(),
        }
    }

    let results: QueryResults = response.try_into()?;
    let status_cell = Cell::Text(
        TextCell::builder()
            .id("query-status".to_string())
            .content(format!(
                "Query status: {}",
                match results.query_status {
                    QueryStatus::Scheduled =>
                        "scheduled (Hit \"Run\" again to obtain more results)",
                    QueryStatus::Running => "running (Hit \"Run\" again to obtain more results)",
                    QueryStatus::Complete => "complete",
                    QueryStatus::Failed => "failed",
                    QueryStatus::Cancelled => "cancelled",
                    QueryStatus::Timeout => "timed out",
                    QueryStatus::Unknown => "unknown (Hit \"Run\" again to obtain more results)",
                }
            ))
            .formatting(Vec::new())
            .read_only(true)
            .build(),
    );
    let statistics_cell = Cell::Text(
        TextCell::builder()
            .id("query-statistics".to_string())
            .content(format!(
                "Query statistics:\n\tRecords: {} matched / {} scanned\n\t(Bytes scanned: {})",
                display_f64(results.query_statistics.records_matched),
                display_f64(results.query_statistics.records_scanned),
                display_f64(results.query_statistics.bytes_scanned),
            ))
            .formatting(Vec::new())
            .read_only(true)
            .build(),
    );
    let logs_cell = Cell::Log(
        LogCell::builder()
            .id("query-results".to_string())
            .data_links(vec![format!("cell-data:{EVENTS_MIME_TYPE},self")])
            .build(),
    );

    Ok(vec![status_cell, statistics_cell, logs_cell])
}

pub fn extract_data_handler(
    response: Blob,
    mime_type: String,
    _query: Option<String>,
) -> Result<Blob, Error> {
    let response: QueryResults = response.try_into()?;

    if mime_type == EVENTS_MIME_TYPE {
        return Blob::try_from(response.events);
    }

    Err(Error::UnsupportedRequest)
}

struct QueryResultsInput {
    query_id: String,
}

impl TryFrom<Blob> for QueryResultsInput {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type != FORM_ENCODED_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        let mut query_id = String::new();
        for (key, value) in form_urlencoded::parse(&blob.data) {
            if let QUERY_ID_PARAM_NAME = key.as_ref() {
                query_id = value.to_string()
            }
        }

        Ok(Self { query_id })
    }
}

#[derive(Serialize, Deserialize)]
struct QueryResults {
    events: LogLines,
    query_status: QueryStatus,
    query_statistics: QueryStatistics,
}

impl TryFrom<QueryResults> for Blob {
    type Error = Error;

    fn try_from(value: QueryResults) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .data(rmp_serde::to_vec_named(&value)?)
            .mime_type(format!("{QUERY_RESULTS_MIME_TYPE}+msgpack"))
            .build())
    }
}

impl TryFrom<Blob> for QueryResults {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type == format!("{QUERY_RESULTS_MIME_TYPE}+msgpack") {
            Ok(
                rmp_serde::from_slice(&blob.data).map_err(|e| Error::Deserialization {
                    message: format!("could not deserialize query results: {e}"),
                })?,
            )
        } else if blob.mime_type == format!("{QUERY_RESULTS_MIME_TYPE}+json") {
            Ok(
                serde_json::from_slice(&blob.data).map_err(|e| Error::Deserialization {
                    message: format!("could not deserialize query results: {e}"),
                })?,
            )
        } else {
            Err(Error::UnsupportedRequest)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LogLines(Vec<ProviderEvent>);

impl LogLines {
    fn bare_event_from_response(entry: Vec<ResultField>) -> ProviderEvent {
        let kv: std::collections::HashMap<String, String> = entry
            .into_iter()
            .filter_map(|field| {
                if let (Some(key), Some(value)) = (field.field, field.value) {
                    Some((key, value))
                } else {
                    None
                }
            })
            .collect();

        let time = kv
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

        let title = kv
            .get(PTR_KEY.0)
            .map(ToString::to_string)
            .unwrap_or_default();

        let labels: BTreeMap<String, String> = kv
            .iter()
            .filter(|(k, _)| *k != TS_KEY.0)
            .fold(Vec::new(), |mut acc, (k, v)| {
                if *k == BODY_KEY.0 {
                    if let Ok(Ok(serde_json::Value::Object(flattened))) = serde_json::from_str(v)
                        .map(|document| {
                            use flatten_json_object::ArrayFormatting;
                            use flatten_json_object::Flattener;
                            Flattener::new()
                                .set_key_separator(".")
                                .set_array_formatting(ArrayFormatting::Surrounded {
                                    start: "[".to_string(),
                                    end: "]".to_string(),
                                })
                                .set_preserve_empty_arrays(false)
                                .set_preserve_empty_objects(false)
                                .flatten(&document)
                        })
                    {
                        acc.extend(flattened.iter().map(|(flattened_k, flattened_v)| {
                            (
                                flattened_k.clone(),
                                serde_json::to_string(flattened_v).unwrap(),
                            )
                        }));
                    }
                }
                // We also push the original BODY_KEY value in case the user
                // wants to do copy-paste/manual parsing
                acc.push((k.clone(), v.clone()));
                acc
            })
            .into_iter()
            .collect();

        let otel = {
            let resource = match kv.get(LOG_KEY.0) {
                Some(val) => [(LOG_KEY.0.to_string(), val.to_string().into())].into(),
                None => Default::default(),
            };
            let attributes = labels
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string().into()))
                .collect();

            let mut otel = OtelMetadata::builder()
                .resource(resource)
                .attributes(attributes)
                .build();
            otel.trace_id = kv.get(TRACE_KEY.0).map(|t_id| {
                OtelTraceId::new(
                    t_id.as_bytes()[0..16]
                        .try_into()
                        .expect("OtelSpanId wraps a [u8; 16]"),
                )
            });
            otel.span_id = kv.get(SPAN_KEY.0).map(|s_id| {
                OtelSpanId::new(
                    s_id.as_bytes()[0..8]
                        .try_into()
                        .expect("OtelSpanId wraps a [u8; 8]"),
                )
            });
            otel
        };

        let mut event = ProviderEvent::builder()
            .time(time)
            .otel(otel)
            .title(title)
            .labels(labels)
            .build();

        event.description = kv.get(BODY_KEY.0).map(ToString::to_string);

        event
    }
}

impl TryFrom<Blob> for LogLines {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type == EVENTS_MSGPACK_MIME_TYPE {
            Ok(Self(
                rmp_serde::from_slice::<Vec<ProviderEvent>>(&blob.data).map_err(|e| {
                    Error::Deserialization {
                        message: format!("could not deserialize log events: {e}"),
                    }
                })?,
            ))
        } else if blob.mime_type == EVENTS_JSON_MIME_TYPE {
            Ok(Self(
                serde_json::from_slice::<Vec<ProviderEvent>>(&blob.data).map_err(|e| {
                    Error::Deserialization {
                        message: format!("could not deserialize log events: {e}"),
                    }
                })?,
            ))
        } else {
            Err(Error::UnsupportedRequest)
        }
    }
}

impl TryFrom<LogLines> for Blob {
    type Error = Error;

    fn try_from(value: LogLines) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .data(rmp_serde::to_vec_named(&value.0)?)
            .mime_type(EVENTS_MSGPACK_MIME_TYPE.to_owned())
            .build())
    }
}
