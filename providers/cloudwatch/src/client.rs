mod canonical_request;
pub mod cloudwatch;
pub mod cloudwatch_logs;
pub mod resource_groups_tagging;
mod sigv4;

pub use self::canonical_request::request_state;
pub use canonical_request::CanonicalRequest;
use fiberplane_pdk::{
    prelude::{log, make_http_request, now},
    providers::{Error, HttpRequest, HttpRequestError, HttpRequestMethod},
};
use http::Method;
use secrecy::SecretString;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ClientError {
    InvalidRequest(String),
    Host(HttpRequestError),
    UnexpectedResponse { expected: String, actual: String },
}

impl std::error::Error for ClientError {}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::InvalidRequest(e) => {
                write!(f, "invalid request to send to runtime host: {e}")
            }
            ClientError::Host(e) => write!(f, "runtime http client error: {e:?}"),
            ClientError::UnexpectedResponse { expected, actual } => {
                write!(
                    f,
                    "unexpected API response, expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self::Invocation {
            message: format!("HTTP request to AWS failed: {value}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientCommon {
    service: String,
    host: String,
    endpoint: String,
    region: String,
    access_key_id: SecretString,
    secret_access_key: SecretString,
}

impl ClientCommon {
    pub async fn send<R, D>(
        &self,
        request: R,
        extra_unsigned_headers: Option<&BTreeMap<String, String>>,
    ) -> Result<D, ClientError>
    where
        R: Into<CanonicalRequest<{ request_state::STEM }>>,
        D: DeserializeOwned,
    {
        let canonical = request.into();
        let request = canonical.prepare(&self.host, &now());

        // Converting a ready_to_sign Canonical request into a signed, bindings::HttpRequest payload
        let method = match &request.method {
            &Method::GET => Ok(HttpRequestMethod::Get),
            &Method::POST => Ok(HttpRequestMethod::Post),
            unknown => Err(ClientError::InvalidRequest(format!(
                "Method {unknown} is unsupported"
            ))),
        }?;
        let headers = self.format_headers(&request, extra_unsigned_headers);

        let uri: &str = request.uri.as_ref();
        let querystring = request.querystring();
        let url = if querystring.is_empty() {
            format!("{}{}", self.endpoint, uri)
        } else {
            format!("{}{}?{}", self.endpoint, uri, querystring)
        };

        let request = HttpRequest::builder()
            .url(url)
            .method(method)
            .headers(Some(headers.into_iter().collect()))
            .body(request.body.clone())
            .build();

        log(format!("CloudWatch: Sending request {request:?}"));

        make_http_request(request)
            .await
            .map_err(|err| {
                match &err {
                    HttpRequestError::ServerError {
                        status_code,
                        response,
                    } => {
                        log(format!(
                            "CloudWatch: HTTP error: {status_code}; {:?}",
                            String::from_utf8_lossy(response)
                        ));
                    }
                    other => log(format!("CloudWatch: HTTP error: {other:?}")),
                };
                ClientError::Host(err)
            })
            .and_then(|response| {
                serde_json::from_slice(&response.body).map_err(|err| {
                    log(format!(
                        "CloudWatch response body:\n{}",
                        String::from_utf8_lossy(&response.body)
                    ));
                    ClientError::InvalidRequest(format!(
                        "could not deserialize the result of the call: {err}"
                    ))
                })
            })
    }

    fn format_headers(
        &self,
        request: &CanonicalRequest<{ request_state::READY_TO_SIGN_V4 }>,
        extra_unsigned_headers: Option<&BTreeMap<String, String>>,
    ) -> BTreeMap<String, String> {
        let mut acc = request.headers.clone();
        let (auth, sig) = request.to_sigv4_auth_header(
            &self.access_key_id,
            &self.secret_access_key,
            &self.region,
            &self.service,
        );
        acc.insert(auth, sig);
        if let Some(extra_headers) = extra_unsigned_headers {
            acc.extend(
                extra_headers
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string())),
            )
        }
        acc.insert("Accept".to_string(), "application/json".to_string());
        acc
    }
}
