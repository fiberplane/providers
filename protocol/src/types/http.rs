use fp_bindgen::prelude::*;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

/// HTTP request options.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpRequestMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<ByteBuf>,
}

/// Possible errors that may happen during an HTTP request.
#[derive(Serializable, Debug)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HttpRequestError {
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    ResponseTooBig,
    #[fp(rename_all = "camelCase")]
    ServerError {
        status_code: u16,
        response: ByteBuf,
    },
    #[fp(rename_all = "camelCase")]
    Other {
        reason: String,
    },
}

/// HTTP request method.
// Note: we use SCREAMING_SNAKE_CASE here because this is
// effectively a constant
#[derive(Serializable, Debug)]
#[fp(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(unused)]
pub enum HttpRequestMethod {
    Delete,
    Get,
    Head,
    Post,
}

/// Response to an HTTP request.
#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]
pub struct HttpResponse {
    pub body: ByteBuf,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}
