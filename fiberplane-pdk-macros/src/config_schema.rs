use crate::schema_generator::generate_schema;
use proc_macro::TokenStream;
use quote::quote;

pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    let schema: proc_macro2::TokenStream = generate_schema("ConfigField", input).into();

    let output = quote! {
        #[pdk_export]
        fn get_config_schema() -> fiberplane_pdk::providers::ConfigSchema {
            #schema
        }
    };
    output.into()
}
