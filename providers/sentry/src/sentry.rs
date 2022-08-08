use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryIssue {
    #[serde(default)]
    pub first_seen: String,
    pub has_seen: bool,
    pub id: String,
    pub last_seen: String,
    pub level: String,
    pub num_comments: usize,
    pub permalink: String,
    pub platform: String,
    pub project: SentryProjectSummary,
    pub short_id: String,
    pub status: String,
    pub title: String,
    pub user_count: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryProjectSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
}
