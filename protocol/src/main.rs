mod types;

use fp_bindgen::{prelude::*, RustPluginConfig};
use std::collections::BTreeMap;
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
    async fn invoke(request: ProviderRequest, config: Config) -> ProviderResponse;
}

fn main() {
    const BINDINGS: &[(BindingsType, &str)] = &[
        (BindingsType::RustPlugin, "../fp-provider"),
        (
            BindingsType::RustWasmerRuntime,
            "../runtimes/fp-provider-runtime/src",
        ),
        (BindingsType::TsRuntime, "../runtimes/ts-runtime"),
    ];

    for (bindings_type, path) in BINDINGS {
        fp_bindgen!(BindingConfig {
            bindings_type: *bindings_type,
            path,
            rust_plugin_config: Some(RustPluginConfig {
                name: "fp-provider",
                authors: r#"["Fiberplane <info@fiberplane.com>"]"#,
                version: "1.0.0-alpha.1",
                dependencies: BTreeMap::new()
            })
        });
        println!("Generated bindings written to `{}/`.", path);
    }

    println!("Bindings generated.");
}
