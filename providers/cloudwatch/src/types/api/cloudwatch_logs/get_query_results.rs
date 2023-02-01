use std::collections::BTreeMap;

use crate::client::{request_state, CanonicalRequest};

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsRequest {
    pub query_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsResponse {
    pub results: Vec<Vec<ResultField>>,
    pub statistics: QueryStatistics,
    pub status: QueryStatus,
}

impl From<GetQueryResultsRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: GetQueryResultsRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body =
            serde_json::to_vec(&req).expect("GetQueryResultsRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                format!("{}.{}", X_AMZ_TARGET_PREFIX, "GetQueryResults"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}
