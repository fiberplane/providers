use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryEvent {
    pub id: String,
    #[serde(default)]
    pub entries: Vec<SentryEventEntry>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SentryEventEntry {
    Breadcrumbs {},
    Exception { data: SentryExceptionData },
    Message {},
    Request {},
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryException {
    #[serde(default)]
    pub stacktrace: Option<SentryStacktrace>,
    pub value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryExceptionData {
    pub values: Vec<SentryException>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryIssue {
    #[serde(default)]
    pub first_seen: String,
    pub has_seen: bool,
    pub id: String,
    pub last_seen: String,
    pub level: String,
    pub num_comments: u32,
    pub permalink: String,
    pub platform: String,
    pub project: SentryProjectSummary,
    pub short_id: String,
    pub status: String,
    #[serde(default)]
    pub tags: Vec<SentryTag>,
    pub title: String,
    pub user_count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryProjectSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryStacktrace {
    pub frames: Vec<SentryStackframe>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryStackframe {
    #[serde(default)]
    pub col_no: Option<u32>,
    #[serde(default)]
    pub context: Vec<SentryStackframeContext>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub function: Option<String>,
    #[serde(default)]
    pub line_no: Option<u32>,
    #[serde(default)]
    pub module: Option<String>,
}

/// Tuple of line number and line content.
#[derive(Deserialize)]
pub struct SentryStackframeContext(u32, String);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryTag {
    pub key: String,
    pub name: String,
    pub total_values: u32,
    #[serde(default)]
    pub top_values: Vec<SentryTagValue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentryTagValue {
    pub key: String,
    pub name: String,
    pub value: String,
    pub count: u32,
    pub last_seen: String,
    pub first_seen: String,
}
