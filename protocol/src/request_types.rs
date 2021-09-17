use fp_bindgen::prelude::*;
use std::collections::HashMap;

/// HTTP request options.
#[derive(Serializable)]
pub struct Request {
    pub url: String,
    pub method: RequestMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

/// Possible errors that may happen during an HTTP request.
#[derive(Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    ServerError { status_code: u16, response: Vec<u8> },
    Other { reason: String },
}

/// HTTP request method.
#[derive(Serializable)]
#[fp(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(unused)]
pub enum RequestMethod {
    Delete,
    Get,
    Head,
    Post,
}

/// Response to an HTTP request.
#[derive(Serializable)]
pub struct Response {
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}
