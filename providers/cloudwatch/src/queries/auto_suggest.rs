//! Auto suggestions implementation

use crate::{
    client::cloudwatch::Client,
    client::cloudwatch_logs::Client as LogsClient,
    client::resource_groups_tagging::Client as TagsClient,
    config::Config,
    constants::{
        DISCOVERABLE_FIELDS, EXPRESSION_PARAM_NAME, GRAPH_METRIC_QUERY_TYPE,
        LIST_METRICS_QUERY_TYPE, LOG_GROUP_PARAM_NAME, PERIOD_PARAM_NAME, QUERY_PARAM_NAME,
        START_LOG_QUERY_QUERY_TYPE, SUGGESTIONS_MSGPACK_MIME_TYPE, TAG_KEY_PARAM_NAME,
        TAG_VALUE_PARAM_NAME,
    },
};
use fiberplane_pdk::prelude::*;
use itertools::Itertools;

pub async fn invoke2_handler(query_data: Blob, config: Config) -> Result<Blob> {
    let query = AutoSuggestRequest::from_query_data(&query_data)?;

    let suggestions = match query.query_type.as_str() {
        LIST_METRICS_QUERY_TYPE => list_metrics_suggestions(query, config).await?,
        START_LOG_QUERY_QUERY_TYPE => list_log_query_suggestions(query, config).await?,
        GRAPH_METRIC_QUERY_TYPE => list_graph_metric_suggestions(query, config).await?,
        unknown => {
            log(format!(
                "Received a suggestion query for unsupported query type {unknown}"
            ));

            return Err(Error::UnsupportedRequest);
        }
    };

    Ok(Blob {
        data: rmp_serde::to_vec_named(&suggestions)?.into(),
        mime_type: SUGGESTIONS_MSGPACK_MIME_TYPE.to_owned(),
    })
}

async fn list_metrics_suggestions(
    query: AutoSuggestRequest,
    config: Config,
) -> Result<Vec<Suggestion>> {
    match query.field.as_str() {
        TAG_KEY_PARAM_NAME => {
            let mut keys = TagsClient::from(&config).get_tag_keys(None).await?;

            keys.retain(|k| k.contains(&query.query));
            Ok(keys
                .into_iter()
                .map(|k| Suggestion {
                    from: Some(0),
                    text: k,
                    description: None,
                })
                .collect())
        }
        TAG_VALUE_PARAM_NAME => {
            let mut values = TagsClient::from(&config)
                .get_tag_values("FP.Cluster".to_string(), None)
                .await?;

            values.retain(|v| v.contains(&query.query));
            Ok(values
                .into_iter()
                .map(|v| Suggestion {
                    from: Some(0),
                    text: v,
                    description: None,
                })
                .collect())
        }
        _unknown => Err(Error::UnsupportedRequest),
    }
}

async fn list_log_query_suggestions(
    query: AutoSuggestRequest,
    config: Config,
) -> Result<Vec<Suggestion>> {
    match query.field.as_str() {
        LOG_GROUP_PARAM_NAME => {
            let identifier = query
                .query
                .rsplit(|c: char| !(c.is_ascii_alphabetic() || ['/', '_', '-'].contains(&c)))
                .next()
                .unwrap_or_default();
            let from = query.query.len() - identifier.len();
            let keys = LogsClient::from(&config)
                .list_log_groups(Some(identifier.to_string()), None)
                .await?;

            Ok(keys
                .into_iter()
                .map(|k| Suggestion {
                    from: Some(from.try_into().unwrap()),
                    text: k
                        .log_group_name
                        .expect("All log groups returned by AWS have a name"),
                    description: k.stored_bytes.map(|bytes| format!("{bytes} bytes")),
                })
                .collect())
        }
        QUERY_PARAM_NAME => {
            if query.query.ends_with(|c: char| !c.is_ascii_alphabetic()) {
                return Ok(Vec::new());
            }

            let identifier = query
                .query
                .rsplit(|c: char| c.is_whitespace())
                .next()
                .map(ToString::to_string)
                .unwrap_or_default();
            let from = query.query.len() - identifier.len();

            Ok(DISCOVERABLE_FIELDS
                .iter()
                .filter_map(|(field, description)| {
                    if field.starts_with(&identifier) {
                        Some(Suggestion {
                            from: Some(from.try_into().unwrap()),
                            text: field.to_string(),
                            description: Some(description.to_string()),
                        })
                    } else {
                        None
                    }
                })
                .collect())
        }
        _unknown => Ok(Vec::new()),
    }
}

async fn list_graph_metric_suggestions(
    query: AutoSuggestRequest,
    config: Config,
) -> Result<Vec<Suggestion>> {
    match query.field.as_str() {
        EXPRESSION_PARAM_NAME => {
            let identifier = query
                .query
                .rsplit(|c: char| !c.is_ascii_alphabetic())
                .next()
                .map(|ident| ident.to_lowercase())
                .unwrap_or_default();
            if identifier.is_empty() {
                return Ok(Vec::new());
            }
            let from = query.query.len() - identifier.len();

            let mut keys = Client::from(&config)
                .list_metrics(None, None, None, None)
                .await?;

            keys.retain(|k| {
                k.metric_name
                    .as_ref()
                    .map_or(false, |name| name.to_lowercase().contains(&identifier))
            });

            Ok(keys
                .into_iter()
                .map(|k| Suggestion {
                    from: Some(from.try_into().unwrap()),
                    text: k
                        .metric_name
                        .expect("All metrics returned by AWS have a name"),
                    description: k.namespace,
                })
                .unique_by(|suggestion| (suggestion.text.clone(), suggestion.description.clone()))
                .collect())
        }
        PERIOD_PARAM_NAME => {
            let values = ["1", "5", "10", "30", "60"];

            Ok(values
                .iter()
                .map(|v| Suggestion {
                    from: Some(0),
                    text: v.to_string(),
                    description: None,
                })
                .collect())
        }
        _unknown => Err(Error::UnsupportedRequest),
    }
}
