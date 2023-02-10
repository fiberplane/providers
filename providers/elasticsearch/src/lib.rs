use elasticsearch_dsl::{Hit, SearchResponse};
use fiberplane_pdk::prelude::*;
use fiberplane_provider_bindings::{
    LegacyLogRecord as LogRecord, LegacyProviderRequest as ProviderRequest,
    LegacyProviderResponse as ProviderResponse, QueryLogs,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{cmp::Ordering, collections::HashMap, str::FromStr};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use url::Url;

#[cfg(test)]
mod tests;

pub(crate) static TIMESTAMP_FIELDS: &[&str] = &["@timestamp", "timestamp", "fields.timestamp"];
pub(crate) static BODY_FIELDS: &[&str] =
    &["body", "message", "fields.body", "fields.message", "log"];
// This mapping is based on the recommended mapping from the
// Elastic Common Schema to the OpenTelemetry Log specification
// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#elastic-common-schema
static RESOURCE_FIELD_PREFIXES: &[&str] = &["agent.", "cloud.", "container.", "host.", "service."];
static RESOURCE_FIELD_EXCEPTIONS: &[&str] = &["container.labels", "host.uptime", "service.state"];
static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    url: Url,
    #[serde(default)]
    timestamp_field_names: Vec<String>,
    #[serde(default)]
    body_field_names: Vec<String>,
    api_key: Option<String>,
}

#[derive(Serialize)]
struct SearchRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u32>,
}

#[pdk_export]
async fn invoke(request: ProviderRequest, config: ProviderConfig) -> ProviderResponse {
    log(format!(
        "Elasticsearch provider (commit: {COMMIT_HASH}, built at: {BUILD_TIMESTAMP}) invoked with request: {request:?}"
    ));

    let config: Config = match serde_json::from_value(config) {
        Ok(config) => config,
        Err(err) => {
            return ProviderResponse::Error {
                error: Error::Config {
                    message: format!("Error parsing config: {err:?}"),
                },
            }
        }
    };
    match request {
        // TODO implement AutoSuggest
        ProviderRequest::Logs(query) => fetch_logs(query, config)
            .await
            .map(|log_records| ProviderResponse::LogRecords { log_records })
            .unwrap_or_else(|error| ProviderResponse::Error { error }),
        ProviderRequest::Status => check_status(config)
            .await
            .map(|_| ProviderResponse::StatusOk)
            .unwrap_or_else(|error| ProviderResponse::Error { error }),
        _ => ProviderResponse::Error {
            error: Error::UnsupportedRequest,
        },
    }
}

async fn fetch_logs(query: QueryLogs, config: Config) -> Result<Vec<LogRecord>> {
    let mut url = config.url;

    // Look for the timestamp and body first in the configured fields and then in the default fields
    let timestamp_field_names = config
        .timestamp_field_names
        .iter()
        .map(|s| s.as_str())
        .chain(TIMESTAMP_FIELDS.iter().copied())
        .collect::<Vec<_>>();
    let body_field_names = config
        .body_field_names
        .iter()
        .map(|s| s.as_str())
        .chain(BODY_FIELDS.iter().copied())
        .collect::<Vec<_>>();

    // Add "_search" to the path
    let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
        message: "Invalid ElasticSearch URL".to_string(),
    })?;
    path_segments.push("_search");
    drop(path_segments);

    let mut headers = HashMap::new();
    if let Some(api_key) = config.api_key {
        headers.insert("Authorization".to_string(), format!("ApiKey {api_key}"));
    }
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    // Lucene Query Syntax: https://www.elastic.co/guide/en/kibana/current/lucene-query.html
    // TODO should we determine timestamp field name from API?
    let query_string = format!(
        "{} AND {}:[{} TO {}]",
        query.query,
        timestamp_field_names[0],
        timestamp_to_rfc3339(query.time_range.from),
        timestamp_to_rfc3339(query.time_range.to)
    );

    let body = SearchRequestBody { size: query.limit };
    url.set_query(Some(&format!("q={}", &query_string)));

    let request = HttpRequest::builder()
        .body(Some(
            serde_json::to_vec(&body)
                .map_err(|e| Error::Data {
                    message: format!("Error serializing query: {e:?}"),
                })?
                .into(),
        ))
        .headers(Some(headers))
        .method(HttpRequestMethod::Post)
        .url(url.to_string())
        .build();

    // Parse response
    let response = make_http_request(request).await?.body;
    let response: SearchResponse = serde_json::from_slice(&response).map_err(|e| Error::Data {
        message: format!("Error parsing ElasticSearch response: {e:?}"),
    })?;

    if response.timed_out {
        return Err(Error::Other {
            message: "ElasticSearch query timed out".to_string(),
        });
    }

    log(format!(
        "Got {} query results from Elasticsearch",
        response.hits.hits.len()
    ));

    parse_response(response, &timestamp_field_names, &body_field_names)
}

