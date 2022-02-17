use elasticsearch_dsl::{Hit, SearchResponse};
use fp_provider::{
    fp_export_impl, log, make_http_request, Config, Error, HttpRequest, HttpRequestMethod,
    LogRecord, ProviderRequest, ProviderResponse, QueryLogs,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::{Map, Value};
use std::{cmp::Ordering, collections::HashMap, str::FromStr};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use url::Url;

#[cfg(test)]
mod tests;

// This mapping is based on the recommended mapping from the
// Elastic Common Schema to the OpenTelemetry Log specification
// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#elastic-common-schema
static RESOURCE_FIELD_PREFIXES: &[&str] = &["agent.", "cloud.", "container.", "host.", "service."];
static RESOURCE_FIELD_EXCEPTIONS: &[&str] = &["container.labels", "host.uptime", "service.state"];

#[derive(Serialize)]
struct SearchRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct Document {
    #[serde(flatten)]
    fields: Map<String, Value>,
    // TODO parse fields from elastic common schema
}

#[fp_export_impl(fp_provider)]
async fn invoke(request: ProviderRequest, config: Config) -> ProviderResponse {
    match request {
        // TODO implement AutoSuggest
        ProviderRequest::Logs(query) => fetch_logs(query, config)
            .await
            .map(|log_records| ProviderResponse::LogRecords { log_records })
            .unwrap_or_else(|error| ProviderResponse::Error { error }),
        _ => ProviderResponse::Error {
            error: Error::UnsupportedRequest,
        },
    }
}

async fn fetch_logs(query: QueryLogs, config: Config) -> Result<Vec<LogRecord>, Error> {
    let url = config.url.ok_or_else(|| Error::Config {
        message: "URL is required".to_string(),
    })?;
    let timestamp_field = config
        .options
        .get("timestamp_name")
        .cloned()
        .unwrap_or_else(|| "@timestamp".to_owned());

    let body_field = config
        .options
        .get("body_name")
        .cloned()
        .unwrap_or_else(|| "message".to_owned());

    let mut url = Url::parse(&url).map_err(|e| Error::Config {
        message: format!("Invalid ElasticSearch URL: {:?}", e),
    })?;

    // Add "_search" to the path
    let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
        message: "Invalid ElasticSearch URL".to_string(),
    })?;
    path_segments.push("_search");
    drop(path_segments);

    // TODO parse the authentication details from the URL and put them in the headers
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    // Lucene Query Syntax: https://www.elastic.co/guide/en/kibana/current/lucene-query.html
    // TODO should we determine timestamp field name from API?
    let query_string = format!(
        "{} AND {}:[{} TO {}]",
        query.query,
        timestamp_field,
        timestamp_to_rfc3339(query.time_range.from),
        timestamp_to_rfc3339(query.time_range.to)
    );

    let body = SearchRequestBody { size: query.limit };
    url.set_query(Some(&format!("q={}", &query_string)));

    let request = HttpRequest {
        body: Some(ByteBuf::from(serde_json::to_vec(&body).map_err(|e| {
            Error::Data {
                message: format!("Error serializing query: {:?}", e),
            }
        })?)),
        headers: Some(headers),
        method: HttpRequestMethod::Post,
        url: url.to_string(),
    };

    // Parse response
    let response = make_http_request(request)
        .await
        .map_err(|error| Error::Http { error })?
        .body;
    let response: SearchResponse<Document, Document> =
        serde_json::from_slice(&response).map_err(|e| Error::Data {
            message: format!("Error parsing ElasticSearch response: {:?}", e),
        })?;

    if response.timed_out {
        return Err(Error::Other {
            message: "ElasticSearch query timed out".to_string(),
        });
    }

    log(format!("Got {} results", response.hits.hits.len()));

    parse_response(&timestamp_field, &body_field, response)
}

fn parse_response(
    timestamp_field: &str,
    body_field: &str,
    response: SearchResponse<Document, Document>,
) -> Result<Vec<LogRecord>, Error> {
    let mut logs: Vec<LogRecord> = response
        .hits
        .hits
        .into_iter()
        .filter_map(|hit| parse_hit(hit, timestamp_field, body_field))
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
    hit: Hit<Document, Document>,
    timestamp_field: &str,
    body_field: &str,
) -> Option<LogRecord> {
    let source = hit.source?;
    let mut flattened_fields = HashMap::new();
    for (key, val) in source.fields.into_iter() {
        flatten_nested_value(&mut flattened_fields, key, val);
    }

    // Parse the trace ID and span ID from hex if they exist
    let mut parse_id = |key: &str| {
        if let Some((key, val)) = flattened_fields.remove_entry(key) {
            if let Ok(bytes) = hex::decode(val.to_string().replace("-", "")) {
                Some(ByteBuf::from(bytes))
            } else {
                log(format!("unable to decode ID as hex in log: {}", val));
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

    let timestamp = flattened_fields.remove(timestamp_field).or_else(|| {
        log(format!(
            "missing timestamp field ({}) in log. ignoring record: {:?}",
            timestamp_field, flattened_fields
        ));
        None
    })?;
    let timestamp = if let Ok(timestamp) = OffsetDateTime::parse(&timestamp, &Rfc3339) {
        timestamp.unix_timestamp() as f64
    } else if let Ok(timestamp) = f64::from_str(&timestamp) {
        timestamp
    } else {
        log(format!(
            "unable to parse timestamp field ({}: {}) in log. ignoring record: {:?}",
            timestamp_field, timestamp, flattened_fields
        ));
        return None;
    };

    let body = flattened_fields.remove(body_field).or_else(|| {
        log(format!(
            "missing body field ({}) in log. ignoring record: {:?}",
            body_field, flattened_fields
        ));
        None
    })?;

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
                flatten_nested_value(output, format!("{}.{}", key, sub_key), val);
            }
        }
        Value::Array(v) => {
            for (index, val) in v.into_iter().enumerate() {
                // TODO should the separator be dots instead?
                flatten_nested_value(output, format!("{}[{}]", key, index), val);
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

fn timestamp_to_rfc3339(timestamp: f64) -> String {
    OffsetDateTime::from_unix_timestamp(timestamp.trunc() as i64)
        .expect("Error parsing timestamp")
        .format(&Rfc3339)
        .expect("Error formatting timestamp as RFC3339")
}
