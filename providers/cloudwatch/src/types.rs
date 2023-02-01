pub mod api;

use crate::{
    EXPRESSION_PARAM_NAME, LIST_METRICS_MIME_TYPE, PERIOD_PARAM_NAME, QUERY_DATA_MIME_TYPE,
    TIME_RANGE_PARAM_NAME,
};
use api::cloudwatch::{ListMetricsResponse, ListMetricsResult, Metric as SdkMetric};
use fiberplane_pdk::prelude::{log, Blob, Error, ProviderRequest, ValidationError};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};
use time::{
    ext::NumericalDuration, format_description::well_known::Rfc3339, Duration, OffsetDateTime,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricList {
    pub(crate) inner: Vec<Metric>,
}

impl std::fmt::Display for MetricList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.inner.iter().map(ToString::to_string).join("\n")
        )
    }
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Metric: {}\n\tNamespace: {}",
            self.name,
            self.namespace.as_deref().unwrap_or("<None>")
        )?;
        if self.dimensions.is_empty() {
            return Ok(());
        }
        write!(f, "\n\tDimensions:")?;
        self.dimensions
            .iter()
            .try_for_each(|(key, value)| write!(f, "\n\t\tName: {key}\n\t\tValue: {value}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub namespace: Option<String>,
    pub dimensions: HashMap<String, String>,
}

impl MetricList {
    // Not a TryFrom trait because that might depend on configuration
    pub fn try_into_blob(self) -> Result<Blob, Error> {
        Ok(Blob {
            mime_type: LIST_METRICS_MIME_TYPE.to_string(),
            data: serde_json::to_string(&self)
                .map_err(|e| Error::Invocation {
                    message: format!("could not serialize MetricList: {e}"),
                })?
                .into(),
        })
    }

    // Not a TryFrom trait because that might depend on configuration
    pub fn try_from_blob(blob: Blob) -> Result<Self, Error> {
        if blob.mime_type != LIST_METRICS_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        serde_json::from_slice(&blob.data).map_err(|e| Error::Deserialization {
            message: format!("could not deserialize blob into MetricList: {e}"),
        })
    }
}

impl From<&ListMetricsResult> for MetricList {
    fn from(output: &ListMetricsResult) -> Self {
        Self {
            inner: output.metrics.iter().map(Metric::from).collect(),
        }
    }
}

impl From<&ListMetricsResponse> for MetricList {
    fn from(lmr: &ListMetricsResponse) -> Self {
        Self::from(&lmr.list_metrics_result)
    }
}

impl From<&SdkMetric> for Metric {
    fn from(sdk: &SdkMetric) -> Self {
        Self {
            name: sdk.metric_name.as_ref().unwrap().to_string(),
            namespace: sdk.namespace.clone(),
            dimensions: sdk
                .dimensions
                .as_ref()
                .map(|dims| {
                    dims.iter()
                        .map(|dim| (dim.name.to_string(), dim.value.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphMetricQuery {
    metrics: Vec<String>,
    period: Duration,
    from: f64,
    to: f64,
}

impl GraphMetricQuery {
    fn parse_iso_date(timestamp: &str) -> Result<f64, Error> {
        OffsetDateTime::parse(timestamp, &Rfc3339)
            .map(|timestamp| timestamp.unix_timestamp_nanos() as f64 / 1_000_000_000.0)
            .map_err(|e| Error::Deserialization {
                message: format!("error reading timestamp from RFC3339 format: {e}"),
            })
    }

    fn to_iso_date(timestamp: f64) -> String {
        let time = SystemTime::UNIX_EPOCH + timestamp.seconds();
        OffsetDateTime::from(time)
            .format(&Rfc3339)
            .expect("Error formatting timestamp as RFC3339 timestamp")
    }

    // This is what could be automatically generated from SupportedQueryType
    pub fn try_from_url_query(request: ProviderRequest) -> Result<Self, Error> {
        if request.query_data.mime_type != QUERY_DATA_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        // Not using try_fold because we try to collect all errors instead of
        // short-circuiting
        let (query, errors) = form_urlencoded::parse(&request.query_data.data)
            .into_iter()
            .fold(
                (Self::default(), Vec::<ValidationError>::new()),
                |(mut acc, mut errors), (key, value)| {
                    match key.as_ref() {
                        EXPRESSION_PARAM_NAME => {
                            acc.metrics = value.lines().map(ToString::to_string).collect();
                        }
                        PERIOD_PARAM_NAME => match value.parse::<u32>() {
                            Ok(secs) => acc.period = Duration::seconds(secs.into()),
                            Err(e) => errors.push(ValidationError {
                                field_name: PERIOD_PARAM_NAME.to_string(),
                                message: format!("Invalid period: {e}"),
                            }),
                        },
                        TIME_RANGE_PARAM_NAME => {
                            if let Some(split) = value.split_once(' ') {
                                match Self::parse_iso_date(split.0) {
                                    Ok(ts) => acc.from = ts,
                                    Err(e) => errors.push(ValidationError {
                                        field_name: TIME_RANGE_PARAM_NAME.to_string(),
                                        message: format!("Invalid start of time range: {e}"),
                                    }),
                                };
                                match Self::parse_iso_date(split.1) {
                                    Ok(ts) => acc.to = ts,
                                    Err(e) => errors.push(ValidationError {
                                        field_name: TIME_RANGE_PARAM_NAME.to_string(),
                                        message: format!("Invalid end of time range: {e}"),
                                    }),
                                };
                            }
                            if acc.from >= acc.to {
                                errors.push(ValidationError {
                                    field_name: TIME_RANGE_PARAM_NAME.to_string(),
                                    message: format!(
                                        "Time range is invalid: {} is not strictly before {}",
                                        Self::to_iso_date(acc.from),
                                        Self::to_iso_date(acc.to)
                                    ),
                                })
                            }
                        }
                        unknown => {
                            log(format!(
                                "https provider received an unknown query parameter: {unknown}",
                            ));
                        }
                    }
                    (acc, errors)
                },
            );

        if !errors.is_empty() {
            return Err(Error::ValidationError { errors });
        }
        Ok(query)
    }
}
