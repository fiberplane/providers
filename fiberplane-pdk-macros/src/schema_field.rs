use fiberplane_models::providers::*;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::ext::IdentExt;
use syn::parenthesized;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Ident, LitStr, Result, Token};

use crate::casing::Casing;

/// All the possible field types we can generate.
pub enum SchemaField {
    Checkbox(CheckboxField),
    DateTimeRange(DateTimeRangeField),
    Integer(IntegerField),
    Label(LabelField),
    Select(SelectField),
    Text(TextField),
    Array(ArraySchema),
}

/// A compile-time representation of an ArrayField schema,
/// useful at macro expansion time to generate the correct
/// code.
///
/// This type is not meant to be interacted with outside
/// procedural macro code.
///
/// This is why all fields are only exposed to the crate,
/// and no public method is available either.
pub struct ArraySchema {
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) element_struct_type_name: String,
    pub(crate) minimum_length: u32,
    pub(crate) maximum_length: Option<u32>,
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
            Array(schema) => Array(ArraySchema {
                minimum_length: 1,
                ..schema
            }),
        }
    }

    /// Convert a SchemaField into a TokenStream representing a QueryField variant.
    pub fn to_token_stream(
        &self,
        field_enum: &str,
        field_attrs: &[Attribute],
        struct_attrs: &[Attribute],
    ) -> proc_macro2::TokenStream {
        let serde_field_attrs = SerdeAttrs::from_attrs(field_attrs);
        let serde_struct_attrs = SerdeAttrs::from_attrs(struct_attrs);

        use SchemaField::*;

        let field_variant = match &self {
            Checkbox(_) => quote! { Checkbox },
            DateTimeRange(_) => quote! { DateTimeRange },
            Integer(_) => quote! { Integer },
            Label(_) => quote! { Label },
            Select(_) => quote! { Select },
            Text(_) => quote! { Text },
            Array(_) => quote! { Array },
        };
        let enum_ident = Ident::new(field_enum, Span::call_site());
        let field_ident = Ident::new(&format!("{field_variant}Field"), Span::call_site());

        let name = serde_field_attrs.rename.unwrap_or_else(|| {
            let name = match &self {
                Checkbox(field) => &field.name,
                DateTimeRange(field) => &field.name,
                Integer(field) => &field.name,
                Label(field) => &field.name,
                Select(field) => &field.name,
                Text(field) => &field.name,
                Array(field) => &field.name,
            };
            serde_struct_attrs.rename_all.format_string(name)
        });
        let name = quote! { .with_name(#name) };

        let checked = match &self {
            Checkbox(field) if field.checked => quote! { .checked_by_default() },
            _ => quote! {},
        };

        let label = match &self {
            Checkbox(field) => &field.label,
            DateTimeRange(field) => &field.label,
            Integer(field) => &field.label,
            Label(field) => &field.label,
            Select(field) => &field.label,
            Text(field) => &field.label,
            Array(field) => &field.label,
        };
        let label = match label.is_empty() {
            true => quote! {},
            false => quote! { .with_label(#label) },
        };

        let max = match &self {
            Integer(IntegerField { max: Some(max), .. }) => quote! { .with_max(#max) },
            Array(ArraySchema {
                maximum_length: Some(maximum_length),
                ..
            }) => quote! { .with_maximum_length(#maximum_length) },
            _ => quote! {},
        };

        let min = match &self {
            Integer(IntegerField { min: Some(min), .. }) => quote! { .with_min(#min) },
            Array(ArraySchema { minimum_length, .. }) if *minimum_length != 0 => {
                quote! { .with_minimum_length(#minimum_length) }
            }
            _ => quote! {},
        };

        let multiple = match &self {
            Text(field) if field.multiple => quote! { .multiple() },
            Label(field) if field.multiple => quote! { .multiple() },
            Select(field) if field.multiple => quote! { .multiple() },
            _ => quote! {},
        };

        let multiline = match &self {
            Text(field) if field.multiline => quote! { .multiline() },
            _ => quote! {},
        };

        let options = match &self {
            Select(SelectField { options, .. }) if !options.is_empty() => {
                quote! { .with_options([#(#options.into()),*]) }
            }
            _ => quote! {},
        };

        let placeholder = match &self {
            Checkbox(_) => "",
            DateTimeRange(field) => &field.placeholder,
            Integer(_) => "",
            Label(field) => &field.placeholder,
            Select(field) => &field.placeholder,
            Text(field) => &field.placeholder,
            _ => "",
        };
        let placeholder = match placeholder.is_empty() {
            true => quote! {},
            false => quote! { .with_placeholder(#placeholder) },
        };

        let prerequisites = match &self {
            Select(SelectField { prerequisites, .. }) | Text(TextField { prerequisites, .. })
                if !prerequisites.is_empty() =>
            {
                quote! { .with_prerequisites([#(#prerequisites.into()),*]) }
            }
            _ => quote! {},
        };

        let required = match &self {
            Checkbox(field) => field.required,
            DateTimeRange(field) => field.required,
            Integer(field) => field.required,
            Label(field) => field.required,
            Select(field) => field.required,
            Text(field) => field.required,
            Array(_) => false,
        };
        let required = match required {
            true => quote! { .required() },
            false => quote! {},
        };

        let step = match &self {
            Integer(IntegerField {
                step: Some(step), ..
            }) => quote! { .with_step(#step) },
            _ => quote! {},
        };

        let supports_suggestions = match &self {
            Select(SelectField {
                supports_suggestions,
                ..
            })
            | Text(TextField {
                supports_suggestions,
                ..
            }) if *supports_suggestions => {
                quote! { .with_suggestions() }
            }
            _ => quote! {},
        };

        let value = match &self {
            Checkbox(CheckboxField { value, .. }) if !value.is_empty() => quote! {
                .with_value(#value)
            },
            _ => quote! {},
        };

        let element_schema = match &self {
            Array(ArraySchema {
                element_struct_type_name,
                ..
            }) => {
                let type_ident = format_ident!("{element_struct_type_name}");
                quote! {
                .with_element_schema(<#type_ident>::schema())
                }
            }
            _ => quote! {},
        };

        quote! {
            #enum_ident::#field_variant(#field_ident::new()
                #name
                #checked
                #label
                #max
                #min
                #multiple
                #multiline
                #options
                #placeholder
                #prerequisites
                #required
                #step
                #supports_suggestions
                #value
                #element_schema)
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
            Array(field) => Array(ArraySchema {
                label: label.to_string(),
                ..field
            }),
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
            Array(field) => Array(ArraySchema {
                name: name.to_string(),
                ..field
            }),
        }
    }

    pub fn with_placeholder(self, name: &str) -> Self {
        use SchemaField::*;
        match self {
            Checkbox(_) => self,
            DateTimeRange(field) => DateTimeRange(field.with_placeholder(name)),
            Integer(_) => self,
            Label(field) => Label(field.with_placeholder(name)),
            Select(field) => Select(field.with_placeholder(name)),
            Text(field) => Text(field.with_placeholder(name)),
            Array(_) => self,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SerdeAttrs {
    pub rename: Option<String>,
    pub rename_all: Casing,
}

impl SerdeAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        for attr in attrs {
            if attr.path.is_ident("serde") {
                return syn::parse2::<Self>(attr.tokens.clone())
                    .expect("Could not parse Serde field attributes");
            }
        }

        Self::default()
    }
}

impl Parse for SerdeAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let parse_value = || -> Result<String> {
            content.parse::<Token![=]>()?;
            Ok(content
                .parse::<LitStr>()?
                .to_token_stream()
                .to_string()
                .trim_matches('"')
                .to_owned())
        };

        let parse_optional_value = || -> Result<String> {
            if content.peek(Token![=]) {
                parse_value()
            } else {
                Ok(String::new())
            }
        };

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match key.to_string().as_ref() {
                "rename" => result.rename = Some(parse_value()?),
                "rename_all" => {
                    result.rename_all = Casing::try_from(parse_value()?.as_ref())
                        .map_err(|err| Error::new(content.span(), err))?
                }
                _ => {
                    parse_optional_value()?;
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
