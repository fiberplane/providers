use super::{constants::*, prometheus::*};
use crate::timeseries::{
    round_to_grid, step_for_range, to_float, to_iso_date, validate_or_parse_message,
    RoundToGridEdge,
};
use async_recursion::async_recursion;
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde::Deserialize;

#[derive(Deserialize, QuerySchema)]
pub(crate) struct NodeGraphQuery {
    #[pdk(label = "Enter your Prometheus query", supports_suggestions)]
    function: String,

    #[pdk(label = "Specify a time range")]
    time_range: DateTimeRange,

    #[pdk(label = "Specify the maximum depth")]
    depth: u8,
}

pub(crate) async fn node_graph(query: NodeGraphQuery, config: Config) -> Result<Blob> {
    validate_query(&query)?;

    run_query(
        &query.function,
        0,
        query.depth,
        &query.function,
        &query,
        &config,
    )
    .await
    // .into_iter()
    // .collect::<core::result::Result<Vec<_>, Error>>()
    .and_then(|mut nodes| {
        // Add the root node
        let root_node = Node::builder()
            .name(query.function.to_string())
            .id(query.function.to_string())
            .module("".to_string())
            .children(nodes.iter().map(|node| node.id.to_string()).collect())
            .build();
        nodes.push(root_node);

        Ok(Blob::builder()
            .data(rmp_serde::to_vec_named(&nodes)?)
            .mime_type(NODE_GRAPH_MIME_TYPE.to_owned())
            .build())
    })
}

#[async_recursion(?Send)]
async fn run_query(
    function_name: &str,
    depth: u8,
    max_depth: u8,
    prefix: &str,
    query: &NodeGraphQuery,
    config: &Config,
) -> Result<Vec<Node>> {
    let function_query = format!("sum by (function, module) (increase(function_calls_count{{caller=\"{function_name}\"}}[120m]))");
    let from = to_float(query.time_range.from);
    let to = to_float(query.time_range.to);
    let step = step_for_range(from, to);
    let start = to_iso_date(round_to_grid(from, step, RoundToGridEdge::Start));
    let end = to_iso_date(round_to_grid(to, step, RoundToGridEdge::End));

    let mut form_data = form_urlencoded::Serializer::new(String::new());
    form_data.append_pair("start", &start);
    form_data.append_pair("end", &end);
    form_data.append_pair("step", &step.to_string());
    form_data.append_pair("query", &function_query);

    let new_depth = depth + 1;

    if new_depth > max_depth {
        return Err(Error::Other {
            message: "Weird error".to_owned(),
        });
    }

    let query_string = form_data.finish();
    let body = Blob::builder()
        .data(query_string.into_bytes())
        .mime_type(FORM_ENCODED_MIME_TYPE.to_owned())
        .build();

    let response: PrometheusResponse =
        query_direct_and_proxied(config, "prometheus", "api/v1/query_range", Some(body))
            .await
            .map_err(|err| match err {
                Error::Other { message } => validate_or_parse_message(function_name, &message),
                err => err,
            })?;

    let PrometheusData::Matrix(matrix) = response.data;

    let result_nodes = matrix
        .into_iter()
        .map(|vector| RangeVector::into_node(vector, prefix))
        .collect::<core::result::Result<Vec<_>, Error>>();

    if new_depth >= max_depth || result_nodes.is_err() {
        return result_nodes;
    }

    let mut result = Vec::new();

    if let Ok(nodes) = result_nodes {
        for mut node in nodes {
            let children_result =
                run_query(node.name.as_str(), depth, max_depth, prefix, query, config).await;

            match children_result {
                Ok(mut children) => {
                    // let children_id = children
                    //     .into_iter()
                    //     .map(|child| child.id.to_string())
                    //     .collect();
                    let mut ids = children
                        .iter()
                        .map(|node| node.id.to_string())
                        .collect::<Vec<_>>();

                    node.children.append(&mut ids);
                    result.push(node);
                    result.append(&mut children);
                }
                Err(error) => return Err(error),
            }
        }
    }

    Ok(result)

    // for let Some(node) in nodes {
    //     let children =                 run_query(
    //         node.name.as_str(),
    //         new_depth,
    //         max_depth,
    //         prefix,
    //         query,
    //         config,
    //     )
    //     .await;
    // }
    // nodes
    //     .map(|nodes| async {
    //         // if (new_depth >= max_depth) {
    //         //     return nodes;
    //         // }

    //         let result = Vec::new();

    //         for node in nodes {}
    //         futures::stream::iter(nodes).for_each(|node| async {
    //             run_query(
    //                 node.name.as_str(),
    //                 new_depth,
    //                 max_depth,
    //                 prefix,
    //                 query,
    //                 config,
    //             )
    //             .await
    //             .and_then(|children| result.append(children));
    //             // result.append(children);
    //         });

    //         result
    //         // let children = Vec::new();
    //         // if (new_depth < max_depth) {
    //         //     nodes = run_query(node., depth, max_depth, prefix, query, config)
    //         // }
    //     })
    //     .and_then(|nodes| Ok(nodes))
}

fn validate_query(query: &NodeGraphQuery) -> Result<()> {
    let mut errors = Vec::new();
    if query.function.is_empty() {
        errors.push(
            ValidationError::builder()
                .field_name(QUERY_PARAM_NAME.to_owned())
                .message("Please enter a query".to_owned())
                .build(),
        );
    }

    match errors.is_empty() {
        true => Ok(()),
        false => Err(Error::ValidationError { errors }),
    }
}
