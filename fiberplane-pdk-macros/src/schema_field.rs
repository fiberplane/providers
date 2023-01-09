use fiberplane_models::providers::*;
use quote::quote;

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

    pub fn to_token_stream(self) -> proc_macro2::TokenStream {
        use SchemaField::*;

        let field_type = match &self {
            Checkbox(_) => quote! { CheckboxField },
            DateTimeRange(_) => quote! { DateTimeRangeField },
            Integer(_) => quote! { IntegerField },
            Label(_) => quote! { LabelField },
            Select(_) => quote! { SelectField },
            Text(_) => quote! { TextField },
        };

        let name = match &self {
            Checkbox(field) => &field.name,
            DateTimeRange(field) => &field.name,
            Integer(field) => &field.name,
            Label(field) => &field.name,
            Select(field) => &field.name,
            Text(field) => &field.name,
        };
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
                quote! { .with_options(#(#options),*) }
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
                quote! { .with_prerequisites(#(#prerequisites),*) }
            }
            _ => quote! {},
        };

        let step = match &self {
            Integer(IntegerField {
                step: Some(step), ..
            }) => quote! { .with_step(#step) },
            _ => quote! {},
        };

        let value = match &self {
            Checkbox(CheckboxField { value, .. }) if !value.is_empty() => quote! {
                .with_value(#value)
            },
            _ => quote! {},
        };

        quote! {
            #field_type::new()
                #name
                #checked
                #label
                #max
                #min
                #multiline
                #options
                #placeholder
                #prerequisites
                #step
                #value
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
