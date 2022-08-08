use fp_bindgen::prelude::Serializable;

/// Defines the fields that should be included in the query data.
///
/// Note that query data is encoded as "application/x-www-form-urlencoded",
/// unless a `File` field is present in the schema, in which case
/// "multipart/form-data" may be used.
pub type QuerySchema = Vec<QueryField>;

#[derive(Debug, Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum QueryField {
    Button(ButtonField),
    Checkbox(CheckboxField),
    Date(DateField),
    DateTime(DateTimeField),
    DateTimeRange(DateTimeRangeField),
    File(FileField),
    Label(LabelField),
    Number(NumberField),
    Select(SelectField),
    Text(TextField),
}

#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct ButtonField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display on the button.
    label: String,

    /// Value of the button as it will be included in the encoded query. By
    /// checking whether the field with the given `name` has this `value`,
    /// providers may know which button was pressed.
    value: String,
}

#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct CheckboxField {
    /// Whether the checkbox should be initially checked if no query data is
    /// present.
    checked: bool,

    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the checkbox.
    label: String,

    /// Value of the field as it will be included in the encoded query. Note
    /// that only checked checkboxes will be included.
    value: String,
}

/// Defines a field that produces a date value in `YYYY-MM-DD` format.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct DateField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field.
    label: String,

    /// Whether a value is required.
    required: bool,
}

/// Defines a field that produces a date-time value that is valid RFC 3339 as
/// well as valid ISO 8601-1:2019.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct DateTimeField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field.
    label: String,

    /// Whether a value is required.
    required: bool,
}

/// Defines a field that produces two `DateTime` values, a "from" and a "to"
/// value, separated by a space.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct DateTimeRangeField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field.
    label: String,

    /// Whether a value is required.
    required: bool,
}

/// Defines a field that allows files to be uploaded as part of the query data.
///
/// Note that query data that includes files will be encoded as
/// "multipart/form-data".
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct FileField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field.
    label: String,

    /// Whether multiple files may be uploaded.
    multiple: bool,

    /// Whether a file is required.
    required: bool,
}

/// Defines a field that allows labels to be selected.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct LabelField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field (not to be confused with
    /// labels to be selected).
    label: String,

    /// Whether multiple labels may be selected.
    multiple: bool,

    /// Whether a value is required.
    required: bool,
}

/// Defines a field that allows labels to be selected.
///
/// Note that because the value is encoded as a string anyway, and depending on
/// the chosen `step` this field can be used for either integers or floating
/// point numbers, the values in the schema are simply presented as strings.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct NumberField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field.
    label: String,

    /// Optional maximum value to be selected.
    max: Option<String>,

    /// Optional minimal value to be selected.
    min: Option<String>,

    /// Whether a value is required.
    required: bool,

    /// Specifies the granularity that any specified numbers must adhere to.
    ///
    /// If omitted, `step` defaults to "1", meaning only integers are allowed.
    step: Option<String>,
}

/// Defines a field that allows selection from a predefined list of options.
///
/// Note that values to be selected from can be either hard-coded in the schema,
/// or fetched on-demand the same way as auto-suggestions.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct SelectField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the field.
    label: String,

    /// Whether multiple values may be selected.
    multiple: bool,

    /// A list of options to select from. If empty, the auto-suggest mechanism
    /// is used to fetch options as needed.
    options: Vec<String>,

    /// An optional list of fields that should be filled in before allowing the
    /// user to fill in this field. This forces a certain ordering in the data
    /// entry, which enables richer auto-suggestions, as the filled in
    /// prerequisite fields can provide additional context.
    prerequisites: Vec<String>,

    /// Whether a value is required.
    required: bool,
}

/// Defines a free-form text entry field.
///
/// Is commonly used for filter text and query entry. For the latter case,
/// `supports_highlighting` can be set to `true` if the provider supports syntax
/// highlighting for the query language.
#[derive(Debug, Default, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct TextField {
    /// Name of the field as it will be included in the encoded query.
    name: String,

    /// Suggested label to display along the form field.
    label: String,

    /// Suggests whether multi-line input is useful for this provider.
    multiline: bool,

    /// An optional list of fields that should be filled in before allowing the
    /// user to fill in this field. This forces a certain ordering in the data
    /// entry, which enables richer auto-suggestions, as the filled in
    /// prerequisite fields can provide additional context.
    prerequisites: Vec<String>,

    /// Whether a value is required.
    required: bool,

    /// Whether the provider implements syntax highlighting for this field.
    /// See `highlight_field()` in the protocol definition.
    supports_highlighting: bool,
}
