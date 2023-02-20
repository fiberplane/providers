use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub url: Url,
    #[serde(flatten)]
    pub auth: Option<Auth>,
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Auth {
    Basic { username: String, password: String },
    Bearer { token: String },
}
impl Config {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let with_token = r#"{
        "url": "http://localhost:3100",
        "token": "mytoken"
      }"#;
        let config: Config = serde_json::from_str(with_token).unwrap();
        assert!(matches!(config.auth, Some(Auth::Bearer { token }) if token == "mytoken"));

        let with_username_password = r#"{
        "url": "http://localhost:3100",
        "username": "myusername",
        "password": "mypassword"
      }"#;
        let config: Config = serde_json::from_str(with_username_password).unwrap();
        assert!(
            matches!(config.auth, Some(Auth::Basic { username, password }) if username == "myusername" && password == "mypassword")
        );

        let without_auth = r#"{
        "url": "http://localhost:3100"
      }"#;
        let config: Config = serde_json::from_str(without_auth).unwrap();
        assert!(config.auth.is_none());
    }
}
