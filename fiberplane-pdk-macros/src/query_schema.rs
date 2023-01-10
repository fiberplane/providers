use crate::schema_generator::generate_schema;
use crate::schema_struct::SchemaStruct;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn derive_query_schema(input: TokenStream) -> TokenStream {
    let schema: proc_macro2::TokenStream = generate_schema("QueryField", input.clone()).into();
    let schema_struct = parse_macro_input!(input as SchemaStruct);
    let ident = schema_struct.ident;

    let output = quote! {
        impl #ident {
            pub fn parse(query_data: fiberplane_pdk::prelude::Blob)
                    -> core::result::Result<Self, fiberplane_pdk::bindings::Error> {
                fiberplane_pdk::parse_query(query_data)
            }

            pub fn schema() -> fiberplane_pdk::providers::QuerySchema {
                #schema
            }
        }
    };
    output.into()
}
