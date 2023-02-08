mod deserialize;
use serde::Serialize;
use time::OffsetDateTime;

/// Type to use for query data fields of type "date_time_range".
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DateTimeRange {
    /// Start time of the range, inclusive.
    pub from: OffsetDateTime,

    /// End time of the range, exclusive.
    pub to: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use time::macros::datetime;

    use super::*;
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    struct TargetType {
        range: DateTimeRange,
    }

    #[test]
    fn deserialize_from_url() {
        // URL encoding of "2023-02-08T09:16:27.794Z 2023-02-08T09:31:27.794Z"
        let input = "range=2023-02-08T09%3A16%3A27.794Z%202023-02-08T09%3A31%3A27.794Z";
        let target: TargetType = serde_html_form::from_str(input).unwrap();
        assert_eq!(
            target,
            TargetType {
                range: DateTimeRange {
                    from: datetime!(2023-02-08 09:16:27.794 +00:00),
                    to: datetime!(2023-02-08 09:31:27.794 +00:00)
                }
            }
        )
    }

    #[test]
    fn deserialize_from_bytes() {
        // URL encoding of "2023-02-08T09:16:27.794Z 2023-02-08T09:31:27.794Z"
        let input = b"range=2023-02-08T09%3A16%3A27.794Z%202023-02-08T09%3A31%3A27.794Z";
        let target: TargetType = serde_html_form::from_bytes(input).unwrap();
        assert_eq!(
            target,
            TargetType {
                range: DateTimeRange {
                    from: datetime!(2023-02-08 09:16:27.794 +00:00),
                    to: datetime!(2023-02-08 09:31:27.794 +00:00)
                }
            }
        )
    }
}
