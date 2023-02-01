use std::collections::BTreeMap;

use super::{canonical_request::request_state, CanonicalRequest, ClientCommon};
use crate::api::paginate::{paginate, paginate_vec, Paginate};
use crate::{api::SdkResponse, api::TaggedSdkResponse};
use crate::{config::Config, types::api::cloudwatch::*};
use fiberplane_provider_bindings::{Error, Timestamp};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) common: ClientCommon,
}

impl Client {
    /// Build an AWS SDK Cloudwatch client relying on runtime provided http client.
    fn from_config(config: &Config) -> Self {
        Self {
            common: ClientCommon {
                service: "monitoring".to_string(),
                host: format!("monitoring.{}.amazonaws.com", config.region),
                endpoint: format!("https://monitoring.{}.amazonaws.com", config.region),
                region: config.region.clone(),
                access_key_id: config.access_key_id.clone(),
                secret_access_key: config.secret_access_key.clone(),
            },
        }
    }

    fn format_action_header(action: &str) -> String {
        format!("{}.{}", X_AMZ_TARGET_PREFIX, action)
    }

    /// Note: Using a limit of `Some(0)` means only a single call is made to the API
    pub async fn list_metrics(
        &self,
        dimensions: Option<Vec<DimensionFilter>>,
        metric_name: Option<String>,
        namespace: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<Metric>, Error> {
        let init_request = ListMetrics {
            dimensions: dimensions.clone(),
            metric_name: metric_name.clone(),
            namespace: namespace.clone(),
            next_token: None,
        };
        return paginate_vec(
            &self.common,
            init_request,
            |response: SdkResponse| {
                if let SdkResponse::Tagged(TaggedSdkResponse::ListMetricsResponse(
                    list_metrics_response,
                )) = response
                {
                    Some(
                        list_metrics_response
                            .list_metrics_result
                            .metrics
                            .into_iter(),
                    )
                } else {
                    None
                }
            },
            |response| {
                if let SdkResponse::Tagged(TaggedSdkResponse::ListMetricsResponse(
                    list_metrics_response,
                )) = response
                {
                    list_metrics_response.list_metrics_result.next_token.clone()
                } else {
                    None
                }
            },
            limit,
        )
        .await;
    }

    pub async fn get_metric_data(
        &self,
        expressions: Vec<(String, Option<String>, usize)>,
        start_time: Timestamp,
        end_time: Timestamp,
        max_datapoints: Option<u64>,
        order_points_by: Option<ScanOrder>,
    ) -> Result<Vec<MetricDataResult>, Error> {
        let metric_data_queries: Vec<MetricDataQuery> = expressions
            .into_iter()
            .enumerate()
            .map(|(id, (expr, label, period))| MetricDataQuery {
                account_id: None,
                expression: Some(expr),
                id: crate::utils::sluggify(label.as_deref().unwrap_or_default(), || {
                    format!("expr_{id}")
                }),
                label,
                metric_stat: None,
                period_secs: Some(period),
                return_data: Some(true),
            })
            .collect();
        assert!(!metric_data_queries.is_empty());

        let init_request = GetMetricDataRequest {
            end_time: end_time.into(),
            label_options: None, // TODO: pass the timezone parameter to the call to use here.
            max_datapoints,
            metric_data_queries,
            next_token: None,
            scan_by: order_points_by,
            start_time: start_time.into(),
        };

        return paginate(
            &self.common,
            init_request,
            |metric_data: GetMetricDataResponse| Some(metric_data.metric_data_results.into_iter()),
            |metric_data| metric_data.next_token.clone(),
            |acc: &mut BTreeMap<String, MetricDataResult>, metric_data_results| {
                for partial_result in metric_data_results {
                    let key = format!(
                        "{}{}",
                        partial_result.id.clone().unwrap(),
                        partial_result.label.clone().unwrap_or_default()
                    );
                    match acc.entry(key) {
                        std::collections::btree_map::Entry::Vacant(vacant) => {
                            vacant.insert(partial_result);
                        }
                        std::collections::btree_map::Entry::Occupied(mut occupied) => {
                            let inner_accumulator = occupied.get_mut();
                            if inner_accumulator.messages.is_none() {
                                inner_accumulator.messages = partial_result.messages;
                            } else {
                                inner_accumulator
                                    .messages
                                    .as_mut()
                                    .unwrap()
                                    .extend(partial_result.messages.clone().unwrap_or_default());
                            }
                            if inner_accumulator.timestamps.is_none() {
                                inner_accumulator.timestamps = partial_result.timestamps;
                            } else {
                                inner_accumulator
                                    .timestamps
                                    .as_mut()
                                    .unwrap()
                                    .extend(partial_result.timestamps.clone().unwrap_or_default());
                            }
                            if inner_accumulator.values.is_none() {
                                inner_accumulator.values = partial_result.values;
                            } else {
                                inner_accumulator
                                    .values
                                    .as_mut()
                                    .unwrap()
                                    .extend(partial_result.values.clone().unwrap_or_default());
                            }
                        }
                    }
                }
            },
            None,
        )
        .await
        .map(|metrics| metrics.into_values().collect());
    }
}

impl From<&Config> for Client {
    fn from(conf: &Config) -> Self {
        Self::from_config(conf)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMetrics {
    /// The dimensions to filter against. Only the dimensions that match exactly will be returned.
    /// Type: Array of DimensionFilter objects
    /// Array Members: Maximum number of 10 items.
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<Vec<DimensionFilter>>,
    /// The name of the metric to filter against. Only the metrics with names that match exactly will be returned.
    /// Length Constraints: Minimum length of 1. Maximum length of 255.
    #[serde(skip_serializing_if = "Option::is_none")]
    metric_name: Option<String>,
    /// The metric namespace to filter against. Only the namespace that matches exactly will be returned.
    /// Length Constraints: Minimum length of 1. Maximum length of 255.
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    /// The token returned by a previous call to indicate that there is more data available.
    #[serde(skip_serializing_if = "Option::is_none")]
    next_token: Option<String>,
}

impl From<ListMetrics> for CanonicalRequest<{ request_state::STEM }> {
    fn from(lm: ListMetrics) -> Self {
        let method = http::Method::GET;
        let uri = "/".to_string();
        let query_params = {
            let mut acc = BTreeMap::new();
            acc.insert("Action".to_string(), "ListMetrics".to_string());
            acc.insert("Version".to_string(), VERSION.to_string());
            if let Some(metric_name) = lm.metric_name {
                acc.insert("MetricName".to_string(), metric_name);
            }
            if let Some(namespace) = lm.namespace {
                acc.insert("Namespace".to_string(), namespace);
            }
            if let Some(next_token) = lm.next_token {
                acc.insert("NextToken".to_string(), next_token);
            }
            acc
        };
        let headers = BTreeMap::new();
        let body = None;

        CanonicalRequest::new(method, uri, query_params, headers, body)
    }
}

impl Paginate for ListMetrics {
    fn next_page(self, pagination_token: Option<String>) -> Option<Self> {
        pagination_token.and_then(|token| {
            if token.is_empty() {
                return None;
            }
            Some(Self {
                next_token: Some(token),
                ..self
            })
        })
    }
}

impl From<GetMetricDataRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: GetMetricDataRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&req).expect("GetMetricDataRequest is always serializable.");
        let headers = {
            let mut acc = BTreeMap::new();
            acc.insert(
                "x-amz-target".to_string(),
                Client::format_action_header("GetMetricData"),
            );
            acc.insert("content-type".to_string(), POST_CONTENT_TYPE.to_string());
            acc.insert("accept-encoding".to_string(), "identity".to_string());
            acc.insert(
                "content-encoding".to_string(),
                POST_CONTENT_ENCODING.to_string(),
            );
            acc
        };

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}

impl Paginate for GetMetricDataRequest {
    fn next_page(self, pagination_token: Option<String>) -> Option<Self> {
        pagination_token.and_then(|token| {
            if token.is_empty() {
                return None;
            }
            Some(Self {
                next_token: Some(token),
                ..self
            })
        })
    }
}
