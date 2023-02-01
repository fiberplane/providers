//! List Metrics query handling
use crate::{
    api::resource_groups_tagging::TagFilter,
    client::{cloudwatch::Client, resource_groups_tagging::Client as TagsClient},
    config::Config,
    constants::{TAG_KEY_PARAM_NAME, TAG_VALUE_PARAM_NAME},
    MetricList,
};
use fiberplane_models::providers::FORM_ENCODED_MIME_TYPE;
use fiberplane_provider_bindings::{Blob, Cell, Error, ProviderRequest, TextCell};

pub async fn invoke2_handler(config: Config, request: ProviderRequest) -> Result<Blob, Error> {
    let request = ListMetricsRequest::try_from(request.query_data)?;
    let client = Client::from(&config);

    // This is an async Option::map
    let relevant_resources = match request.tag_filter {
        Some(filter) => Some(
            TagsClient::from(&config)
                .get_resources(
                    None,
                    None,
                    None,
                    None,
                    Some(vec![TagFilter {
                        key: filter.0.into(),
                        values: filter.1.map(|value| vec![value]),
                    }]),
                    None,
                )
                .await?,
        ),
        None => None,
    };
    client
        .list_metrics(None, None, None, None)
        .await
        .map(|mut metrics| {
            if let Some(resource_list) = relevant_resources {
                metrics.retain(|metric| {
                    metric.dimensions.as_ref().map_or(false, |dims| {
                        dims.iter().any(|d| {
                            resource_list.iter().any(|resource| {
                                resource.resource_arn.is_some()
                                    && resource.resource_arn.as_ref().unwrap().ends_with(&d.value)
                            })
                        })
                    })
                })
            }
            metrics
        })
        .and_then(|metrics| {
            MetricList {
                inner: metrics.iter().map(crate::types::Metric::from).collect(),
            }
            .try_into_blob()
        })
}

pub fn create_cells_handler(response: Blob) -> Result<Vec<Cell>, Error> {
    let list = MetricList::try_from_blob(response)?;
    Ok(vec![Cell::Text(TextCell {
        id: "metric-list".to_string(),
        content: list.to_string(),
        formatting: Vec::new(),
        read_only: Some(true),
    })])
}

struct ListMetricsRequest {
    tag_filter: Option<(String, Option<String>)>,
}

impl TryFrom<Blob> for ListMetricsRequest {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type != FORM_ENCODED_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        let mut tag_filter = None;
        for (key, value) in form_urlencoded::parse(&blob.data) {
            match key.as_ref() {
                TAG_KEY_PARAM_NAME => {
                    if value.is_empty() {
                        continue;
                    }
                    if tag_filter.is_none() {
                        tag_filter = Some((Some(value.to_string()), None));
                    } else {
                        tag_filter.as_mut().unwrap().0 = Some(value.to_string())
                    }
                }
                TAG_VALUE_PARAM_NAME => {
                    if value.is_empty() {
                        continue;
                    }
                    if tag_filter.is_none() {
                        tag_filter = Some((None, Some(value.to_string())));
                    } else {
                        tag_filter.as_mut().unwrap().1 = Some(value.to_string())
                    }
                }
                _ => {}
            }
        }

        // TODO: Validation checks for non empty mandatory values.

        Ok(Self {
            tag_filter: tag_filter.map(|(k, v)| (k.unwrap(), v)),
        })
    }
}
