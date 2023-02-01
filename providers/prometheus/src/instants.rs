use super::{constants::*, prometheus::*};
use fiberplane_models::utils::*;
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single data-point in time, with meta-data about the metric it was taken from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instant {
    pub name: String,
    pub labels: BTreeMap<String, String>,
    pub metric: Metric,
}

pub async fn query_instants(query_data: Blob, config: Config) -> Result<Blob> {
    if query_data.mime_type != FORM_ENCODED_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }

    let response: PrometheusResponse =
        query_direct_and_proxied(&config, "prometheus", "api/v1/query", Some(query_data)).await?;

    let instants = match response.data {
        PrometheusData::Vector(v) => v,
        PrometheusData::Matrix(_) => {
            return Err(Error::Data {
                message: "Expected a vector of instants, got a matrix".to_string(),
            })
        }
    };

    instants
        .into_iter()
        .map(InstantVector::into_instant)
        .collect::<Result<Vec<_>>>()
        .map(create_table_cell_for_instants)
        .and_then(|cell| {
            Ok(Blob {
                data: rmp_serde::to_vec_named(&[cell])?.into(),
                mime_type: CELLS_MSGPACK_MIME_TYPE.to_owned(),
            })
        })
}

fn create_table_cell_for_instants(instants: Vec<Instant>) -> Cell {
    let mut rows = vec![TableRow {
        cols: vec![
            TableColumn {
                formatting: Some(vec![AnnotationWithOffset {
                    annotation: Annotation::StartBold,
                    offset: 0,
                }]),
                text: "Element".to_owned(),
            },
            TableColumn {
                formatting: Some(vec![AnnotationWithOffset {
                    annotation: Annotation::StartBold,
                    offset: 0,
                }]),
                text: "Value".to_owned(),
            },
        ],
    }];

    for instant in instants {
        rows.push(TableRow {
            cols: vec![
                create_column_for_instant_name_and_labels(&instant),
                TableColumn {
                    formatting: None,
                    text: instant.metric.value.to_string(),
                },
            ],
        });
    }

    Cell::Table(TableCell {
        id: "instants".to_owned(),
        read_only: Some(true),
        rows,
    })
}

fn create_column_for_instant_name_and_labels(instant: &Instant) -> TableColumn {
    let mut formatting = Formatting::default();
    let mut text = instant.name.clone();

    let mut offset = char_count(&text);
    for (key, value) in &instant.labels {
        text.push(' ');
        offset += 1;

        // TODO: Use label annotation
        formatting.push(AnnotationWithOffset {
            offset,
            annotation: Annotation::StartHighlight,
        });

        let label = if value.is_empty() {
            key.clone()
        } else {
            format!("{key}={value}")
        };
        text.push_str(&label);
        offset += char_count(&label);

        formatting.push(AnnotationWithOffset {
            offset,
            annotation: Annotation::EndHighlight,
        });
    }

    TableColumn {
        formatting: Some(formatting),
        text,
    }
}
