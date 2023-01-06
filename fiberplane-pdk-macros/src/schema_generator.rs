use fiberplane_models::providers::*;
use proc_macro::TokenStream;
use proc_macro_error::{abort};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, token, Field, Ident, Result, Token, Type, PathArguments, GenericArgument};

/// Generates a schema from the given struct.
pub fn generate_schema(struct_item: TokenStream) -> TokenStream {
    let schema_struct = parse_macro_input!(struct_item as SchemaStruct);
    let fields: Vec<_> = schema_struct
        .fields
        .iter()
        .map(generate_schema_field)
        .collect();

    let ts = quote! { vec![#(#fields),*] };
    ts.into()
}

/// Represents the parsed struct from which to generate the schema.
struct SchemaStruct {
    struct_token: Token![struct],
    ident: Ident,
    brace_token: token::Brace,
    fields: Punctuated<Field, Token![,]>,
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

fn generate_schema_field(field: &Field) -> proc_macro2::TokenStream {
    let schema_field = determine_field_type(field);

    quote! {
        #field_type::new()
    }

    TextField::new()
            .with_name("baseUrl")
            .with_label("Base URL of the API we are interested in")
            .with_placeholder("Leave empty to allow querying any URL")
            .into(),
}

/// All the possible field types we can generate.
enum SchemaField {
    Checkbox(CheckboxField),
    DateTimeRange(DateTimeRangeField),
    Integer(IntegerField),
    Label(LabelField),
    Select(SelectField),
    Text(TextField),
}

impl SchemaField {
    pub fn required(self) -> Self {
        use SchemaField::*;
        match self {
            Checkbox(field) => Checkbox(field.required()),
            DateTimeRange(field) => DateTimeRange(field.required()),
            Integer(field) => Integer(field.required()),
            Label(field) => Label(field.required()),
            Select(field) => Select(field.required()),
            Text(field) => Text(field.required()),
        }
    }

    pub fn with_label(self, label: &str) -> Self {
        use SchemaField::*;
        match self {
            Checkbox(field) => Checkbox(field.with_label(label)),
            DateTimeRange(field) => DateTimeRange(field.with_label(label)),
            Integer(field) => Integer(field.with_label(label)),
            Label(field) => Label(field.with_label(label)),
            Select(field) => Select(field.with_label(label)),
            Text(field) => Text(field.with_label(label)),
        }
    }

    pub fn with_name(self, name: &str) -> Self {
        use SchemaField::*;
        match self {
            Checkbox(field) => Checkbox(field.with_name(name)),
            DateTimeRange(field) => DateTimeRange(field.with_name(name)),
            Integer(field) => Integer(field.with_name(name)),
            Label(field) => Label(field.with_name(name)),
            Select(field) => Select(field.with_name(name)),
            Text(field) => Text(field.with_name(name)),
        }
    }
}

fn determine_field_type(field: &Field) -> SchemaField {
    let path = match field.ty {
        Type::Path(type_path) if type_path.qself.is_none() => type_path.path,
        ty => abort!(ty, "unsupported type in schema")
    };

    let mut is_vec = false;
    let mut required = true;

    let (ident, arguments) = match path.segments.last() {
        Some(segment) => (segment.ident, segment.arguments),
        None => abort!(path, "unsupported type in schema")
    };

    // Extract a possibly nested identifier in the presence of an `Option`.
    // If an `Option` is present, the field will no longer be marked as required.
    let string_ident = ident.to_string();
    let string_ident = match (string_ident.as_str(), arguments) {
        ("Option", PathArguments::AngleBracketed(args)) => {
            required = false;

            let nested_path = match args.args.last() {
                Some(GenericArgument::Type(Type::Path(type_path))) if type_path.qself.is_none() => type_path.path,
                _ => abort!(path, "unsupported type in schema")
            };

            match path.segments.last() {
                Some(segment) if segment.arguments.is_none() => segment.ident.to_string(),
                _ => abort!(path, "unsupported type in schema")
            }
        }
        (string_ident, PathArguments::None) => string_ident.to_owned(),
        _ => abort!(path, "unsupported type in schema")
    };

    // Do the same for `Vec`, only now we set `is_vec` based on its presence.
    let string_ident = match (string_ident.as_str(), arguments) {
        ("Vec", PathArguments::AngleBracketed(args)) => {
            is_vec = true;

            let nested_path = match args.args.last() {
                Some(GenericArgument::Type(Type::Path(type_path))) if type_path.qself.is_none() => type_path.path,
                _ => abort!(path, "unsupported type in schema")
            };

            match path.segments.last() {
                Some(segment) if segment.arguments.is_none() => segment.ident.to_string(),
                _ => abort!(path, "unsupported type in schema")
            }
        }
        (string_ident, PathArguments::None) => string_ident.to_owned(),
        _ => abort!(path, "unsupported type in schema")
    };

    let name = match field.ident {
        Some(ident) => ident.to_string(),
        None => abort!(field, "struct field must have an identifier")
    };

    match (string_ident.as_str(), is_vec) {
        ("bool", false) => SchemaField::Checkbox(CheckboxField::new()),
        ("DateTimeRange", false) => SchemaField::DateTimeRange(DateTimeRangeField::new()),
        ("i8" | "i16" | "i32" | "i64", false) => SchemaField::Integer(IntegerField::new()),
        ("Label", is_vec) => {
            let mut field = LabelField::new();
            if is_vec {
                field = field.multiple();
            }
            SchemaField::Label(field)
        }
        ("Label", is_vec) => {
            let mut field = LabelField::new();
            if is_vec {
                field = field.multiple();
            }
            SchemaField::Label(field)
        }
        ("String", is_vec) => SchemaField::Text(TextField::new()),
        ("u8" | "u16" | "u32" | "u64", false) => {
            SchemaField::Integer(IntegerField::new().with_min(0))
        }
        _ => abort!(path, "unsupported type in schema")
    }
}