fn parse_response(
    response: SearchResponse,
    timestamp_field_names: &[&str],
    body_field_names: &[&str],
) -> Result<Vec<LogRecord>> {
    let hits = response.hits.hits.into_iter();
    let mut logs: Vec<LogRecord> = hits
        .filter_map(|hit| parse_hit(hit, timestamp_field_names, body_field_names))
        .collect();
    // Sort logs so the newest ones are first
    logs.sort_by(|a, b| {
        b.timestamp
            .partial_cmp(&a.timestamp)
            .unwrap_or(Ordering::Equal)
    });
    Ok(logs)
}

fn parse_hit(
    hit: Hit,
    timestamp_field_names: &[&str],
    body_field_names: &[&str],
) -> Option<LogRecord> {
    let source: Map<String, Value> = hit
        .source()
        .map_err(|err| {
            log(format!(
                "Error parsing ElasticSearch hit as JSON object: {err:?}"
            ));
        })
        .ok()?;
    let mut flattened_fields = HashMap::new();
    for (key, val) in source.into_iter() {
        flatten_nested_value(&mut flattened_fields, key, val);
    }

    // Parse the trace ID and span ID from hex if they exist
    let mut parse_id = |key: &str| {
        if let Some((key, val)) = flattened_fields.remove_entry(key) {
            if let Ok(bytes) = hex::decode(val.to_string().replace('-', "")) {
                Some(bytes.into())
            } else {
                log(format!("unable to decode ID as hex in log: {val}"));
                // Put the value back if we were unable to parse it
                flattened_fields.insert(key, val);
                None
            }
        } else {
            None
        }
    };
    let trace_id = parse_id("trace.id");
    let span_id = parse_id("span.id");

    // Find the timestamp field (or set it to NaN if none is found)
    // Note: this will leave the original timestamp field in the flattened_fields
    let mut timestamp = None;
    for field_name in timestamp_field_names {
        if let Some(ts) = flattened_fields.get(*field_name) {
            // Try parsing the field either as an RFC 3339 timestamp or a unix timestamp
            if let Ok(ts) = OffsetDateTime::parse(ts, &Rfc3339) {
                timestamp = Some(ts.unix_timestamp() as f64);
                break;
            } else if let Ok(ts) = f64::from_str(ts) {
                timestamp = Some(ts);
                break;
            }
        }
    }
    let timestamp = timestamp.unwrap_or(f64::NAN);

    // Find the body field (or set it to an empty string if none is found)
    // Note: this will leave the body field in the flattened_fields and copy
    // it into the body of the LogRecord
    let mut body = String::new();
    for field_name in body_field_names {
        if let Some(b) = flattened_fields.get(*field_name) {
            body = b.to_string();
            break;
        }
    }

    // All fields that are not mapped to the resource field
    // become part of the attributes field
    // TODO refactor this so we only make one pass over the fields
    let (resource, attributes): (HashMap<String, String>, HashMap<String, String>) =
        flattened_fields.into_iter().partition(|(key, _)| {
            RESOURCE_FIELD_PREFIXES
                .iter()
                .any(|prefix| key.starts_with(prefix))
                && !RESOURCE_FIELD_EXCEPTIONS.contains(&key.as_str())
        });

    Some(LogRecord {
        body,
        timestamp,
        attributes,
        resource,
        trace_id,
        span_id,
    })
}

fn flatten_nested_value(output: &mut HashMap<String, String>, key: String, value: Value) {
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
        Value::String(v) => {
            output.insert(key, v);
        }
        Value::Number(v) => {
            output.insert(key, v.to_string());
        }
        Value::Bool(v) => {
            output.insert(key, v.to_string());
        }
        Value::Null => {
            output.insert(key, "".to_string());
        }
    };
}

async fn check_status(config: Config) -> Result<()> {
    let mut url = config.url;

    // Add "_xpack" to the path
    {
        let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
            message: "Invalid ElasticSearch URL: cannot-be-a-base".to_string(),
        })?;
        path_segments.push("_xpack");
    }

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let request = HttpRequest::builder()
        .body(None)
        .headers(Some(headers))
        .method(HttpRequestMethod::Get)
        .url(url.to_string())
        .build();

    let _ = make_http_request(request).await?;

    // At this point we don't care to validate the info LOKI sends back
    // We just care it responded with 200 OK
    Ok(())
}

fn timestamp_to_rfc3339(timestamp: f64) -> String {
    OffsetDateTime::from_unix_timestamp(timestamp.trunc() as i64)
        .expect("Error parsing timestamp")
        .format(&Rfc3339)
        .expect("Error formatting timestamp as RFC3339")
}
