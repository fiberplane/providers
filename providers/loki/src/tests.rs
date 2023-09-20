use crate::{data_mapper, Data, QueryData, QueryResponse};
use fiberplane_pdk::prelude::*;
use fiberplane_pdk::serde_json::{json, Deserializer};
use serde::Deserialize;
use std::collections::BTreeMap;
use time::OffsetDateTime;

const DATA: &str = r#"{
        "status": "success",
        "data": {
          "resultType": "streams",
          "result": [
            {
              "stream": {
                "filename": "/var/log/myproject.log",
                "job": "varlogs",
                "level": "info"
              },
              "values": [
                [
                  "1569266497240578000",
                  "foo"
                ],
                [
                  "1569266492548155000",
                  "bar"
                ]
              ]
            }
          ],
          "stats": {
          }
        }
      }"#;

#[test]
fn test_deserialization() {
    let value = QueryResponse::deserialize(&mut Deserializer::from_str(DATA)).unwrap();

    assert_eq!(
        value,
        QueryResponse {
            data: QueryData::Streams(vec![Data {
                labels: BTreeMap::from([
                    ("filename".to_owned(), json!("/var/log/myproject.log")),
                    ("job".to_owned(), json!("varlogs")),
                    ("level".to_owned(), json!("info")),
                ]),
                values: vec![
                    ("1569266497240578000".to_owned(), "foo".to_owned()),
                    ("1569266492548155000".to_owned(), "bar".to_owned()),
                ],
            }]),
            status: "success".to_owned(),
        },
    )
}

#[test]
fn test_data_mapper() {
    let value = QueryResponse::deserialize(&mut Deserializer::from_str(DATA)).unwrap();
    let QueryData::Streams(data) = &value.data else {
        panic!("unexpected query data type");
    };

    let mapped = data_mapper(&data[0]).collect::<Result<Vec<_>>>().unwrap();
    let metadata = OtelMetadata::builder()
        .attributes(data[0].labels.clone())
        .resource(BTreeMap::new())
        .build();
    assert_eq!(mapped.len(), 2);
    assert_eq!(
        mapped[0],
        ProviderEvent::builder()
            .time(OffsetDateTime::from_unix_timestamp_nanos(1_569_266_497_240_578_000).unwrap())
            .title("foo".to_owned())
            .otel(metadata.clone())
            .build()
    );
    assert_eq!(
        mapped[1],
        ProviderEvent::builder()
            .time(OffsetDateTime::from_unix_timestamp_nanos(1_569_266_492_548_155_000).unwrap())
            .title("bar".to_owned())
            .otel(metadata.clone())
            .build()
    );
}
