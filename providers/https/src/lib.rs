use fiberplane_pdk::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::env;
use url::Url;

mod config;
mod constants;
mod provider_response;

use config::*;
use constants::*;
use provider_response::HttpsProviderResponse;

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[pdk_export]
fn get_config_schema() -> ConfigSchema {
    vec![
        TextField::new()
            .with_name("baseUrl")
            .with_label("Base URL of the API we are interested in")
            .with_placeholder("Leave empty to allow querying any URL")
            .into(),
        TextField::new()
            .with_name("healthCheckPath")
            .with_label("Path to the healthcheck or status endpoint, relative to the base URL")
            .into(),
        TextField::new()
            .with_name("username")
            .with_label("Username for authentication (if the API uses Basic auth)")
            .into(),
        TextField::new()
            .with_name("password")
            .with_label("Password for authentication (if the API uses Basic auth)")
            .into(),
        TextField::new()
            .with_name("token")
            .with_label("Token for authentication (if the API uses Bearer auth)")
            .into(),
        CheckboxField::new()
            .with_name("showHeaders")
            .with_label("Show response headers in the query results")
            .into(),
    ]
}

pdk_query_types! {
    PERFORM_QUERY_TYPE => {
        label: "Perform HTTP query",
        handler: handle_query(HttpProviderQuery, Config).await,
        supported_mime_types: [CELLS_MIME_TYPE]
    },
    STATUS_MIME_TYPE => {
        handler: check_status(ProviderRequest).await,
        supported_mime_types: [STATUS_MIME_TYPE]
    }
}

#[pdk_export]
fn create_cells(query_type: String, response: Blob) -> Result<Vec<Cell>> {
    Err(Error::Invocation {
        message: format!("create_cells is not implemented for this provider, it only returns {} blobs that must be handled by the runtime natively (received a {} blob for {}).", CELLS_MSGPACK_MIME_TYPE, response.mime_type, query_type)
    })
}

/// Send a query to the given URL
async fn send_query(
    url: &Url,
    path_and_query: &str,
    method: HttpRequestMethod,
    headers: Option<BTreeMap<String, String>>,
    body: Option<Blob>,
) -> Result<HttpsProviderResponse> {
    let url = url
        .join(path_and_query)
        .map_err(|e| Error::Config {
            message: format!("Invalid URL: {e:?}"),
        })?
        .to_string();

    let mut headers = headers.unwrap_or_default();
    if let Some(ref blob) = body {
        headers.insert("Content-Type".to_owned(), blob.mime_type.clone());
    };

    log(format!("Sending {method:?} request to {url}"));

    let mut request = HttpRequest::default();
    request.url = url.clone();
    request.method = method;
    request.headers = Some(headers);
    request.body = body.map(|blob| blob.data);

    make_http_request(request).await.try_into()
}

async fn check_status(request: ProviderRequest) -> Result<Blob> {
    log(format!(
        "https provider (commit: {}, built at: {}) invoked for query type \"{}\" and query data \"{:?}\" with config \"{:?}\"",
        COMMIT_HASH, BUILD_TIMESTAMP, request.query_type, request.query_data, request.config
    ));

    let config: Config = Config::parse(request.config.clone())?;
    match config.api {
        Some(api) if api.health_check_path.is_some() => {
            let info = send_query(
                &api.base_url,
                api.health_check_path.as_deref().unwrap_or_default(),
                HttpRequestMethod::Get,
                api.to_headers(),
                None,
            )
            .await?;
            Ok(info.try_into_blob(config.show_headers)?)
        }
        _ => Ok(HttpsProviderResponse {
            status: "ok".to_string(),
            headers: None,
            payload: Vec::new(),
        }
        // We do not care about headers for the Ok status response
        .try_into_blob(false)?),
    }
}

#[derive(QuerySchema, Deserialize)]
pub struct HttpProviderQuery {
    pub method: String,
    pub path: String,
    pub extra_headers: Vec<Header>,
    pub query_params: Vec<QueryParam>,
}

#[derive(QuerySchema, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(QuerySchema, Deserialize)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
}

async fn handle_query(request: HttpProviderQuery, config: Config) -> Result<Blob> {
    let mut path = String::new();
    let mut url = Err(Error::Invocation {
        message: "no URL given".to_string(),
    });
    let mut headers: BTreeMap<String, String> = BTreeMap::new();

    if let Some(ref api) = config.api {
        if request.path.parse::<Url>().is_ok() {
            return Err(Error::ValidationError {
                errors: vec![ValidationError::builder()
                    .field_name(PATH_PARAM_NAME.to_string())
                    .message("a provider with a baseUrl cannot query arbitrary URLs".to_string())
                    .build()],
            });
        }
        url = Ok(api.base_url.clone());
        path = request.path.clone();
        if let Some(api_headers) = api.to_headers() {
            headers = api_headers;
        };
    } else if let Ok(full_url) = request.path.parse::<Url>() {
        url = Ok(full_url);
    } else {
        return Err(Error::ValidationError {
            errors: vec![ValidationError::builder()
                .field_name(PATH_PARAM_NAME.to_string())
                .message(format!("invalid url: {:?}", request.path))
                .build()],
        });
    }

    let method = match request.method.as_str().to_uppercase().as_str() {
        "GET" => HttpRequestMethod::Get,
        unsupported => {
            return Err(Error::ValidationError {
                errors: vec![ValidationError::builder()
                    .field_name(HTTP_METHOD_PARAM_NAME.to_string())
                    .message(format!(
                        "{unsupported} is not a supported HTTPS method with this provider."
                    ))
                    .build()],
            })
        }
    };
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.extend_pairs(
        request
            .query_params
            .iter()
            .map(|param| (param.key.to_string(), param.value.to_string())),
    );
    let query = serializer.finish();

    request.extra_headers.iter().for_each(|header| {
        headers.insert(header.key.clone(), header.value.clone());
    });
    let url = url?;

    if !query.is_empty() {
        path = format!("{path}?{query}");
    }

    send_query(&url, &path, method, Some(headers), None)
        .await
        .and_then(|resp| resp.try_into_blob(config.show_headers))
}
