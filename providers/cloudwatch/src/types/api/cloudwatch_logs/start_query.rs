use super::*;
use crate::client::{request_state, CanonicalRequest};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_StartQuery.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartQueryRequest {
    pub end_time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    // TODO: Use serde(flatten) on an enum field for mutually exclusive arguments
    // the enum variant being externally tagged with rename="camelCase" should be enough??
    /// The list of log groups to query. You can include up to 50 log groups.
    ///
    /// You can specify them by the log group name or ARN. If a log group that
    /// you're querying is in a source account and you're using a monitoring
    /// account, you must specify the ARN of the log group here. The query
    /// definition must also be defined in the monitoring account.
    ///
    /// If you specify an ARN, the ARN can't end with an asterisk (*).
    ///
    /// A StartQuery operation must include exactly one of the following parameters:
    /// logGroupName, logGroupNames or logGroupIdentifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_identifiers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_names: Option<Vec<String>>,
    pub query_string: String,
    pub start_time: Timestamp,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartQueryResponse {
    pub query_id: String,
}

impl From<StartQueryRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(start_query_request: StartQueryRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&start_query_request)
            .expect("StartQueryRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                format!("{}.{}", X_AMZ_TARGET_PREFIX, "StartQuery"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}
