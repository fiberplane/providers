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

#[cfg(test)]
mod tests {
    use crate::prelude::DateTimeRange;
    use serde::Deserialize;
    use time::macros::datetime;

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct ShowcaseQueryData {
        pub query: String,
        pub time_range: DateTimeRange,
        #[serde(default)]
        pub live: bool,
        pub tags: String,
    }

    #[test]
    fn parse_sample_provider_query() {
        let input = b"query=I%27m+a+pretty+query%21%21&tags=Test&time_range=2023-02-08T09%3A16%3A27.794Z+2023-02-08T09%3A31%3A27.794Z";
        let actual: ShowcaseQueryData = serde_html_form::from_bytes(input).unwrap();
        assert_eq!(
            actual,
            ShowcaseQueryData {
                query: "I'm a pretty query!!".to_string(),
                time_range: DateTimeRange {
                    from: datetime!(2023-02-08 09:16:27.794 +00:00),
                    to: datetime!(2023-02-08 09:31:27.794 +00:00)
                },
                live: false,
                tags: "Test".to_string()
            }
        );
    }
}
