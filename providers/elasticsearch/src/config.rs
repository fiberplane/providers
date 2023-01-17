use fiberplane_pdk::prelude::*;
use serde::Deserialize;
use std::str::FromStr;
use url::Url;

#[derive(ConfigSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ElasticConfig {
    #[pdk(
        label = "Your Elasticsearch endpoint",
        placeholder = "Please specify a URL"
    )]
    pub url: String,

    #[serde(default)]
    #[pdk(label = "Optional field names to treat as timestamps")]
    pub timestamp_field_names: Vec<String>,

    #[serde(default)]
    #[pdk(label = "Optional field names to treat as the message body")]
    pub body_field_names: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[pdk(label = "Your API key")]
    pub api_key: Option<String>,
}

impl ElasticConfig {
    pub fn parse_url(&self) -> Result<Url, Error> {
        Url::from_str(&self.url).map_err(|err| Error::Config {
            message: format!("Invalid Elasticsearch URL: {err}"),
        })
    }
}
