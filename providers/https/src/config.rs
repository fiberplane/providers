use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// If it is absent, allow the provider to hit any address
    /// on Internet.
    #[serde(flatten)]
    pub api: Option<Api>,
    /// Show response headers in the query results
    /// Defaults to false
    #[serde(default)]
    pub show_headers: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// The description of an API to hit.
pub struct Api {
    /// The base url of the API we are interested in
    pub base_url: Url,
    /// The path to the healthcheck/status endpoint
    pub health_check_path: Option<String>,
    /// Authentication credentials
    #[serde(flatten)]
    pub auth: Option<Auth>,
}

impl Api {
    /// Convert the configuration to associated headers
    pub fn to_headers(&self) -> Option<HashMap<String, String>> {
        match &self.auth {
            Some(Auth::Basic { username, password }) => Some(HashMap::from([(
                "Authorization".to_string(),
                format!("Basic {}", base64::encode(format!("{username}:{password}"))),
            )])),
            Some(Auth::Bearer { token }) => Some(HashMap::from([(
                "Authorization".to_string(),
                format!("Bearer {token}"),
            )])),
            None => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Auth {
    Basic { username: String, password: String },
    Bearer { token: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_allowed() {
        let empty = r#"{
      }"#;
        let _: Config = serde_json::from_str(empty).unwrap();
    }

    #[test]
    fn headers_only_config_is_allowed() {
        let allow_headers = r#"{
       "showHeaders": true
      }"#;
        let config: Config = serde_json::from_str(allow_headers).unwrap();
        assert!(config.show_headers);
    }

    #[test]
    fn auth_token_config_deserialization() {
        let with_token = r#"{
        "baseUrl": "http://localhost:3100",
        "healthCheckPath": "/",
        "token": "mytoken"
      }"#;
        let config: Config = serde_json::from_str(with_token).unwrap();
        assert!(
            matches!(config.api.and_then(|api| api.auth), Some(Auth::Bearer { token }) if token == "mytoken")
        );
    }

    #[test]
    fn auth_user_password_config_deserialization() {
        let with_username_password = r#"{
        "baseUrl": "http://localhost:3100",
        "healthCheckPath": "/health",
        "username": "myusername",
        "password": "mypassword"
      }"#;
        let config: Config = serde_json::from_str(with_username_password).unwrap();
        assert!(
            matches!(config.api.and_then(|api| api.auth), Some(Auth::Basic { username, password }) if username == "myusername" && password == "mypassword")
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
