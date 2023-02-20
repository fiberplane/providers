use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryConfig {
    pub token: String,

    pub organization_slug: String,

    pub project_slug: String,
}
