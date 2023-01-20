use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result};

#[derive(Debug)]
pub enum IdentOrString {
    Ident(Ident),
    String(String),
}

impl Parse for IdentOrString {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            Ok(Self::Ident(input.parse()?))
        } else {
            Ok(Self::String(
                input
                    .parse::<LitStr>()?
                    .to_token_stream()
                    .to_string()
                    .trim_matches('"')
                    .to_owned(),
            ))
        }
    }
}

impl ToTokens for IdentOrString {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::String(string) => string.to_tokens(tokens),
        }
    }
}
