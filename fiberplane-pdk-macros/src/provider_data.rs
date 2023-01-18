use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, parse_macro_input, Attribute, Error, Ident, ItemStruct, Result, Token};

use crate::ident_or_string::IdentOrString;

pub fn derive_provider_data(input: TokenStream) -> TokenStream {
    let schema_struct = parse_macro_input!(input as ItemStruct);
    let attrs = ProviderDataAttrs::from_attrs(&schema_struct.attrs);
    let ident = &schema_struct.ident;
    let Some(mime_type) = attrs.mime_type else {
        abort!(schema_struct, "Missing required PDK attribute: mime-type");
    };

    let output = quote! {
        impl #ident {
            pub fn parse(blob: fiberplane_pdk::bindings::Blob)
                    -> fiberplane_pdk::prelude::Result<Self> {
                fiberplane_pdk::provider_data::parse_blob(#mime_type, blob)
            }

            pub fn serialize(&self)
                    -> fiberplane_pdk::prelude::Result<fiberplane_pdk::bindings::Blob> {
                fiberplane_pdk::provider_data::to_blob(#mime_type, &self)
            }
        }
    };
    output.into()
}

#[derive(Default)]
struct ProviderDataAttrs {
    pub mime_type: Option<IdentOrString>,
}

impl ProviderDataAttrs {
    fn from_attrs(attrs: &[Attribute]) -> Self {
        attrs
            .iter()
            .find(|attr| attr.path.is_ident("pdk"))
            .map(|attr| {
                syn::parse2::<Self>(attr.tokens.clone())
                    .unwrap_or_else(|err| abort!(attr, "Cannot parse attribute: {}", err))
            })
            .unwrap_or_default()
    }
}

impl Parse for ProviderDataAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let parse_ident_or_string = || -> Result<IdentOrString> {
            content.parse::<Token![=]>()?;
            Ok(IdentOrString::Ident(content.parse()?))
        };

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match key.to_string().as_ref() {
                "mime_type" => result.mime_type = Some(parse_ident_or_string()?),
                other => {
                    return Err(Error::new(
                        content.span(),
                        format!("Unexpected attribute: {other}"),
                    ))
                }
            }

            if content.is_empty() {
                break;
            }

            content.parse::<Token![,]>()?;
        }

        Ok(result)
    }
}
