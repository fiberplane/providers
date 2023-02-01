use super::{POST_CONTENT_TYPE, X_AMZ_TARGET_PREFIX};
use crate::client::{request_state, CanonicalRequest};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLogRecordRequest {
    pub log_record_pointer: String,
    /// To use this operation with this parameter, you must be signed into an
    /// account with the logs:Unmask permission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmask: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLogRecordResponse {
    pub log_record: HashMap<String, String>,
}

impl From<GetLogRecordRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(get_log_record_request: GetLogRecordRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&get_log_record_request)
            .expect("GetLogRecordRequest is always serializable.");
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
