use fp_provider::{fp_export, prelude::*};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryPayload {
    query: String,
    query_type: QueryType,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum QueryType {
    #[serde(rename_all = "camelCase")]
    Instant { time: Timestamp },
    #[serde(rename_all = "camelCase")]
    Series { time_range: TimeRange },
}

fp_export!(
    async fn fetch_instant(
        query: String,
        opts: QueryInstantOptions,
    ) -> Result<Vec<Instant>, FetchError> {
        let data_source = match opts.data_source {
            DataSource::Proxy(data_source) => Ok(data_source),
            _ => Err(FetchError::Other {
                message: "Incompatible data source".to_owned(),
            }),
        }?;

        let payload = QueryPayload {
            query,
            query_type: QueryType::Instant { time: opts.time },
        };

        let result = make_request(Request {
            body: Some(
                serde_json::ser::to_vec(&payload).map_err(|err| FetchError::Other {
                    message: format!("Could not serialize query: {}", err),
                })?,
            ),
            headers: Some(create_headers()),
            method: RequestMethod::Post,
            url: create_url(&data_source),
        })
        .await;
        match result {
            Ok(response) => {
                rmp_serde::decode::from_slice(&response.body).map_err(|err| FetchError::DataError {
                    message: format!("Error parsing Proxy response: {}", err),
                })
            }
            Err(error) => Err(FetchError::RequestError { payload: error }),
        }
    }
);

fp_export!(
    async fn fetch_series(
        query: String,
        opts: QuerySeriesOptions,
    ) -> Result<Vec<Series>, FetchError> {
        let data_source = match opts.data_source {
            DataSource::Proxy(data_source) => Ok(data_source),
            _ => Err(FetchError::Other {
                message: "Incompatible data source".to_owned(),
            }),
        }?;

        let payload = QueryPayload {
            query,
            query_type: QueryType::Series {
                time_range: opts.time_range,
            },
        };

        let result = make_request(Request {
            body: Some(
                serde_json::ser::to_vec(&payload).map_err(|err| FetchError::Other {
                    message: format!("Could not serialize query: {}", err),
                })?,
            ),
            headers: Some(create_headers()),
            method: RequestMethod::Post,
            url: create_url(&data_source),
        })
        .await;
        match result {
            Ok(response) => {
                rmp_serde::decode::from_slice(&response.body).map_err(|err| FetchError::DataError {
                    message: format!("Error parsing Proxy response: {}", err),
                })
            }
            Err(error) => Err(FetchError::RequestError { payload: error }),
        }
    }
);

fn create_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Accept".to_owned(), "application/x-msgpack".to_owned());
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());
    headers
}

fn create_url(data_source: &ProxyDataSource) -> String {
    use urlencoding::encode;

    format!(
        "/api/proxies/{}/relay?dataSourceName={}",
        encode(&data_source.proxy_id),
        encode(&data_source.data_source_name)
    )
}
