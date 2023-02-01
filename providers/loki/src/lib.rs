use fiberplane_pdk::prelude::*;
use fiberplane_provider_bindings::{
    LegacyLogRecord as LogRecord, LegacyProviderRequest as ProviderRequest,
    LegacyProviderResponse as ProviderResponse, QueryLogs,
};
use grafana_common::{query_direct_and_proxied, Config};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};

#[cfg(test)]
mod tests;

const CONVERSION_FACTOR: f64 = 1e9;
static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[pdk_export]
async fn invoke(request: ProviderRequest, config: ProviderConfig) -> ProviderResponse {
    log(format!(
        "Loki provider (commit: {COMMIT_HASH}, built at: {BUILD_TIMESTAMP}) invoked with request: {request:?}"
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
    labels: HashMap<String, String>,
    values: Vec<(String, String)>,
}

async fn fetch_logs(query: QueryLogs, config: Config) -> Result<Vec<LogRecord>> {
    // Convert unix epoch in seconds to epoch in nanoseconds
    let from = (query.time_range.from * CONVERSION_FACTOR).to_string();
    let to = (query.time_range.to * CONVERSION_FACTOR).to_string();

    let query_string: String =
        url::form_urlencoded::Serializer::new(String::with_capacity(query.query.capacity()))
            .append_pair("query", &query.query)
            .append_pair(
                "limit",
                &query.limit.map_or("".to_owned(), |l| l.to_string()),
            )
            .append_pair("start", &from)
            .append_pair("end", &to)
            .finish();

    let response: QueryResponse = query_direct_and_proxied(
        &config,
        "loki",
        &format!("loki/api/v1/query_range?{}", &query_string),
        None,
    )
    .await?;

    if response.status != "success" {
        return Err(Error::Data {
            message: format!("Query didn't succeed, returned: \"{}\"", response.status),
        });
    }

    let data = match response.data {
        QueryData::Streams(d) => d,
        QueryData::Matrix(d) => d,
        _ => {
            return Err(Error::Data {
                message: "Query didn't return a stream or matrix".to_string(),
            })
        }
    };

    let logs = data
        .iter()
        .flat_map(data_mapper)
        .collect::<core::result::Result<Vec<LogRecord>, _>>()
        .map_err(|e| Error::Data {
            message: format!("Failed to parse data, got error: {e:?}"),
        })?;

    Ok(logs)
}

fn data_mapper(
    d: &Data,
) -> impl Iterator<Item = core::result::Result<LogRecord, <LegacyTimestamp as FromStr>::Err>> + '_ {
    let att = &d.labels;
    d.values.iter().map(move |(ts, v)| {
        //convert unix epoch in nanoseconds to unix epoch in seconds
        //https://grafana.com/docs/loki/latest/api/#get-lokiapiv1query_range
        let timestamp = LegacyTimestamp::from_str(ts)? / CONVERSION_FACTOR;
        Ok(LogRecord {
            timestamp,
            body: v.clone(),
            attributes: att.clone(),
            span_id: None,
            trace_id: None,
            resource: HashMap::default(),
        })
    })
}

async fn check_status(config: Config) -> Result<()> {
    // Send a fake query to check the status
    let query_string = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("query", r#"{job="fiberplane_check_status"} != ``"#)
        .finish();

    query_direct_and_proxied::<Value>(
        &config,
        "loki",
        &format!("loki/api/v1/query_range?{}", &query_string),
        None,
    )
    .await?;
    Ok(())
}
