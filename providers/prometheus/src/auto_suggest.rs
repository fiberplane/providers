use super::{constants::*, prometheus::*};
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};

/// See: https://prometheus.io/docs/prometheus/latest/querying/functions/
const PROM_QL_FUNCTIONS: &[&str] = &[
    "abs",
    "absent",
    "ceil",
    "changes",
    "clamp_max",
    "clamp_min",
    "day_of_month",
    "day_of_week",
    "days_in_month",
    "delta",
    "deriv",
    "exp",
    "floor",
    "histogram_quantile",
    "holt_winters",
    "hour",
    "idelta",
    "increase",
    "irate",
    "label_join",
    "label_replace",
    "ln",
    "log2",
    "log10",
    "minute",
    "month",
    "predict_linear",
    "rate",
    "resets",
    "round",
    "scalar",
    "sort",
    "sort_desc",
    "sqrt",
    "time",
    "timestamp",
    "vector",
    "year",
    "avg_over_time",
    "min_over_time",
    "max_over_time",
    "sum_over_time",
    "count_over_time",
    "quantile_over_time",
    "stddev_over_time",
    "stdvar_over_time",
];

pub async fn query_suggestions(query: AutoSuggestRequest, config: Config) -> Result<Blob> {
    let (identifier, from) = extract_identifier(&query.query);

    let response: PrometheusMetadataResponse =
        query_direct_and_proxied(&config, "prometheus", "api/v1/metadata", None).await?;

    let mut suggestions: Vec<Suggestion> = response
        .data
        .into_iter()
        .filter_map(|(name, values)| {
            values.into_iter().next().map(|value| {
                Suggestion::builder()
                    .from(from)
                    .text(name)
                    .description(value.help)
                    .build()
            })
        })
        .collect();

    if !identifier.is_empty() {
        suggestions.retain(|suggestion| {
            suggestion.text.contains(identifier)
                || suggestion
                    .description
                    .as_ref()
                    .map(|description| description.contains(identifier))
                    .unwrap_or_default()
        })
    }
    for &function in PROM_QL_FUNCTIONS {
        if identifier.is_empty() || function.contains(identifier) {
            suggestions.push(
                Suggestion::builder()
                    .from(from)
                    .text(function.to_owned())
                    .description(Some("Function".to_owned()))
                    .build(),
            )
        }
    }

    Ok(Blob::builder()
        .data(rmp_serde::to_vec_named(&suggestions)?)
        .mime_type(SUGGESTIONS_MSGPACK_MIME_TYPE.to_owned())
        .build())
}

/// Extracts the identifier and starting offset that is currently being typed from the query. This
/// identifier is used to filter the suggestions. If the identifier is empty,
/// no filtering would be applied.
fn extract_identifier(query: &str) -> (&str, Option<u32>) {
    let chars: Vec<char> = query.chars().collect();
    if let Some((offset, _)) = chars
        .iter()
        .enumerate()
        .rev()
        .find(|(_, &c)| !is_identifier_char(c))
    {
        (&query[(offset + 1)..], Some(offset as u32 + 1))
    } else {
        (query, Some(0))
    }
}

fn is_letter(c: char) -> bool {
    ('A'..='Z').contains(&c) || ('a'..='z').contains(&c)
}

fn is_number(c: char) -> bool {
    ('0'..='9').contains(&c)
}

fn is_identifier_char(c: char) -> bool {
    is_letter(c) || is_number(c) || c == '_'
}

#[cfg(test)]
mod tests {
    use crate::auto_suggest::extract_identifier;

    #[test]
    fn test_extract_identifier() {
        assert_eq!(extract_identifier("hello"), ("hello", Some(0)));
        assert_eq!(extract_identifier("hello foo"), ("foo", Some(6)));
        assert_eq!(extract_identifier("hello!foo"), ("foo", Some(6)));
        assert_eq!(extract_identifier("##@!"), ("", Some(4)));
    }
}
