mod types;

use fiberplane::protocols::{blobs::Blob, core::Cell};
use fp_bindgen::{prelude::*, types::CargoDependency};
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, BTreeSet};
use types::*;

fp_import! {
    /// Logs a message to the (development) console.
    fn log(message: String);

    /// Performs an HTTP request.
    async fn make_http_request(request: HttpRequest) -> Result<HttpResponse, HttpRequestError>;

    /// Returns the current timestamp.
    fn now() -> Timestamp;

    /// Generates random bytes.
    fn random(len: u32) -> Vec<u8>;
}

fp_export! {
    type Formatting = Vec<AnnotationWithOffset>;
    type QuerySchema = Vec<QueryField>;
    type Timestamp = f64;

    /// Returns the query types supported by this provider.
    /// This function allows Studio to know upfront which formats will be
    /// supported, and which providers (and their query types) are eligible to
    /// be selected for certain use cases.
    async fn get_supported_query_types(config: Value) -> Vec<SupportedQueryType>;

    /// Legacy invoke function.
    async fn invoke(request: LegacyProviderRequest, config: Value) -> LegacyProviderResponse;

    /// Invokes the provider to perform a data request.
    async fn invoke2(request: ProviderRequest) -> Result<Blob, Error>;

    /// Creates output cells based on the response.
    /// Studio would typically embed the created cells in the provider cell,
    /// but other actions could be desired.
    ///
    /// When any created cells use a `data` field with the value
    /// `cell-data:<mime-type>,self`, Studio will replace the value `self` with
    /// the ID of the cell for which the query was invoked. This allows the
    /// provider to create cells that reference its own data without knowing the
    /// context of the cell in which it was executed.
    ///
    /// Note: When the MIME type in the provider response is
    /// `application/vnd.fiberplane.cells` (suffixed with either `+json` or
    /// `+msgpack`), Studio will elide the call to `create_cells()` and simply
    /// parse the data directly to a `Vec<Cell>`.
    fn create_cells(query_type: String, response: Blob) -> Result<Vec<Cell>, Error>;

    /// Takes the response data, and returns it in the given MIME type,
    /// optionally passing an additional query string to customize extraction
    /// behavior.
    ///
    /// Returns `Err(Error::UnsupportedRequest)` if an unsupported MIME type is
    /// passed.
    ///
    /// Note: When the MIME type in the provider response is the same as the
    /// MIME type given as the second argument, and the `query` is omitted, the
    /// return value is expected to be equivalent to the raw response data. This
    /// means Studio should be allowed to elide calls to this function if there
    /// is no query string and the MIME type is an exact match. This elision
    /// should not change the outcome.
    fn extract_data(response: Blob, mime_type: String, query: Option<String>) -> Result<ByteBuf, Error>;
}

fn main() {
    {
        let dependencies = BTreeMap::from([
            (
                "fiberplane",
                fp_bindgen::types::CargoDependency {
                    git: Some("ssh://git@github.com/fiberplane/fiberplane-rs.git"),
                    branch: Some("main"),
                    ..Default::default()
                },
            ),
            (
                "rmpv",
                CargoDependency {
                    version: Some("1.0.0"),
                    features: BTreeSet::from(["with-serde"]),
                    ..Default::default()
                },
            ),
            (
                "serde_bytes",
                CargoDependency {
                    version: Some("0.11"),
                    ..Default::default()
                },
            ),
        ]);

        let path = "../fp-provider";
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::RustPlugin(RustPluginConfig {
                name: "fp-provider",
                authors: r#"["Fiberplane <info@fiberplane.com>"]"#,
                version: "2.0.0",
                dependencies,
            }),
            path,
        });
        println!("Rust plugin bindings written to `{}/`.", path);
    }

    {
        let path = "../runtimes/fp-provider-runtime/src/spec";
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::RustWasmerRuntime,
            path,
        });
        println!("Rust Wasmer runtime bindings written to `{}/`.", path);
    }

    {
        let path = "../runtimes/ts-runtime";
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::TsRuntimeWithExtendedConfig(
                TsExtendedRuntimeConfig::new().with_raw_export_wrappers()
            ),
            path,
        });
        println!("TypeScript runtime bindings written to `{}/`.", path);
    }
}
