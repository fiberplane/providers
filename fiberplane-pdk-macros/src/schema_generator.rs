use crate::field_attrs::FieldAttrs;
use crate::schema_field::SchemaField;
use fiberplane_models::providers::*;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse_macro_input, Field, GenericArgument, ItemStruct, PathArguments, PathSegment, Type,
};

/// Generates a schema from the given struct.
pub fn generate_schema(field_enum: &str, struct_item: TokenStream) -> TokenStream {
    let schema_struct = parse_macro_input!(struct_item as ItemStruct);
    let fields: Vec<_> = schema_struct
        .fields
        .iter()
        .map(|field: &Field| {
            let schema_field = determine_field_type(field);
            schema_field.to_token_stream(field_enum, &field.attrs, &schema_struct.attrs)
        })
        .collect();

    let ts = quote! { vec![#(#fields),*] };
    ts.into()
}

fn determine_field_type(field: &Field) -> SchemaField {
    let name = match &field.ident {
        Some(ident) => ident.to_string(),
        None => abort!(field, "struct field must have an identifier"),
    };

    let attrs = FieldAttrs::from_attrs(&field.attrs);

    let (type_ident, required, multiple) = get_ident(field);

    let mut schema_field = match (type_ident.as_str(), multiple) {
        ("bool", false) => {
            let mut field =
                CheckboxField::new().with_value(attrs.value.as_deref().unwrap_or("true"));
            if attrs.checked {
                field = field.checked_by_default();
            }
            SchemaField::Checkbox(field)
        }
        ("i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64", false) => {
            let mut field = IntegerField::new();
            if let Some(max) = attrs.max {
                field = field.with_max(max);
            }
            if let Some(min) = attrs.min {
                field = field.with_min(min);
            } else if type_ident.as_str().starts_with("u") {
                field = field.with_min(0);
            }
            if let Some(step) = attrs.step {
                field = field.with_step(step);
            }
            SchemaField::Integer(field)
        }
        ("DateTimeRange", false) => SchemaField::DateTimeRange(DateTimeRangeField::new()),
        ("Label", multiple) => {
            let mut field = LabelField::new();
            if multiple {
                field = field.multiple();
            }
            SchemaField::Label(field)
        }
        ("String", multiple) => {
            if attrs.select {
                let mut field = SelectField::new();
                if multiple {
                    field = field.multiple();
                }
                if !attrs.options.is_empty() {
                    field = field
                        .with_options(&attrs.options.iter().map(String::as_str).collect::<Vec<_>>())
                }
                if !attrs.prerequisites.is_empty() {
                    field = field.with_prerequisites(
                        &attrs
                            .prerequisites
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                    );
                }
                if attrs.supports_suggestions {
                    field = field.with_suggestions();
                }
                SchemaField::Select(field)
            } else {
                let mut field = TextField::new();
                if attrs.multiline {
                    field = field.multiline();
                }
                if multiple {
                    field = field.multiple();
                }
                if !attrs.prerequisites.is_empty() {
                    field = field.with_prerequisites(
                        &attrs
                            .prerequisites
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                    );
                }
                if attrs.supports_suggestions {
                    field = field.with_suggestions();
                }
                SchemaField::Text(field)
            }
        }
        _ => abort!(field.ty, "unsupported type in schema"),
    };

    if let Some(label) = attrs.label {
        schema_field = schema_field.with_label(&label);
    }

    if let Some(placeholder) = attrs.placeholder {
        schema_field = schema_field.with_placeholder(&placeholder);
    }

    if required {
        schema_field = schema_field.required();
    }

    schema_field.with_name(&name)
}

fn get_ident(field: &Field) -> (String, bool, bool) {
    let path = match &field.ty {
        Type::Path(type_path) if type_path.qself.is_none() => &type_path.path,
        ty => abort!(ty, "unsupported type in schema"),
    };

    let mut multiple = false;
    let mut required = !path.is_ident("bool");

    let Some(mut path_segment) = path.segments.last().cloned() else {
        abort!(path, "unsupported type in schema")
    };

    // If the type is wrapped in an `Option`, it means the field is not required
    // and we use the nested type instead.
    if let Some(nested_segment) = get_nested_type(&path_segment, "Option") {
        path_segment = nested_segment;
        required = false;
    }

    // If the type is wrapped in a `Vec`, it means multiple values are allowed
    // and we use the nested type instead.
    if let Some(nested_segment) = get_nested_type(&path_segment, "Vec") {
        path_segment = nested_segment;
        multiple = true;
    }

    if !path_segment.arguments.is_none() {
        abort!(path_segment, "unsupported type in schema")
    }

    (path_segment.ident.to_string(), required, multiple)
}

fn get_nested_type(segment: &PathSegment, container_ident: &str) -> Option<PathSegment> {
    if segment.ident == container_ident {
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            let Some(GenericArgument::Type(Type::Path(nested_path))) = args.args.last() else {
                abort!(segment, "unsupported type in schema")
            };

            if nested_path.qself.is_some() {
                abort!(segment, "unsupported type in schema")
            }

            return nested_path.path.segments.last().cloned();
        }
    }

    None
}
