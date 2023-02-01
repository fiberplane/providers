use super::*;
use elasticsearch_dsl::{Hit, HitsMetadata, SearchResponse, TotalHits, TotalHitsRelation};
use fiberplane_provider_bindings::common::mem::FatPtr;
use serde_json::json;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[no_mangle]
unsafe fn __fp_gen_log(_: FatPtr) {}

#[no_mangle]
unsafe fn __fp_host_resolve_async_value(_: FatPtr, _: FatPtr) {}

#[no_mangle]
unsafe fn __fp_gen_make_http_request(_: FatPtr) -> FatPtr {
    todo!()
}

#[test]
fn flatten_nested_values() {
    let mut fields = HashMap::new();
    flatten_nested_value(&mut fields, "a".to_string(), json!(1));
    assert_eq!(fields.get("a").unwrap(), "1");

    flatten_nested_value(
        &mut fields,
        "b".to_string(),
        json!({
            "c": true
        }),
    );
    assert_eq!(fields.get("b.c").unwrap(), "true");

    flatten_nested_value(
        &mut fields,
        "e.f".to_string(),
        json!({
            "g": {
                "h": null
            }
        }),
    );
    assert_eq!(fields.get("e.f.g.h").unwrap(), "");

    flatten_nested_value(&mut fields, "j.arr".to_string(), json!(["apple", "banana"]));
    assert_eq!(fields.get("j.arr[0]").unwrap(), "apple");
    assert_eq!(fields.get("j.arr[1]").unwrap(), "banana");
}

#[test]
fn extracts_timestamp_and_body_from_fields() {
    let hit = serde_json::from_value(json!({
        "_index": "index",
        "_type": "type",
        "_id": "id",
        "_score": 1.0,
        "_source": {
            "timestamp": "2020-01-01T00:00:00Z",
            "body": "test",
        }
    }))
    .unwrap();
    let record = parse_hit(hit, TIMESTAMP_FIELDS, BODY_FIELDS).unwrap();
    assert_eq!(
        record.timestamp,
        OffsetDateTime::parse("2020-01-01T00:00:00Z", &Rfc3339)
            .unwrap()
            .unix_timestamp() as f64
    );
    assert_eq!(record.body, "test");

    let hit = serde_json::from_value(json!({
        "_index": "index",
        "_type": "type",
        "_id": "id",
        "_score": 1.0,
        "_source": {
            "@timestamp": "2020-01-01T00:00:00Z",
            "fields": {
                "my_body": "test",
            }
        }
    }))
    .unwrap();
    let record = parse_hit(hit, TIMESTAMP_FIELDS, &["body", "fields.my_body"]).unwrap();
    assert_eq!(
        record.timestamp,
        OffsetDateTime::parse("2020-01-01T00:00:00Z", &Rfc3339)
            .unwrap()
            .unix_timestamp() as f64
    );
    assert_eq!(record.body, "test");
}

#[test]
fn uses_default_values_if_timestamp_or_body_extraction_fails() {
    let hit = serde_json::from_value(json!({
        "_index": "index",
        "_type": "type",
        "_id": "id",
        "_score": 1.0,
        "_source": {
            "other-timestamp": "2020-01-01T00:00:00Z",
            "other-body": "test",
        }
    }))
    .unwrap();
    let record = parse_hit(hit, TIMESTAMP_FIELDS, BODY_FIELDS).unwrap();
    assert!(record.timestamp.is_nan());
    assert_eq!(record.body, "");
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

    let document: Hit = serde_json::from_value(js).unwrap();
    let record = parse_hit(document, TIMESTAMP_FIELDS, BODY_FIELDS).unwrap();

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

    let document: Hit = serde_json::from_value(js).unwrap();
    let record = parse_hit(document, TIMESTAMP_FIELDS, BODY_FIELDS).unwrap();

    assert_eq!(
        record.timestamp,
        OffsetDateTime::parse("2021-12-20T15:59:32.739Z", &Rfc3339)
            .unwrap()
            .unix_timestamp() as f64
    );
}

#[test]
fn sorts_logs_by_timestamp_newest_first() {
    let hit = |timestamp: &str, body: &str| {
        let hit = json!({
            "_source": {
            "timestamp": timestamp,
            "body": body,
            }
        });
        serde_json::from_value::<Hit>(hit).unwrap()
    };
    let response: SearchResponse = SearchResponse {
        hits: HitsMetadata {
            total: Some(TotalHits {
                value: 3,
                relation: TotalHitsRelation::Equal,
            }),
            max_score: None,
            hits: vec![
                hit("2020-12-20T16:59:32.739Z", "2"),
                hit("2020-12-20T15:59:32.739Z", "3"),
                hit("2020-12-25T15:59:32.739Z", "1"),
            ],
        },
        ..Default::default()
    };
    let logs = parse_response(response, TIMESTAMP_FIELDS, BODY_FIELDS).unwrap();
    assert_eq!(logs[0].body, "1");
    assert_eq!(logs[1].body, "2");
    assert_eq!(logs[2].body, "3");
}
