use fiberplane_pdk::prelude::*;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::env;

// use url::Url;

mod config;
mod constants;
mod provider_response;

use config::*;
use constants::*;
use provider_response::HttpsProviderResponse;

use serde::{Serialize};

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(Serialize)]
struct OpenAiRequestBody {
    model: String,
    prompt: String,
    max_tokens: i32,
    temperature: f32,

}

#[pdk_export]
fn get_config_schema() -> ConfigSchema {
    vec![
        TextField::new()
            .with_name("organizationId")
            .with_label("OpenAI organization ID")
            .with_placeholder("Organization ID")
            .into(),
        TextField::new()
            .with_name("token")
            .with_label("OpenAI API key")
            .into(),
        TextField::new()
            .with_name("context")
            .with_label("Optional context for all requests (prefixed to all queries)")
            .into(),
    ]
}

#[pdk_export]
async fn get_supported_query_types(config: ProviderConfig) -> Vec<SupportedQueryType> {
    vec![
        SupportedQueryType::new(PERFORM_QUERY_TYPE)
            .with_schema(vec![
                TextField::new()
                    .with_name(PROMPT_PARAM_NAME)
                    .with_label("Prompt")
                    .required()
                    .into(),
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob> {
    log(format!(
        "openai provider (commit: {}, built at: {}) invoked for query type \"{}\" and query data \"{:?}\" with config \"{:?}\"",
        COMMIT_HASH, BUILD_TIMESTAMP, request.query_type, request.query_data, request.config
    ));

    let config: Config =
        serde_json::from_value(request.config.clone()).map_err(|err| Error::Config {
            message: format!("Error parsing config: {err:?}"),
        })?;

    match request.query_type.as_str() {
        PERFORM_QUERY_TYPE => handle_query(config, request).await,
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
    method: HttpRequestMethod,
    headers: Option<BTreeMap<String, String>>,
    body: Option<Blob>,
    // body: Vec<u8>,
) -> Result<HttpsProviderResponse> {
    let url = "https://api.openai.com/v1/completions".to_string();

    let mut headers = headers.unwrap_or_default();
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());

    log(format!("Sending {method:?} request to {url}"));
    log(format!("Headers {headers:?}"));

    let mut request = HttpRequest::default();
    request.url = url.clone();
    request.method = method;
    request.headers = Some(headers);
    request.body = body.map(|blob| blob.data);;

    // NOTE - the easier way
    //
    // let request = HttpRequest::post(url, body).with_headers(headers);

    log(format!("Request {request:?}"));

    make_http_request(request).await.try_into()
}

async fn handle_query(config: Config, request: ProviderRequest) -> Result<Blob> {
    if request.query_data.mime_type != FORM_ENCODED_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }
    let mut prompt = String::new();
    log(format!("Prompt {prompt:?}"));

    let headers: Option<BTreeMap<String, String>> = config.api.unwrap().to_headers();
    log(format!("Headers?"));

    let method = HttpRequestMethod::Post;

    for (key, value) in form_urlencoded::parse(&request.query_data.data) {
        match key.as_ref() {
            PROMPT_PARAM_NAME => {
                prompt = value.as_ref().to_string();
            }
            _ => {
                log(format!(
                    "https provider received an unknown query parameter: {}",
                    key.as_ref()
                ));
            }
        }
    }

    let request_body = OpenAiRequestBody {
        model: "text-davinci-003".to_string(),
        prompt: prompt,
        max_tokens: 1500,
        temperature: 0.5,
    };

    let body = serde_json::to_vec(&request_body).map_err(|err| Error::Data {
        message: format!("Error serializing openai request body: {err:?}"),
    })?;

    let blob = Blob::builder()
        .mime_type("application/json".to_string())
        .data(body)
        .build();

    send_query(method, headers, Some(blob))
        .await
        .and_then(|resp| resp.try_into_blob())
}

