use std::collections::BTreeMap;

use crate::{
    api::paginate::Paginate,
    client::{request_state, CanonicalRequest},
};

use super::*;
use serde::{Deserialize, Serialize};

/// Describe Log Groups
///
/// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_DescribeLogGroups.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeLogGroupsRequest {
    /// When includeLinkedAccounts is set to True, use this parameter to specify
    /// the list of accounts to search. You can specify as many as 20 account
    /// IDs in the array.
    ///
    /// Array Members: Minimum number of 0 items. Maximum number of 20 items.
    ///
    /// Length Constraints: Fixed length of 12.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifiers: Option<Vec<String>>,

    /// If you are using a monitoring account, set this to True to have the
    /// operation return log groups in the accounts listed in
    /// accountIdentifiers.
    ///
    /// If this parameter is set to true and accountIdentifiers contains a null
    /// value, the operation returns all log groups in the monitoring account and
    /// all log groups in all source accounts that are linked to the monitoring
    /// account.
    ///
    /// Note
    /// > If you specify includeLinkedAccounts in your request, then
    /// > metricFilterCount, retentionInDays, and storedBytes are not included in the
    /// > response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_linked_accounts: Option<bool>,

    /// The maximum number of items returned. If you don't specify a value, the
    /// default is up to 50 items.
    ///
    /// Valid Range: Minimum value of 1. Maximum value of 50.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// The pattern to match.
    ///
    /// If you specify a string for this parameter, the operation returns only
    /// log groups that have names that match the string based on a
    /// case-sensitive substring search. For example, if you specify Foo, log
    /// groups named FooBar, aws/Foo, and GroupFoo would match, but foo, F/o/o
    /// and Froo would not match.
    ///
    /// Note
    ///
    /// > logGroupNamePattern and logGroupNamePrefix are mutually exclusive. Only
    /// > one of these parameters can be passed.
    ///
    /// Length Constraints: Minimum length of 0. Maximum length of 512.
    ///
    /// Pattern: [\.\-_/#A-Za-z0-9]*
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_name_pattern: Option<String>,

    /// The prefix to match.
    ///
    /// Note
    /// > logGroupNamePrefix and logGroupNamePattern are mutually exclusive. Only
    /// > one of these parameters can be passed.
    ///
    /// Length Constraints: Minimum length of 1. Maximum length of 512.
    ///
    /// Pattern: [\.\-_/#A-Za-z0-9]+
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_group_name_prefix: Option<String>,

    /// The token for the next set of items to return. (You received this token
    /// from a previous call.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Describe Log Groups Response
///
/// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_DescribeLogGroups.html#API_DescribeLogGroups_ResponseElements
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeLogGroupsResponse {
    /// The log groups.
    ///
    /// If the retentionInDays value is not included for a log group, then that log
    /// group's events do not expire.
    pub log_groups: Vec<LogGroup>,

    /// The token for the next set of items to return.
    pub next_token: Option<String>,
}

impl From<DescribeLogGroupsRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: DescribeLogGroupsRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body =
            serde_json::to_vec(&req).expect("DescribeLogGroupsRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                format!("{}.{}", X_AMZ_TARGET_PREFIX, "DescribeLogGroups"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}

impl Paginate for DescribeLogGroupsRequest {
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
