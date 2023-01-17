use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn derive_provider_data(input: TokenStream) -> TokenStream {
    let schema_struct = parse_macro_input!(input as ItemStruct);
    let ident = schema_struct.ident;

    let output = quote! {
        impl #ident {
            pub fn to_blob(&self)
                    -> fiberplane_pdk::prelude::Result<fiberplane_pdk::bindings::Blob> {
                fiberplane_pdk::serde_json::from_value(config).map_err(|err| Error::Config {
                    message: format!("Error parsing config: {:?}", err),
                })
            }
        }
    };
    output.into()
}
