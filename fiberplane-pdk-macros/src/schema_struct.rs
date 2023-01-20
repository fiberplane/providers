use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, token, Attribute, Field, Ident, Result, Token};

/// Represents the parsed struct from which to generate the schema.
pub struct SchemaStruct {
    pub attrs: Vec<Attribute>,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

impl Parse for SchemaStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(SchemaStruct {
            attrs: input.call(Attribute::parse_outer)?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse_named)?,
        })
    }
}
