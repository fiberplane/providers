use super::*;
use crate::{
    api::paginate::Paginate,
    client::{request_state, CanonicalRequest},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_DescribeQueries.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeQueriesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<QueryStatus>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeQueriesResponse {
    pub next_token: Option<String>,
    pub queries: Vec<QueryInfo>,
}

impl From<DescribeQueriesRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(describe_queries_request: DescribeQueriesRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&describe_queries_request)
            .expect("DescribeQueriesRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                format!("{}.{}", X_AMZ_TARGET_PREFIX, "DescribeQueries"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}

impl Paginate for DescribeQueriesRequest {
    fn next_page(self, pagination_token: Option<String>) -> Option<Self> {
        pagination_token.and_then(|token| {
            if token.is_empty() {
                return None;
            }
            Some(Self {
                next_token: Some(token),
                ..self
            })
        })
    }
}
