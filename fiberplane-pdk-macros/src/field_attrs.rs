use proc_macro_error::abort;
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Attribute, Error, Ident, LitInt, LitStr, Result, Token};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct FieldAttrs {
    /// Whether the checkbox should be initially checked.
    ///
    /// Only supported on checkbox fields.
    pub checked: bool,

    /// Suggested label to display along the field.
    ///
    /// Supported on any field type for which a `label` can be specified in the
    /// schema.
    pub label: Option<String>,

    /// Optional maximum value to be entered.
    ///
    /// Only supported on integer fields.
    pub max: Option<i32>,

    /// Optional minimum value to be entered.
    ///
    /// Only supported on integer fields.
    pub min: Option<i32>,

    /// Whether multi-line input is useful for this field.
    ///
    /// Only supported on text fields.
    pub multiline: bool,

    /// List of options from which to choose.
    ///
    /// Only supported on select fields. Multiple options are specified using
    /// multiple `option = "..."` annotations.
    pub options: Vec<String>,

    /// Suggested placeholder to display when there is no value for the field.
    ///
    /// Supported on any field type for which a `placeholder` can be specified
    /// in the schema.
    pub placeholder: Option<String>,

    /// An optional list of fields that should be filled in before allowing the
    /// user to fill in this field.
    ///
    /// Supported on any field type for which `prerequisites` can be specified
    /// in the schema. Multiple prerequisites are specified using multiple
    /// `prerequisite = "..."` annotations.
    pub prerequisites: Vec<String>,

    /// Indicates that this is a select field. Both text fields and select
    /// fields accept values of type `String`, so the `select` attribute is
    /// distinguish between them.
    ///
    /// Note that select fields that accept multiple values use `Vec<String>`
    /// as a type. These still require the use of the `select` attribute.
    pub select: bool,

    /// Specifies the granularity that any specified numbers must adhere to.
    ///
    /// Only supported on integer fields.
    pub step: Option<i32>,

    /// Specifies the field supports suggestions.
    ///
    /// Only supported on select fields and text fields.
    pub supports_suggestions: bool,

    /// Value of the field as it will be included in the encoded query.
    ///
    /// Only supported on checkbox fields. If omitted, a default value of "true"
    /// is used.
    pub value: Option<String>,
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
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

impl Parse for FieldAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let parse_i32 = || -> Result<i32> {
            content.parse::<Token![=]>()?;
            content
                .parse::<LitInt>()?
                .to_token_stream()
                .to_string()
                .parse()
                .map_err(|err| {
                    Error::new(content.span(), format!("Expected a number, found: {err}"))
                })
        };

        let parse_string = || -> Result<String> {
            content.parse::<Token![=]>()?;
            Ok(content
                .parse::<LitStr>()?
                .to_token_stream()
                .to_string()
                .trim_matches('"')
                .to_owned())
        };

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match key.to_string().as_ref() {
                "checked" | "checked_by_default" => result.checked = true,
                "label" => result.label = Some(parse_string()?),
                "max" => result.max = Some(parse_i32()?),
                "min" => result.min = Some(parse_i32()?),
                "multiline" => result.multiline = true,
                "option" => result.options.push(parse_string()?),
                "prerequisite" => result.prerequisites.push(parse_string()?),
                "placeholder" => result.placeholder = Some(parse_string()?),
                "select" => result.select = true,
                "step" => result.step = Some(parse_i32()?),
                "supports_suggestions" => result.supports_suggestions = true,
                "value" => result.value = Some(parse_string()?),
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
