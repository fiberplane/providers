use crate::bindings::*;
use crate::providers::FORM_ENCODED_MIME_TYPE;
use crate::types::Result;
use serde::de::DeserializeOwned;

/// Parses a query data to a Serde struct.
///
/// You probably want to use the `QuerySchema` derive macro and use the struct's
/// `parse()` method.
pub fn parse_query<T: DeserializeOwned>(query_data: Blob) -> Result<T> {
    if query_data.mime_type != FORM_ENCODED_MIME_TYPE {
        return Err(Error::Data {
            message: format!("Incorrect MIME type: {}", query_data.mime_type),
        });
    }

    serde_html_form::from_bytes(&query_data.data).map_err(|err| Error::Other {
        message: err.to_string(),
    })
}
