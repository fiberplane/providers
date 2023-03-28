use crate::schema_generator::generate_schema;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn derive_query_schema(input: TokenStream) -> TokenStream {
    let schema: proc_macro2::TokenStream = generate_schema("QueryField", input.clone()).into();
    let schema_struct = parse_macro_input!(input as ItemStruct);
    let ident = schema_struct.ident;

    let output = quote! {
        #[automatically_derived]
        impl #ident {
            pub fn parse(query_data: fiberplane_pdk::bindings::Blob)
                    -> fiberplane_pdk::prelude::Result<Self> {
                fiberplane_pdk::parse_query(query_data)
            }

            pub fn schema() -> fiberplane_pdk::providers::QuerySchema {
                use fiberplane_pdk::providers::*;

                #schema
            }
        }
    };
    output.into()
}
