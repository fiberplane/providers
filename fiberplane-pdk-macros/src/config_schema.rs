use crate::schema_generator::generate_schema;
use crate::schema_struct::SchemaStruct;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    let schema: proc_macro2::TokenStream = generate_schema("ConfigField", input.clone()).into();
    let schema_struct = parse_macro_input!(input as SchemaStruct);
    let ident = schema_struct.ident;

    let output = quote! {
        impl #ident {
            pub fn parse(config: fiberplane_pdk::providers::ProviderConfig)
                    -> core::result::Result<Self, fiberplane_pdk::bindings::Error> {
                fiberplane_pdk::serde_json::from_value(config).map_err(|err| Error::Config {
                    message: format!("Error parsing config: {:?}", err),
                })
            }
        }

        #[pdk_export]
        fn get_config_schema() -> fiberplane_pdk::providers::ConfigSchema {
            #schema
        }
    };
    output.into()
}
