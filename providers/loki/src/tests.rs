use crate::{data_mapper, Data, QueryData, QueryResponse};
use fiberplane_provider_bindings::LegacyLogRecord as LogRecord;
use serde::Deserialize;
use serde_json::Deserializer;
use std::collections::HashMap;

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
                labels: HashMap::from([
                    ("filename".to_owned(), "/var/log/myproject.log".to_owned()),
                    ("job".to_owned(), "varlogs".to_owned()),
                    ("level".to_owned(), "info".to_owned()),
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
    if let QueryData::Streams(data) = &value.data {
        let mapped = data_mapper(&data[0])
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            mapped,
            vec![
                LogRecord {
                    timestamp: 1_569_266_497.240_578,
                    body: "foo".to_owned(),
                    attributes: data[0].labels.clone(),
                    span_id: None,
                    trace_id: None,
                    resource: HashMap::default(),
                },
                LogRecord {
                    timestamp: 1569266492.5481548, //not the exact value due to floating point precision
                    body: "bar".to_owned(),
                    attributes: data[0].labels.clone(),
                    span_id: None,
                    trace_id: None,
                    resource: HashMap::default(),
                }
            ]
        );
    } else {
        panic!("unexpected query data type");
    }
}
