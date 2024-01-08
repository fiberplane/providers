use fiberplane_models::autometrics::PrometheusResponse;
use fiberplane_pdk::prelude::*;
use fp_bindgen::prelude::Serializable;
use grafana_common::{query_direct_and_proxied, Config};
use serde::{Deserialize, Serialize};

pub const CONFIG_QUERY_TYPE: &str = "x-prometheus-config";

pub const YAML_MIME_TYPE: &str = "text/yaml";

#[derive(Deserialize, QuerySchema)]
pub(crate) struct ConfigQuery;

#[derive(Clone, Deserialize, PartialEq, Serialize, Serializable)]
pub struct ConfigYaml {
    yaml: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, ProviderData, Serialize)]
#[pdk(mime_type = YAML_MIME_TYPE)]
pub struct Yaml(pub String);

pub(crate) async fn query_config(_query: ConfigQuery, config: Config) -> Result<Blob> {
    let response: PrometheusResponse<ConfigYaml> =
        query_direct_and_proxied(&config, "prometheus", "/api/v1/status/config", None).await?;

    Yaml(response.data.yaml).to_blob()
}

pub fn create_code_cell(response: Blob) -> Result<Vec<Cell>> {
    let config_yaml = Yaml::parse_blob(response)?;

    let code_cell = Cell::Code(
        CodeCell::builder()
            .id("config".to_owned())
            .content(config_yaml.0)
            .syntax("yaml")
            .build(),
    );
    Ok(vec![code_cell])
}
