use super::*;
use time::OffsetDateTime;

#[test]
fn timestamp_deserializes_from_unix_epoch() {
    let string = r#"{"timestamp": 1640010678}"#;
    let document: Document = serde_json::from_str(string).unwrap();
    assert_eq!(
        document.timestamp,
        Timestamp::Unix(OffsetDateTime::from_unix_timestamp(1640010678).unwrap())
    );
}

#[test]
fn timestamp_deserializes_from_rfc3339() {
    let string = r#"{"timestamp": "2021-12-20T15:59:32.739Z"}"#;
    let document: Document = serde_json::from_str(string).unwrap();
    assert_eq!(
        document.timestamp,
        Timestamp::Rfc3339(OffsetDateTime::parse("2021-12-20T15:59:32.739Z", &Rfc3339).unwrap())
    );
}
