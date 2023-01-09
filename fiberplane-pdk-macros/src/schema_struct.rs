use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{token, Field, Ident, Result, Token};

/// Represents the parsed struct from which to generate the schema.
pub struct SchemaStruct {
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

impl Parse for SchemaStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            input.parse()
        } else {
            Err(lookahead.error())
        }
    }
}
