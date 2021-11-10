use crate::types::*;

#[fp_bindgen_support::fp_import_signature]
/// Logs a message to the (development) console.
pub fn log(message: String);

#[fp_bindgen_support::fp_import_signature]
/// Performs an HTTP request.
pub async fn make_http_request(request: HttpRequest) -> Result<HttpResponse, HttpRequestError>;

#[fp_bindgen_support::fp_import_signature]
/// Returns the current timestamp.
pub fn now() -> Timestamp;

#[fp_bindgen_support::fp_import_signature]
/// Generates random bytes.
pub fn random(len: u32) -> Vec<u8>;
