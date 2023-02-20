mod config;

pub use config::Config;
use fiberplane_provider_bindings::{
    log, make_http_request, Blob, Error, HttpRequest, HttpRequestError, HttpRequestMethod,
};
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;
use url::Url;

/// Response to the /api/datasources endpoint
#[derive(Deserialize, Debug)]
struct Datasource {
    id: u32,
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "type")]
    ty: String,
}

/// Try querying the URL as if it were both a plain Prometheus/Loki instance and a Grafana URL.
pub async fn query_direct_and_proxied<T>(
    config: &Config,
    data_source_type: &'static str,
    path_and_query: &str,
    body: Option<Blob>,
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    // Send it first as a direct query
    match send_query(
        &config.url,
        path_and_query,
        config.to_headers(),
        body.clone(),
    )
    .await
    {
        Ok(response) => Ok(response),
        Err(direct_err) => {
            // If the direct query fails, see if the URL is a Grafana URL and try to get the proxy URL from it
            if let Ok(url) = get_grafana_datasource_proxy_url(config, data_source_type).await {
                if let Ok(response) =
                    send_query(&url, path_and_query, config.to_headers(), body).await
                {
                    Ok(response)
                } else {
                    Err(direct_err)
                }
            } else {
                Err(direct_err)
            }
        }
    }
}

/// Send a query to the given URL
async fn send_query<T>(
    url: &Url,
    path_and_query: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<Blob>,
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let url = url
        .join(path_and_query)
        .map_err(|e| Error::Config {
            message: format!("Invalid URL: {e:?}"),
        })?
        .to_string();

    let request = if let Some(blob) = body {
        let mut headers = headers.unwrap_or_default();
        headers.insert("Content-Type".to_string(), blob.mime_type);
        HttpRequest {
            url,
            headers: Some(headers),
            method: HttpRequestMethod::Post,
            body: Some(blob.data),
        }
    } else {
        HttpRequest {
            url,
            headers,
            method: HttpRequestMethod::Get,
            body: None,
        }
    };

    log(format!(
        "Sending {:?} request to {}",
        request.method, request.url
    ));

    let response = make_http_request(request)
        .await
        .map_err(|error| match &error {
            HttpRequestError::ServerError {
                status_code,
                response,
            } if *status_code == 400 => Error::Other {
                message: String::from_utf8_lossy(response).to_string(),
            },
            _ => Error::Http { error },
        })?;

    serde_json::from_slice(&response.body).map_err(|e| Error::Data {
        message: format!("Error parsing response: {e:?}"),
    })
}

/// Load the Grafana datasources, find the datasource among them, and
/// return the URL to proxy requests to the underlying data source
/// See https://grafana.com/docs/grafana/latest/developers/http_api/data_source/#data-source-proxy-calls-by-id
async fn get_grafana_datasource_proxy_url(
    config: &Config,
    data_source_type: &'static str,
) -> Result<Url, Error> {
    // Query for the available datasources
    let url = config
        .url
        .join("api/datasources")
        .map_err(|e| Error::Config {
            message: format!("Invalid URL: {e:?}"),
        })?;
    let response = make_http_request(HttpRequest {
        body: None,
        headers: config.to_headers(),
        method: HttpRequestMethod::Get,
        url: url.to_string(),
    })
    .await?;
    let data_sources: Vec<Datasource> =
        serde_json::from_slice(&response.body).map_err(|e| Error::Deserialization {
            message: format!("Could not parse Grafana datasources response: {e:?}"),
        })?;

    // Find one of type "loki"
    let loki_data_source = data_sources
        .into_iter()
        .find(|ds| ds.ty == data_source_type)
        .ok_or_else(|| Error::Other {
            message: format!("No {data_source_type} data source found in grafana"),
        })?;

    log(format!("Found loki data source: {loki_data_source:?}"));

    // Construct the proxy URL
    config
        .url
        .join(&format!("api/datasources/proxy/{}/", loki_data_source.id))
        .map_err(|e| Error::Config {
            message: format!("Invalid URL: {e:?}"),
        })
}
