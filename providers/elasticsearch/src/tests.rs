use super::*;
use elasticsearch_dsl::{Hit, Hits, Relation, SearchResponse, Shards, Total};
use fp_provider::common::mem::FatPtr;
use serde_json::json;
use std::iter::FromIterator;
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
fn extracts_timestamp_and_body_fields() {
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
    let record = parse_hit(hit, &"timestamp".to_string(), &"body".to_string()).unwrap();
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
    let record = parse_hit(
        hit,
        &"@timestamp".to_string(),
        &"fields.my_body".to_string(),
    )
    .unwrap();
    assert_eq!(
        record.timestamp,
        OffsetDateTime::parse("2020-01-01T00:00:00Z", &Rfc3339)
            .unwrap()
            .unix_timestamp() as f64
    );
    assert_eq!(record.body, "test");
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

#[test]
fn sorts_logs_by_timestamp_newest_first() {
    let hit = |timestamp: &str, body: &str| Hit {
        index: None,
        id: "".to_string(),
        score: None,
        source: Some(Document {
            fields: Map::from_iter([
                ("timestamp".to_string(), json!(timestamp)),
                ("body".to_string(), json!(body)),
            ]),
        }),
        highlight: Default::default(),
        inner_hits: None,
        matched_queries: Default::default(),
        sort: Default::default(),
        fields: Default::default(),
    };
    let response: SearchResponse<Document, Document> = SearchResponse {
        took: 0,
        timed_out: false,
        shards: Shards {
            skipped: 0,
            failures: None,
            total: 0,
            successful: 0,
            failed: 0,
        },
        aggregations: None,
        hits: Hits {
            total: Some(Total {
                value: 3,
                relation: Relation::Equal,
            }),
            max_score: None,
            hits: vec![
                hit("2020-12-20T16:59:32.739Z", "2"),
                hit("2020-12-20T15:59:32.739Z", "3"),
                hit("2020-12-25T15:59:32.739Z", "1"),
            ],
        },
    };
    let logs = parse_response("timestamp", "body", response).unwrap();
    assert_eq!(logs[0].body, "1");
    assert_eq!(logs[1].body, "2");
    assert_eq!(logs[2].body, "3");
}
