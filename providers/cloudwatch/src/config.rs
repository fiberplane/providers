use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub region: String,
    // https://docs.rs/aws-types/0.51.0/aws_types/credentials/struct.Credentials.html#method.from_keys
    pub access_key_id: SecretString,
    pub secret_access_key: SecretString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_deserialization() {
        let empty = r#"{
      }"#;
        let _ = serde_json::from_str::<Config>(empty).unwrap_err();
    }
}
