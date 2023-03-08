use fiberplane_models::providers::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
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

    pub fn to_token_stream(
        &self,
        field_enum: &str,
        field_attrs: &[Attribute],
        struct_attrs: &[Attribute],
    ) -> proc_macro2::TokenStream {
        let field_attrs = SerdeAttrs::from_attrs(field_attrs);
        let struct_attrs = SerdeAttrs::from_attrs(struct_attrs);

        use SchemaField::*;

        let field_variant = match &self {
            Checkbox(_) => quote! { Checkbox },
            DateTimeRange(_) => quote! { DateTimeRange },
            Integer(_) => quote! { Integer },
            Label(_) => quote! { Label },
            Select(_) => quote! { Select },
            Text(_) => quote! { Text },
        };
        let enum_ident = Ident::new(field_enum, Span::call_site());
        let field_ident = Ident::new(&format!("{field_variant}Field"), Span::call_site());

        let name = field_attrs.rename.unwrap_or_else(|| {
            let name = match &self {
                Checkbox(field) => &field.name,
                DateTimeRange(field) => &field.name,
                Integer(field) => &field.name,
                Label(field) => &field.name,
                Select(field) => &field.name,
                Text(field) => &field.name,
            };
            struct_attrs.rename_all.format_string(name)
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
        };
        let label = match label.is_empty() {
            true => quote! {},
            false => quote! { .with_label(#label) },
        };

        let max = match &self {
            Integer(IntegerField { max: Some(max), .. }) => quote! { .with_max(#max) },
            _ => quote! {},
        };

        let min = match &self {
            Integer(IntegerField { min: Some(min), .. }) => quote! { .with_min(#min) },
            _ => quote! {},
        };

        let multiline = match &self {
            Text(field) if field.multiline => quote! { .multiline() },
            _ => quote! {},
        };

        let options = match &self {
            Select(SelectField { options, .. }) if !options.is_empty() => {
                quote! { .with_options(&[#(#options),*]) }
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
        };
        let placeholder = match placeholder.is_empty() {
            true => quote! {},
            false => quote! { .with_placeholder(#placeholder) },
        };

        let prerequisites = match &self {
            Select(SelectField { prerequisites, .. }) | Text(TextField { prerequisites, .. })
                if !prerequisites.is_empty() =>
            {
                quote! { .with_prerequisites(&[#(#prerequisites),*]) }
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

        quote! {
            #enum_ident::#field_variant(#field_ident::new()
                #name
                #checked
                #label
                #max
                #min
                #multiline
                #options
                #placeholder
                #prerequisites
                #required
                #step
                #supports_suggestions
                #value)
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

    pub fn with_placeholder(self, name: &str) -> Self {
        use SchemaField::*;
        match self {
            Checkbox(_) => self,
            DateTimeRange(field) => DateTimeRange(field.with_placeholder(name)),
            Integer(_) => self,
            Label(field) => Label(field.with_placeholder(name)),
            Select(field) => Select(field.with_placeholder(name)),
            Text(field) => Text(field.with_placeholder(name)),
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
