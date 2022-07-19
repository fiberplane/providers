use crate::types::*;

/// Logs a message to the (development) console.
#[fp_bindgen_support::fp_import_signature]
pub fn log(message: String);

/// Performs an HTTP request.
#[fp_bindgen_support::fp_import_signature]
pub async fn make_http_request(request: HttpRequest) -> Result<HttpResponse, HttpRequestError>;

/// Returns the current timestamp.
#[fp_bindgen_support::fp_import_signature]
pub fn now() -> Timestamp;

/// Generates random bytes.
#[fp_bindgen_support::fp_import_signature]
pub fn random(len: u32) -> Vec<u8>;
