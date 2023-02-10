use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    braced, bracketed, parenthesized, parse_macro_input, parse_quote, token, Expr, Result, Token,
};

use crate::ident_or_string::IdentOrString;

pub fn define_query_types(input: TokenStream) -> TokenStream {
    let QueryTypes { query_types, .. } = parse_macro_input!(input as QueryTypes);
    if query_types.is_empty() {
        abort_call_site!("no query types given");
    }

    let supported_query_types: Vec<_> = query_types
        .iter()
        .map(|query_type| {
            let identifier = &query_type.identifier;

            let schema = query_type
                .handler()
                .as_ref()
                .and_then(|handler| handler.arg_types.first())
                .map(|query_type| {
                    quote! { .with_schema(#query_type::schema()) }
                });

            let label = query_type.label().map(|label| {
                quote! { .with_label(#label) }
            });

            let mime_types = query_type
                .mime_types()
                .map(|supported_mime_types| supported_mime_types.mime_types)
                .map(|mime_types| {
                    quote! { .supporting_mime_types(&[#mime_types]) }
                });

            quote! {
                SupportedQueryType::new(#identifier)
                    #label
                    #schema
                    #mime_types
            }
        })
        .collect();

    let handlers: Vec<_> = query_types
        .iter()
        .map(|query_type| {
            let identifier = &query_type.identifier;

            let handler = query_type.handler().map(|handler| {
                let fn_name = handler.ident;
                let args = handler.arg_types.iter().enumerate().map(|(pos, ty)| {
                    let parse_arg = match pos {
                        0 => quote! { request.query_data },
                        1 => quote! { request.config },
                        _ => abort_call_site!("handlers may specify at most two arguments"),
                    };
                    quote! { #ty::parse(#parse_arg)? }
                });
                if handler.is_async {
                    quote! { #fn_name(#(#args),*).await }
                } else {
                    quote! { #fn_name(#(#args),*) }
                }
            });

            quote! { #identifier => #handler }
        })
        .collect();

    let output = quote! {
        #[pdk_export]
        async fn get_supported_query_types(_config: fiberplane_pdk::providers::ProviderConfig)
            -> Vec<fiberplane_pdk::providers::SupportedQueryType> {
            vec![
                #(#supported_query_types),*
            ]
        }

        #[pdk_export]
        async fn invoke2(request: fiberplane_pdk::providers::ProviderRequest)
            -> fiberplane_pdk::prelude::Result<fiberplane_pdk::bindings::Blob> {
            match request.query_type.as_str() {
                #(#handlers),*,
                _ => Err(Error::UnsupportedRequest)
            }
        }
    };
    output.into()
}

struct QueryTypes {
    query_types: Punctuated<QueryType, Token![,]>,
}

impl Parse for QueryTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            query_types: input.parse_terminated(QueryType::parse)?,
        })
    }
}

struct QueryType {
    identifier: IdentOrString,
    _arrow: token::FatArrow,
    _braces: token::Brace,
    fields: Punctuated<QueryTypeField, Token![,]>,
}

impl QueryType {
    fn handler(&self) -> Option<QueryHandler> {
        self.fields
            .iter()
            .find(|field| field.name == "handler")
            .map(|field| {
                let value = &field.value;
                parse_quote! { #value }
            })
    }

    fn label(&self) -> Option<IdentOrString> {
        self.fields
            .iter()
            .find(|field| field.name == "label")
            .map(|field| {
                let value = &field.value;
                parse_quote! { #value }
            })
    }

    fn mime_types(&self) -> Option<SupportedMimeTypes> {
        self.fields
            .iter()
            .find(|field| field.name == "supported_mime_types")
            .map(|field| {
                let value = &field.value;
                parse_quote! { #value }
            })
    }
}

impl Parse for QueryType {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            identifier: input.parse()?,
            _arrow: input.parse()?,
            _braces: braced!(content in input),
            fields: content.parse_terminated(QueryTypeField::parse)?,
        })
    }
}

struct QueryTypeField {
    name: Ident,
    _colon: token::Colon,
    value: Expr,
}

impl Parse for QueryTypeField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct QueryHandler {
    is_async: bool,
    ident: Ident,
    _parens: token::Paren,
    arg_types: Punctuated<Ident, Token![,]>,
}

impl Parse for QueryHandler {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let ident = input.parse::<Ident>()?;
        let _parens = parenthesized!(content in input);
        let arg_types = content.parse_terminated(Ident::parse)?;
        let is_async = if input.peek(Token![.]) && input.peek2(Token![await]) {
            input.parse::<Token![.]>()?;
            input.parse::<Token![await]>()?;
            true
        } else {
            false
        };

        Ok(Self {
            is_async,
            ident,
            _parens,
            arg_types,
        })
    }
}

struct SupportedMimeTypes {
    _brackets: token::Bracket,
    mime_types: Punctuated<IdentOrString, Token![,]>,
}

impl Parse for SupportedMimeTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            _brackets: bracketed!(content in input),
            mime_types: content.parse_terminated(IdentOrString::parse)?,
        })
    }
}
