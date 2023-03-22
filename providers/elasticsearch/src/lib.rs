mod config;
#[cfg(test)]
mod tests;

use config::ElasticConfig;
use elasticsearch_dsl::{Hit, SearchResponse};
use fiberplane_pdk::prelude::*;
use fiberplane_pdk::serde_json::{self, Map, Value};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const PAGE_SIZE: u32 = 30;

pub(crate) static TIMESTAMP_FIELDS: &[&str] = &["@timestamp", "timestamp", "fields.timestamp"];
pub(crate) static BODY_FIELDS: &[&str] =
    &["body", "message", "fields.body", "fields.message", "log"];

// This mapping is based on the recommended mapping from the Elastic Common
// Schema to the OpenTelemetry Log specification.
//
// See: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#elastic-common-schema
static RESOURCE_FIELD_PREFIXES: &[&str] = &["agent.", "cloud.", "container.", "host.", "service."];
static RESOURCE_FIELD_EXCEPTIONS: &[&str] = &["container.labels", "host.uptime", "service.state"];

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(Deserialize, QuerySchema)]
pub struct ElasticQuery {
    #[pdk(label = "Enter your Elasticsearch query")]
    pub query: String,

    #[pdk(label = "Specify a time range")]
    pub time_range: DateTimeRange,
}

#[derive(Serialize)]
struct SearchRequestBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size: Option<u32>,
}

pdk_query_types! {
    EVENTS_QUERY_TYPE => {
        label: "Elasticsearch query",
        handler: fetch_logs(ElasticQuery, ElasticConfig).await,
        supported_mime_types: [EVENTS_MIME_TYPE]
    },
    STATUS_QUERY_TYPE => {
        supported_mime_types: [STATUS_MIME_TYPE],
        handler: check_status(ProviderRequest).await
    }
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>> {
    log(format!("Creating cells for query type: {query_type}"));

    match query_type.as_str() {
        EVENTS_QUERY_TYPE => create_log_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
}

async fn fetch_logs(query: ElasticQuery, config: ElasticConfig) -> Result<Blob> {
    let mut url = config.parse_url()?;

    // Look for the timestamp and body first in the configured fields and then
    // in the default fields:
    let timestamp_field_names: Vec<_> = config
        .timestamp_field_names
        .iter()
        .map(String::as_str)
        .chain(TIMESTAMP_FIELDS.iter().copied())
        .collect();
    let body_field_names: Vec<_> = config
        .body_field_names
        .iter()
        .map(String::as_str)
        .chain(BODY_FIELDS.iter().copied())
        .collect();

    // Add "_search" to the path
    {
        let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
            message: "Invalid Elasticsearch URL: Not a base URL".to_owned(),
        })?;
        path_segments.push("_search");
    }

    let mut headers = BTreeMap::from([("Content-Type".to_owned(), "application/json".to_owned())]);
    if let Some(api_key) = config.api_key {
        headers.insert("Authorization".to_owned(), format!("ApiKey {api_key}"));
    }

    // Lucene Query Syntax: https://www.elastic.co/guide/en/kibana/current/lucene-query.html
    // TODO should we determine timestamp field name from API?
    let query_string = format!(
        "{} AND {}:[{} TO {}]",
        query.query, timestamp_field_names[0], query.time_range.from, query.time_range.to
    );

    let body = SearchRequestBody {
        size: Some(PAGE_SIZE),
    };
    url.set_query(Some(&format!("q={}", &query_string)));

    let body = serde_json::to_vec(&body).map_err(|err| Error::Data {
        message: format!("Error serializing query: {err:?}"),
    })?;
    let request = HttpRequest::post(url, body).with_headers(headers);

    // Parse response
    let response = make_http_request(request).await?;
    let response: SearchResponse =
        serde_json::from_slice(&response.body).map_err(|err| Error::Data {
            message: format!("Error parsing Elasticsearch response: {err:?}"),
        })?;

    if response.timed_out {
        return Err(Error::Other {
            message: "Elasticsearch query timed out".to_owned(),
        });
    }

    let num_hits = response.hits.hits.len();
    log(format!("Got {num_hits} query results from Elasticsearch"));

    Events(parse_response(
        response,
        &timestamp_field_names,
        &body_field_names,
    ))
    .to_blob()
}

fn parse_response(
    response: SearchResponse,
    timestamp_field_names: &[&str],
    body_field_names: &[&str],
) -> Vec<ProviderEvent> {
    let hits = response.hits.hits.into_iter();
    let mut log_lines: Vec<ProviderEvent> = hits
        .filter_map(|hit| parse_hit(hit, timestamp_field_names, body_field_names))
        .collect();
    // Sort logs so the newest ones are first
    log_lines.sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap_or(Ordering::Equal));
    log_lines
}

