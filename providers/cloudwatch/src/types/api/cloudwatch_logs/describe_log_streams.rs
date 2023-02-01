use std::collections::BTreeMap;

use crate::client::{request_state, CanonicalRequest};

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeLogStreamsRequest {
    /// Defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    ///  Specify either the name or ARN of the log group to view. If the log
    ///  group is in a source account and you are using a monitoring account,
    ///  you must use the log group ARN.
    ///  
    /// If you specify values for both logGroupName and logGroupIdentifier, the
    /// action returns an InvalidParameterException error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_name: Option<String>,
    /// The prefix to match.
    ///
    /// If orderBy is LastEventTime, you cannot specify this parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_stream_name_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderStreamsBy>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeLogStreamsResponse {
    pub log_streams: Vec<LogStream>,
    pub next_token: Option<String>,
}

impl From<DescribeLogStreamsRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: DescribeLogStreamsRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body =
            serde_json::to_vec(&req).expect("DescribeLogStreamsRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                format!("{}.{}", X_AMZ_TARGET_PREFIX, "GetLogRecord"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}
