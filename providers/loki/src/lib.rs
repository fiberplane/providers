use fiberplane_pdk::prelude::*;
use fiberplane_pdk::serde_json::{self, Value};
use grafana_common::{query_direct_and_proxied, Config};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::str::FromStr;
use time::OffsetDateTime;

#[cfg(test)]
mod tests;

const PAGE_SIZE: &str = "30";

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(Deserialize, QuerySchema)]
pub struct LokiQuery {
    #[pdk(label = "Enter your Loki query")]
    pub query: String,

    #[pdk(label = "Specify a time range")]
    pub time_range: DateTimeRange,
}

pdk_query_types! {
    EVENTS_QUERY_TYPE => {
        label: "Loki query",
        handler: fetch_logs(LokiQuery, Config).await,
        supported_mime_types: [EVENTS_MIME_TYPE]
    },
    STATUS_QUERY_TYPE => {
        supported_mime_types: [STATUS_MIME_TYPE],
        handler: check_status(ProviderRequest).await
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct QueryResponse {
    status: String,
    data: QueryData,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "resultType", content = "result", rename_all = "camelCase")]
enum QueryData {
    Streams(Vec<Data>),
    Scalar {},
    Vector {},
    Matrix(Vec<Data>),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Data {
    #[serde(alias = "stream", alias = "metric")]
    labels: BTreeMap<String, Value>,
    values: Vec<(String, String)>,
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>> {
    log(format!("Creating cells for query type: {query_type}"));

    match query_type.as_str() {
        EVENTS_QUERY_TYPE => create_log_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
}

async fn fetch_logs(query: LokiQuery, config: Config) -> Result<Blob> {
    // Convert unix epoch in seconds to epoch in nanoseconds
    let from = (query.time_range.from.unix_timestamp_nanos()).to_string();
    let to = (query.time_range.to.unix_timestamp_nanos()).to_string();

    let query_string: String =
        url::form_urlencoded::Serializer::new(String::with_capacity(query.query.capacity()))
            .append_pair("query", &query.query)
            .append_pair("limit", PAGE_SIZE)
            .append_pair("start", &from)
            .append_pair("end", &to)
            .finish();

    let response: QueryResponse = query_direct_and_proxied(
        &config,
        "loki",
        &format!("loki/api/v1/query_range?{query_string}"),
        None,
    )
    .await?;

    if response.status != "success" {
        return Err(Error::Data {
            message: format!("Query didn't succeed, returned: \"{}\"", response.status),
        });
    }

    let data = match response.data {
        QueryData::Streams(d) => Ok(d),
        QueryData::Matrix(d) => Ok(d),
        _ => Err(Error::Data {
            message: "Query didn't return a stream or matrix".to_owned(),
        }),
    }?;

    let log_lines = data
        .iter()
        .flat_map(data_mapper)
        .collect::<Result<Vec<Event>>>()
        .map_err(|e| Error::Data {
            message: format!("Failed to parse data, got error: {e:?}"),
        })?;

    Events(log_lines).to_blob()
}

fn data_mapper(data: &Data) -> impl Iterator<Item = Result<Event>> + '_ {
    let attributes = &data.labels;
    data.values.iter().map(move |(timestamp, value)| {
        let timestamp = i128::from_str(timestamp)
            .ok()
            .and_then(|ts| OffsetDateTime::from_unix_timestamp_nanos(ts).ok())
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);

        let metadata = OtelMetadata::builder()
            .attributes(attributes.clone())
            .resource(BTreeMap::new())
            .trace_id(None)
            .span_id(None)
            .build();
        let event = Event::builder()
            .title(value.clone())
            .time(timestamp.into())
            .otel(metadata)
            .build();

        Ok(event)
    })
}

async fn check_status(request: ProviderRequest) -> Result<Blob> {
    let config = serde_json::from_value(request.config)?;

    // Send a fake query to check the status
    let query_string = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("query", r#"{job="fiberplane_check_status"} != ``"#)
        .finish();

    query_direct_and_proxied::<Value>(
        &config,
        "loki",
        &format!("loki/api/v1/query_range?{query_string}"),
        None,
    )
    .await?;

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
