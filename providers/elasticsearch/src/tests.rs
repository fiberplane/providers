use super::*;
use fp_provider::common::mem::FatPtr;
use serde_json::json;
use time::OffsetDateTime;

#[no_mangle]
unsafe fn __fp_gen_log(_: FatPtr) {}

#[no_mangle]
unsafe fn __fp_host_resolve_async_value(_: FatPtr, _: FatPtr) {}

#[no_mangle]
unsafe fn __fp_gen_make_http_request(_: FatPtr) -> FatPtr {
    todo!()
}

#[test]
fn timestamp_deserializes_from_unix_epoch() {
    let js = json!({
        "_id": "foobar",
        "_source": {
            "@timestamp": 1640010678u32,
            "my_body": "",
        },
    });

    let document: Hit<Document, Document> = serde_json::from_value(js).unwrap();
    let record = parse_hit(document, &"@timestamp".to_string(), &"my_body".to_string()).unwrap();

    assert_eq!(record.timestamp, 1640010678f64);
}

#[test]
fn timestamp_deserializes_from_rfc3339() {
    let js = json!({
        "_id": "foobar",
        "_source": {
            "@timestamp": "2021-12-20T15:59:32.739Z",
            "my_body": "",
        },
    });

    let document: Hit<Document, Document> = serde_json::from_value(js).unwrap();
    let record = parse_hit(document, &"@timestamp".to_string(), &"my_body".to_string()).unwrap();

    assert_eq!(
        record.timestamp,
        OffsetDateTime::parse("2021-12-20T15:59:32.739Z", &Rfc3339)
            .unwrap()
            .unix_timestamp() as f64
    );
}
