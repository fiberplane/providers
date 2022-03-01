use fiberplane::protocols::core::LokiDataSource;
use fp_provider::{
    fp_export_impl, make_http_request, Error, HttpRequest, HttpRequestMethod, LogRecord,
    ProviderRequest, ProviderResponse, QueryLogs, Timestamp, Value,
};
use rmpv::ext::from_value;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};
use url::Url;

#[fp_export_impl(fp_provider)]
async fn invoke(request: ProviderRequest, config: Value) -> ProviderResponse {
    let config: LokiDataSource = match from_value(config) {
        Ok(config) => config,
        Err(err) => {
            return ProviderResponse::Error {
                error: Error::Config {
                    message: format!("Error parsing config: {:?}", err),
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

async fn fetch_logs(query: QueryLogs, config: LokiDataSource) -> Result<Vec<LogRecord>, Error> {
    let mut url = Url::parse(&config.url).map_err(|e| Error::Config {
        message: format!("Invalid LOKI URL: {:?}", e),
    })?;

    {
        let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
            message: "Invalid LOKI URL".to_string(),
        })?;
        path_segments
            .push("loki")
            .push("api")
            .push("v1")
            .push("query_range");
    }

    let qstr: String =
        url::form_urlencoded::Serializer::new(String::with_capacity(query.query.capacity()))
            .append_pair("query", &query.query)
            .append_pair(
                "limit",
                &query.limit.map_or("".to_owned(), |l| l.to_string()),
            )
            .append_pair("start", &((query.time_range.from) as u64).to_string())
            .append_pair("end", &((query.time_range.to) as u64).to_string())
            .finish();
    url.set_query(Some(&qstr));

    let request = HttpRequest {
        body: None,
        headers: None,
        method: HttpRequestMethod::Post,
        url: url.to_string(),
    };

    // Parse response
    let response = make_http_request(request)
        .await
        .map_err(|error| match &error {
            fp_provider::HttpRequestError::ServerError {
                status_code,
                response,
            } if *status_code == 400 => Error::Other {
                message: format!("Query error: {}", String::from_utf8_lossy(response)),
            },
            _ => Error::Http { error },
        })?
        .body;

    let response: QueryResponse = serde_json::from_slice(&response).map_err(|e| Error::Data {
        message: format!("Error parsing LOKI response: {:?}", e),
    })?;

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
        .collect::<Result<Vec<LogRecord>, _>>()
        .map_err(|e| Error::Data {
            message: format!("Failed to parse data, got error: {:?}", e),
        })?;

    Ok(logs)
}

fn data_mapper(
    d: &Data,
) -> impl Iterator<Item = Result<LogRecord, <Timestamp as FromStr>::Err>> + '_ {
    let att = &d.labels;
    d.values.iter().map(move |(ts, v)| {
        let timestamp = Timestamp::from_str(ts)?;
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

async fn check_status(config: LokiDataSource) -> Result<(), Error> {
    let mut url = Url::parse(&config.url).map_err(|e| Error::Config {
        message: format!("Invalid LOKI URL: {:?}", e),
    })?;

    {
        let mut path_segments = url.path_segments_mut().map_err(|_| Error::Config {
            message: "Invalid LOKI URL: cannot-be-a-base".to_string(),
        })?;
        path_segments
            .push("loki")
            .push("api")
            .push("v1")
            .push("status")
            .push("buildinfo");
    }

    let request = HttpRequest {
        body: None,
        headers: None,
        method: HttpRequestMethod::Get,
        url: url.to_string(),
    };

    let _ = make_http_request(request)
        .await
        .map_err(|error| Error::Http { error })?;

    // At this point we don't care to validate the info LOKI sends back
    // We just care it responded with 200 OK
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{Data, QueryData, QueryResponse};
    use serde::Deserialize;
    use serde_json::Deserializer;
    use std::collections::HashMap;

    #[test]
    fn test_deserialization() {
        let data = r#"{
            "status": "success",
            "data": {
              "resultType": "streams",
              "result": [
                {
                  "stream": {
                    "filename": "/var/log/myproject.log",
                    "job": "varlogs",
                    "level": "info"
                  },
                  "values": [
                    [
                      "1569266497240578000",
                      "foo"
                    ],
                    [
                      "1569266492548155000",
                      "bar"
                    ]
                  ]
                }
              ],
              "stats": {
              }
            }
          }"#;

        let value = QueryResponse::deserialize(&mut Deserializer::from_str(data)).unwrap();

        assert_eq!(
            value,
            QueryResponse {
                data: QueryData::Streams(vec![Data {
                    labels: HashMap::from([
                        ("filename".to_owned(), "/var/log/myproject.log".to_owned()),
                        ("job".to_owned(), "varlogs".to_owned()),
                        ("level".to_owned(), "info".to_owned()),
                    ]),
                    values: vec![
                        ("1569266497240578000".to_owned(), "foo".to_owned()),
                        ("1569266492548155000".to_owned(), "bar".to_owned()),
                    ],
                }]),
                status: "success".to_owned(),
            },
        )
    }
}
