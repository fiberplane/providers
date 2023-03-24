use fiberplane_pdk::prelude::*;
use std::convert::TryInto;
use std::{collections::HashMap, env};
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

#[pdk_export]
async fn get_supported_query_types(config: ProviderConfig) -> Vec<SupportedQueryType> {
    let config = serde_json::from_value::<Config>(config);
    let path_label = match config {
        Ok(config) if config.api.is_some() => "Path to query in the API, starting with /",
        _ => "Address to query, starting with https://",
    };

    vec![
        SupportedQueryType::new(PERFORM_QUERY_TYPE)
            .with_schema(vec![
                // TODO: Wait for Studio to implement the select field (FP-2590),
                // then use a QueryField::Select to implement the type of query
                TextField::new()
                    .with_name(HTTP_METHOD_PARAM_NAME)
                    .with_label("Type of query")
                    .required()
                    .into(),
                TextField::new()
                    .with_name(PATH_PARAM_NAME)
                    .with_label(path_label)
                    .required()
                    .into(),
                TextField::new()
                    .with_name(QUERY_PARAM_NAME)
                    .with_label("Query parameters. One pair key=value per line, like 'q=fiberplane'")
                    .multiline()
                    .into(),
                TextField::new()
                    .with_name(EXTRA_HEADERS_PARAM_NAME)
                    .with_label("Extra headers to pass. One pair key=value per line, like 'Accept=application/json'")
                    .multiline()
                    .into(),
                TextField::new()
                    .with_name(BODY_PARAM_NAME)
                    .with_label("The request body.")
                    .multiline()
                    .into(),
                // TODO: Wait for Studio to implement the checkbox field (FP-2593)
                // to add a FORCE_JSON_PARAM_NAME field that is just a
                // checkbox that adds an 'Accept: application/json' header
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob> {
    log(format!(
        "https provider (commit: {}, built at: {}) invoked for query type \"{}\" and query data \"{:?}\" with config \"{:?}\"",
        COMMIT_HASH, BUILD_TIMESTAMP, request.query_type, request.query_data, request.config
    ));

    let config: Config =
        serde_json::from_value(request.config.clone()).map_err(|err| Error::Config {
            message: format!("Error parsing config: {err:?}"),
        })?;

    match request.query_type.as_str() {
        PERFORM_QUERY_TYPE => handle_query(config, request).await,
        STATUS_QUERY_TYPE => check_status(config).await,
        _ => Err(Error::UnsupportedRequest),
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
    headers: Option<HashMap<String, String>>,
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
        headers.insert("Content-Type".to_string(), blob.mime_type.clone());
    };

    let request = HttpRequest::builder()
        .url(url)
        .headers(Some(headers))
        .method(method)
        .body(body.map(|blob| blob.data))
        .build();
    log(format!(
        "Sending {:?} request to {}",
        request.method, request.url
    ));

    make_http_request(request).await.try_into()
}

async fn check_status(config: Config) -> Result<Blob> {
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

async fn handle_query(config: Config, request: ProviderRequest) -> Result<Blob> {
    // HACK: This KeyValueRow structure is a stop-gap measure until we
    // have ArrayField support in the query data, so we could
    // directly have Vec of KeyValueRow in the provider request QuerySchema.
    /// Row of key-value pair with front-end-relevant metadata.
    /// The metadata needs to stay attached to allow for proper
    /// synchronization of UI state using only the queryData of
    /// the provider cell
    #[derive(Default)]
    struct KeyValueRow {
        /// UUID used for React DOM refreshes.
        /// Safe to ignore here.
        uuid: String,
        /// True if the Key-Value pair has been enabled (checked) on UI
        enabled: bool,
        /// Key (a header key, or a query param name)
        ///
        /// NOTE: Semi-colons aren't allowed in the field, this would break
        /// parsing. As the keys here are either header keys or query parameter
        /// keys, it is a safe assumption to make to simplify the code for the
        /// time being until ArrayField are properly supported. This compromise
        /// is one of the reason the provider is still only Beta in support.
        key: String,
        /// Value (a header value, or a query param value)
        value: String,
    }

    /// A Row is expected to be serialized as semi-colon separated csv:
    /// `UUID;BOOL;KEY;VALUE`
    /// We allow 0/1 or true/false for the bool, but 0/1 is more efficient really
    impl std::str::FromStr for KeyValueRow {
        type Err = Error;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            let mut res = KeyValueRow::default();
            for (i, val) in s.splitn(4, ';').enumerate() {
                match i {
                    0 => res.uuid = val.into(),
                    1 => {
                        if !["1", "0", "true", "false"].contains(&val.to_ascii_lowercase().as_ref())
                        {
                            return Err(Error::UnsupportedRequest);
                        }
                        res.enabled = val == "1" || &val.to_ascii_lowercase() == "true";
                    }
                    2 => res.key = val.into(),
                    3 => res.value = val.into(),
                    _ => unreachable!("The iterator comes from a splitn(4, ...)"),
                }
            }
            Ok(res)
        }
    }

    if request.query_data.mime_type != FORM_ENCODED_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }
    let mut path = String::new();
    let mut query = String::new();
    let mut url = Err(Error::Invocation {
        message: "no URL given".to_string(),
    });
    let mut headers: Option<HashMap<String, String>> = None;
    let mut method = HttpRequestMethod::Get;
    for (key, value) in form_urlencoded::parse(&request.query_data.data) {
        match key.as_ref() {
            HTTP_METHOD_PARAM_NAME => match value.as_ref().to_uppercase().as_str() {
                "GET" => method = HttpRequestMethod::Get,
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
            },
            PATH_PARAM_NAME => {
                if let Some(ref api) = config.api {
                    if value.parse::<Url>().is_ok() {
                        return Err(Error::ValidationError {
                            errors: vec![ValidationError::builder()
                                .field_name(PATH_PARAM_NAME.to_string())
                                .message(
                                    "a provider with a baseUrl cannot query arbitrary URLs"
                                        .to_string(),
                                )
                                .build()],
                        });
                    }
                    url = Ok(api.base_url.clone());
                    path = value.to_string();
                    if let Some(api_headers) = api.to_headers() {
                        if let Some(h) = headers.as_mut() {
                            for (k, v) in api_headers {
                                h.insert(k, v);
                            }
                        } else {
                            headers = Some(api_headers)
                        }
                    };
                } else if let Ok(full_url) = value.parse::<Url>() {
                    url = Ok(full_url);
                } else {
                    return Err(Error::ValidationError {
                        errors: vec![ValidationError::builder()
                            .field_name(PATH_PARAM_NAME.to_string())
                            .message(format!("invalid url: {value:?}"))
                            .build()],
                    });
                }
            }
            EXTRA_HEADERS_PARAM_NAME => {
                if headers.is_none() {
                    headers = Some(HashMap::new())
                }
                for line in value.as_ref().lines() {
                    let row: KeyValueRow = line.parse()?;
                    if row.enabled {
                        headers.as_mut().map(|h| h.insert(row.key, row.value));
                    }
                }
            }
            QUERY_PARAM_NAME => {
                let mut serializer = form_urlencoded::Serializer::new(String::new());
                serializer.extend_pairs(value.as_ref().lines().filter_map(|line| {
                    let row: KeyValueRow = line.parse().ok()?;
                    if row.enabled {
                        Some((row.key, row.value))
                    } else {
                        None
                    }
                }));
                query = serializer.finish()
            }
            _ => {
                log(format!(
                    "https provider received an unknown query parameter: {}",
                    key.as_ref()
                ));
            }
        }
    }

    let url = url?;

    if !query.is_empty() {
        path = format!("{path}?{query}");
    }

    send_query(&url, &path, method, headers, None)
        .await
        .and_then(|resp| resp.try_into_blob(config.show_headers))
}
