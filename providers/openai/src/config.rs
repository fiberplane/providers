use fiberplane_pdk::prelude::log;
use serde::Deserialize;
use std::collections::BTreeMap;
// use url::Url;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// If it is absent, allow the provider to hit any address
    /// on Internet.
    #[serde(flatten)]
    pub api: Option<Api>,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// The description of an API to hit.
pub struct Api {
    pub organization_id: String,
    /// Authentication credentials
    #[serde(flatten)]
    pub auth: Option<Auth>,
}

impl Api {
    /// Convert the configuration to associated headers
    pub fn to_headers(&self) -> Option<BTreeMap<String, String>> {
        log(format!("Converting config to headers {:?}", &self));
        match &self.auth {
            Some(Auth::Bearer { token }) => Some(BTreeMap::from([(
                "Authorization".to_string(),
                format!("Bearer {token}"),
            )])),
            None => Some(BTreeMap::from([(
                "Authorization".to_string(),
                // TODO - fix this, handle error
                format!("Bearer meh"),
            )])),
        }
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Auth {
    // INVESTIGATE - if field is named "apiKey" in schema, deserialization into `api_key` does not work in `to_headers`
    Bearer { token: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO - test that empty config is NOT allowed
    #[test]
    fn empty_config_is_allowed() {
        let empty = r#"{
      }"#;
        let _: Config = serde_json::from_str(empty).unwrap();
    }

    #[test]
    fn auth_token_config_deserialization() {
        let with_token = r#"{
        "baseUrl": "http://localhost:3100",
        "healthCheckPath": "/",
        "apiKey": "mytoken"
      }"#;
        let config: Config = serde_json::from_str(with_token).unwrap();
        assert!(
            matches!(config.api.and_then(|api| api.auth), Some(Auth::Bearer { api_key }) if api_key == "mytoken")
        );
    }

    #[test]
    fn no_auth_config_deserialization() {
        let without_auth = r#"{
        "baseUrl": "http://localhost:3100",
        "healthCheckPath": "/version"
      }"#;
        let config: Config = serde_json::from_str(without_auth).unwrap();
        assert!(config.api.unwrap().auth.is_none());
    }
}
