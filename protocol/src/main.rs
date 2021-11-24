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
    {
        let path = "../fp-provider";
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::RustPlugin(RustPluginConfig {
                name: "fp-provider",
                authors: r#"["Fiberplane <info@fiberplane.com>"]"#,
                version: "1.0.0-alpha.1",
                dependencies: BTreeMap::new(),
            }),
            path,
        });
        println!("Rust plugin bindings written to `{}/`.", path);
    }

    {
        let path = "../runtimes/fp-provider-runtime/src";
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::RustWasmerRuntime,
            path,
        });
        println!("Rust Wasmer runtime bindings written to `{}/`.", path);
    }

    {
        let path = "../runtimes/ts-runtime";
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::TsRuntime(TsRuntimeConfig {
                generate_raw_export_wrappers: true,
            }),
            path,
        });
        println!("TypeScript runtime bindings written to `{}/`.", path);
    }
}
