use super::HttpRequestError;
use fp_bindgen::prelude::Serializable;

#[derive(Debug, Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Error {
    UnsupportedRequest,
    #[fp(rename_all = "camelCase")]
    Http {
        error: HttpRequestError,
    },
    #[fp(rename_all = "camelCase")]
    Data {
        message: String,
    },
    #[fp(rename_all = "camelCase")]
    Deserialization {
        message: String,
    },
    #[fp(rename_all = "camelCase")]
    Config {
        message: String,
    },
    #[fp(rename_all = "camelCase")]
    Other {
        message: String,
    },
}