fn parse_hit(
    hit: Hit,
    timestamp_field_names: &[&str],
    body_field_names: &[&str],
) -> Option<ProviderEvent> {
    let source: Map<String, Value> = hit
        .source()
        .map_err(|err| {
            log(format!(
                "Error parsing ElasticSearch hit as JSON object: {err:?}"
            ));
        })
        .ok()?;
    let mut flattened_fields = BTreeMap::new();
    for (key, val) in source.into_iter() {
        flatten_nested_value(&mut flattened_fields, key, val);
    }

    // Parse the trace ID and span ID from hex if they exist
    let trace_id = flattened_fields.remove("trace.id").and_then(|trace_id| {
        if let Ok(bytes) = hex::decode(trace_id.to_string().replace('-', "")) {
            bytes.try_into().ok().map(OtelTraceId::new)
        } else {
            log(format!("unable to decode ID as hex in log: {trace_id}"));
            // Put the value back if we were unable to parse it
            flattened_fields.insert("trace.id".to_owned(), trace_id);
            None
        }
    });
    let span_id = flattened_fields.remove("span.id").and_then(|span_id| {
        if let Ok(bytes) = hex::decode(span_id.to_string().replace('-', "")) {
            bytes.try_into().ok().map(OtelSpanId::new)
        } else {
            log(format!("unable to decode ID as hex in log: {span_id}"));
            // Put the value back if we were unable to parse it
            flattened_fields.insert("span.id".to_owned(), span_id);
            None
        }
    });

    // Find the timestamp field (or set it to the Epoch if none is found)
    // Note: this will leave the original timestamp field in the flattened_fields
    let timestamp = timestamp_field_names
        .iter()
        .flat_map(|field_name| flattened_fields.get(*field_name))
        .flat_map(|value| {
            // Try parsing the field either as an RFC 3339 timestamp or a UNIX timestamp
            match value {
                Value::String(string) => OffsetDateTime::parse(string, &Rfc3339).ok(),
                Value::Number(number) => number.as_f64().and_then(|seconds| {
                    OffsetDateTime::from_unix_timestamp_nanos((seconds * 1_000_000_000.0) as i128)
                        .ok()
                }),
                _ => None,
            }
        })
        .next()
        .unwrap_or(OffsetDateTime::UNIX_EPOCH);

    // Find the body field (or set it to an empty string if none is found)
    // Note: this will leave the body field in the flattened_fields and copy
    // it into the title of the Event
    let title = body_field_names
        .iter()
        .flat_map(|field_name| flattened_fields.get(*field_name))
        .next()
        .map(|title| match title {
            Value::String(title) => title.clone(),
            Value::Null => "".to_owned(),
            other => other.to_string(),
        })
        .unwrap_or_default();

    // All fields that are not mapped to the resource field
    // become part of the attributes field
    // TODO refactor this so we only make one pass over the fields
    let (resource, attributes): (BTreeMap<String, Value>, BTreeMap<String, Value>) =
        flattened_fields.into_iter().partition(|(key, _)| {
            RESOURCE_FIELD_PREFIXES
                .iter()
                .any(|prefix| key.starts_with(prefix))
                && !RESOURCE_FIELD_EXCEPTIONS.contains(&key.as_str())
        });

    let mut otel = OtelMetadata::builder()
        .attributes(attributes)
        .resource(resource)
        .build();
    otel.trace_id = trace_id;
    otel.span_id = span_id;

    let event = ProviderEvent::builder()
        .title(title)
        .time(timestamp)
        .otel(otel)
        .build();

    Some(event)
}

fn flatten_nested_value(output: &mut BTreeMap<String, Value>, key: String, value: Value) {
    match value {
        Value::Object(v) => {
            for (sub_key, val) in v.into_iter() {
                flatten_nested_value(output, format!("{key}.{sub_key}"), val);
            }
        }
        Value::Array(v) => {
            for (index, val) in v.into_iter().enumerate() {
                // TODO should the separator be dots instead?
                flatten_nested_value(output, format!("{key}[{index}]"), val);
            }
        }
        primitive => {
            output.insert(key, primitive);
        }
    };
}

async fn check_status(request: ProviderRequest) -> Result<Blob> {
    let config = ElasticConfig::parse(request.config)?;
    let mut url = config.parse_url()?;

    // Add "_xpack" to the path
    {
        let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
            message: "Invalid ElasticSearch URL: cannot-be-a-base".to_string(),
        })?;
        path_segments.push("_xpack");
    }

    let request = HttpRequest::get(url)
        .with_headers([("Content-Type".to_owned(), "application/json".to_owned())]);

    // At this point we don't care to validate the info Loki sends back.
    // We just care it responded with 200 OK.
    let _ = make_http_request(request).await?;

    ProviderStatus::builder()
        .status(Ok(()))
        .version(COMMIT_HASH.to_owned())
        .built_at(BUILD_TIMESTAMP.to_owned())
        .build()
        .to_blob()
}

pub fn create_log_cell() -> Result<Vec<Cell>> {
    let logs_cell = Cell::Log(
        LogCell::builder()
            .id("query-results".to_string())
            .data_links(vec![format!("cell-data:{EVENTS_MIME_TYPE},self")])
            .hide_similar_values(false)
            .build(),
    );
    Ok(vec![logs_cell])
}
