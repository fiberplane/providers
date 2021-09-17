mod data_types;
mod request_types;

use data_types::*;
use fp_bindgen::prelude::*;
use request_types::*;

#[derive(Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum FetchError {
    RequestError { payload: RequestError },
    DataError { message: String },
    Other { message: String },
}

fp_import! {
    /// Logs a message to the (development) console.
    fn log(message: String);

    /// Performs an HTTP request.
    async fn make_request(request: Request) -> Result<Response, RequestError>;

    /// Returns the current timestamp.
    fn now() -> Timestamp;

    /// Generates random bytes.
    fn random(len: u32) -> Vec<u8>;
}

fp_export! {
    /// Fetches a data instant based on the given query and options.
    async fn fetch_instant(
        query: String,
        opts: QueryInstantOptions
    ) -> Result<Vec<Instant>, FetchError>;

    /// Fetches a series of data based on the given query and options.
    async fn fetch_series(
        query: String,
        opts: QuerySeriesOptions
    ) -> Result<Vec<Series>, FetchError>;
}

fn main() {
    fp_bindgen!("rust-plugin", "../fp-provider/src/bindings");
    fp_bindgen!("rust-wasmer-runtime", "../runtimes/fp-provider-runtime/src");
    fp_bindgen!("ts-runtime", "../runtimes/ts-runtime");

    println!("Bindings generated.");
}
