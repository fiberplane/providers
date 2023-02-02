use super::{constants::*, prometheus::*};
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde::Deserialize;
use serde_json::Result as SerdeResult;
use std::time::SystemTime;
use time::{ext::NumericalDuration, format_description::well_known::Rfc3339, OffsetDateTime};

struct SeriesRequest {
    query: String,
    from: f64,
    to: f64,
}

#[derive(Clone, Copy)]
struct StepSize {
    amount: u32,
    unit: StepUnit,
}

impl ToString for StepSize {
    fn to_string(&self) -> String {
        format!("{}{}", self.amount, self.unit.to_str())
    }
}

#[derive(Clone, Copy)]
enum StepUnit {
    Hours,
    Minutes,
    Seconds,
}

impl StepUnit {
    fn to_str(self) -> &'static str {
        match self {
            Self::Hours => "h",
            Self::Minutes => "m",
            Self::Seconds => "s",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    error: Option<String>,
}

#[derive(Deserialize)]
enum Status {
    Error,
    Success,
}

fn validate_or_parse_message(query: &str, message: &str) -> Error {
    // Validate first
    match prometheus_parser::parse_expr(query) {
        Ok(_) => {
            let result: SerdeResult<ApiResponse> = serde_json::from_str(message);
            if let Ok(response) = result {
                // No validation error?
                // Then we should parse the returned result
                // from Prometheus
                if let Some(error) = response.error {
                    return Error::ValidationError {
                        errors: vec![ValidationError {
                            field_name: QUERY_PARAM_NAME.to_owned(),
                            message: error,
                        }],
                    };
                }
            }
        }
        Err(error) => {
            return Error::ValidationError {
                errors: vec![ValidationError {
                    field_name: QUERY_PARAM_NAME.to_owned(),
                    message: format!("{error}"),
                }],
            };
        }
    };

    Error::Other {
        message: message.to_owned(),
    }
}

pub async fn query_series(query_data: Blob, config: Config) -> Result<Blob> {
    let request = parse_metrics_request(query_data)?;
    let step = step_for_range(request.from, request.to);
    let start = to_iso_date(round_to_grid(request.from, step, RoundToGridEdge::Start));
    let end = to_iso_date(round_to_grid(request.to, step, RoundToGridEdge::End));

    let mut form_data = form_urlencoded::Serializer::new(String::new());
    form_data.append_pair("query", &request.query);
    form_data.append_pair("start", &start);
    form_data.append_pair("end", &end);
    form_data.append_pair("step", &step.to_string());
    let query_string = form_data.finish();

    let body = Blob {
        data: query_string.into_bytes().into(),
        mime_type: FORM_ENCODED_MIME_TYPE.to_owned(),
    };

    let response: PrometheusResponse =
        query_direct_and_proxied(&config, "prometheus", "api/v1/query_range", Some(body))
            .await
            .map_err(|err| match err {
                Error::Other { message } => validate_or_parse_message(&request.query, &message),
                err => err,
            })?;

    let PrometheusData::Matrix(matrix) = response.data;

    matrix
        .into_iter()
        .map(RangeVector::into_series)
        .collect::<core::result::Result<Vec<_>, Error>>()
        .and_then(|series_vector| {
            Ok(Blob {
                data: rmp_serde::to_vec_named(&series_vector)?.into(),
                mime_type: TIMESERIES_MSGPACK_MIME_TYPE.to_owned(),
            })
        })
}

pub fn create_graph_cell() -> Result<Vec<Cell>> {
    let graph_cell = Cell::Graph(GraphCell {
        id: "graph".to_owned(),
        data_links: vec![format!("cell-data:{TIMESERIES_MIME_TYPE},self")],
        graph_type: GraphType::Line,
        read_only: None,
        stacking_type: StackingType::None,
    });
    Ok(vec![graph_cell])
}

fn parse_metrics_request(query_data: Blob) -> Result<SeriesRequest> {
    if query_data.mime_type != FORM_ENCODED_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }

    let mut query = String::new();
    let mut from = 0.0;
    let mut to = 0.0;
    for (key, value) in form_urlencoded::parse(&query_data.data) {
        match key.as_ref() {
            QUERY_PARAM_NAME => query = value.to_string(),
            TIME_RANGE_PARAM_NAME => {
                if let Some(split) = value.split_once(' ') {
                    from = from_iso_date(split.0)?;
                    to = from_iso_date(split.1)?;
                }
            }
            _ => {}
        }
    }

    let mut errors = Vec::new();
    if query.is_empty() {
        errors.push(ValidationError {
            field_name: QUERY_PARAM_NAME.to_owned(),
            message: "Please enter a query".to_owned(),
        });
    }
    if from == 0.0 || to == 0.0 {
        errors.push(ValidationError {
            field_name: TIME_RANGE_PARAM_NAME.to_owned(),
            message: "Please enter a valid time range".to_owned(),
        });
    }

    if !errors.is_empty() {
        return Err(Error::ValidationError { errors });
    }

    Ok(SeriesRequest { query, from, to })
}

enum RoundToGridEdge {
    Start,
    End,
}

/// Rounds the timestamp to a "grid" with intervals defined by the step size.
/// This assures that when we scroll a chart forward or backward in time, we
/// "snap" to the same grid, to avoid the issue of bucket realignment, giving
/// unexpected jumps in the graph.
fn round_to_grid(timestamp: f64, step: StepSize, edge: RoundToGridEdge) -> f64 {
    let step_seconds = step_to_seconds(step);
    let round = match edge {
        RoundToGridEdge::Start => f64::floor,
        RoundToGridEdge::End => f64::ceil,
    };
    round(timestamp / step_seconds as f64) * step_seconds as f64
}

fn step_to_seconds(step: StepSize) -> u32 {
    match step.unit {
        StepUnit::Hours => ONE_HOUR * step.amount,
        StepUnit::Minutes => ONE_MINUTE * step.amount,
        StepUnit::Seconds => step.amount,
    }
}

/// Returns the step to fetch from the given duration in seconds. We attempt
/// to maintain roughly 30 steps for whatever the duration is, so that for a
/// duration of one hour, we fetch per 2 minutes, and for a duration of one
/// minute, we fetch per 2 seconds.
fn step_for_range(from: f64, to: f64) -> StepSize {
    let mut step = (to - from) / 30.0;
    let mut unit = StepUnit::Seconds;
    if step >= 60.0 {
        step /= 60.0;
        unit = StepUnit::Minutes;
        if step >= 60.0 {
            step /= 60.0;
            unit = StepUnit::Hours;
        }
    }

    StepSize {
        amount: f64::ceil(2.0 * step) as u32,
        unit,
    }
}

fn from_iso_date(timestamp: &str) -> core::result::Result<f64, time::error::Parse> {
    OffsetDateTime::parse(timestamp, &Rfc3339)
        .map(|timestamp| timestamp.unix_timestamp_nanos() as f64 / 1_000_000_000.0)
}

fn to_iso_date(timestamp: f64) -> String {
    let time = SystemTime::UNIX_EPOCH + timestamp.seconds();
    OffsetDateTime::from(time)
        .format(&Rfc3339)
        .expect("Error formatting timestamp as RFC3339 timestamp")
}