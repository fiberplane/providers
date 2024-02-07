use fiberplane_models::autometrics::{AutometricsFunction, PrometheusResponse};
use fiberplane_pdk::prelude::*;
use grafana_common::{query_direct_and_proxied, Config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const ALL_FUNCTIONS_QUERY_TYPE: &str = "x-autometrics-functions";

pub const AUTOMETRICS_FUNCTIONS_MIME_TYPE: &str = "application/vnd.autometrics.functions";

#[derive(Deserialize, QuerySchema)]
pub(crate) struct FunctionsQuery;

#[derive(Clone, Debug, Deserialize, PartialEq, ProviderData, Serialize)]
#[pdk(mime_type = AUTOMETRICS_FUNCTIONS_MIME_TYPE)]
pub struct FunctionsVector(pub Vec<AutometricsFunction>);

pub(crate) async fn query_all_functions(_query: FunctionsQuery, config: Config) -> Result<Blob> {
    let body = Blob::from({
        let mut form_data = form_urlencoded::Serializer::new(String::new());
        form_data.append_pair(
            "match[]",
            r#"{__name__=~"function_calls(_count)?(_total)?", function!="", module!=""}"#,
        );
        form_data
    });

    let response: PrometheusResponse<Vec<AutometricsFunction>> =
        query_direct_and_proxied(&config, "prometheus", "/api/v1/series", Some(body)).await?;

    FunctionsVector(filter_unique_functions(response.data)).to_blob()
}

fn filter_unique_functions(functions: Vec<AutometricsFunction>) -> Vec<AutometricsFunction> {
    let unique_functions: BTreeSet<_> = functions.into_iter().collect();
    unique_functions.into_iter().collect()
}
