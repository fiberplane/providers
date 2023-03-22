use crate::bindings::*;
use crate::providers::FORM_ENCODED_MIME_TYPE;
use crate::types::Result;
use serde::de::DeserializeOwned;

/// Parses a query data blob to a Serde struct.
///
/// You probably want to use the `QuerySchema` derive macro and use the struct's
/// `parse()` method.
pub fn parse_query<T: DeserializeOwned>(query_data: Blob) -> Result<T> {
    if query_data.mime_type != FORM_ENCODED_MIME_TYPE {
        return Err(Error::Data {
            message: format!("Incorrect MIME type: {}", query_data.mime_type),
        });
    }

    parse_bytes(&query_data.data)
}

fn parse_bytes<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
    serde_qs::from_bytes(data).map_err(|err| Error::Other {
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_bytes;
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
        let actual: ShowcaseQueryData = parse_bytes(input).unwrap();
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

    #[test]
    fn parse_array_query() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct RowQueryData {
            pub key: String,
            pub operator: String,
            pub value: i32,
        }

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct ArrayQueryData {
            pub key: String,
            pub time_range: DateTimeRange,
            pub values: Vec<RowQueryData>,
        }

        // Note: The input skips an index in the array (it gives 0 and 2)
        // Note: The rows aren't necessarily serialized adjacent to each other.
        let input = [
            b"key=Global+key&".as_ref(),
            b"values[0][key]=less+than&",
            b"values[2][operator]=%3E&",
            b"time_range=2023-02-08T09%3A16%3A27.794Z+2023-02-08T09%3A31%3A27.794Z&",
            b"values[0][operator]=%3C&",
            b"values[2][key]=greater+than&",
            b"values[2][value]=10&",
            b"values[0][value]=12",
        ]
        .concat();
        let actual: ArrayQueryData = parse_bytes(&input).unwrap();
        assert_eq!(
            actual,
            ArrayQueryData {
                key: "Global key".to_string(),
                time_range: DateTimeRange {
                    from: datetime!(2023-02-08 09:16:27.794 +00:00),
                    to: datetime!(2023-02-08 09:31:27.794 +00:00)
                },
                values: vec![
                    RowQueryData {
                        key: "less than".to_string(),
                        operator: "<".to_string(),
                        value: 12,
                    },
                    RowQueryData {
                        key: "greater than".to_string(),
                        operator: ">".to_string(),
                        value: 10,
                    }
                ]
            }
        );
    }
}
