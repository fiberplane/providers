use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::client::{request_state, resource_groups_tagging::Client, CanonicalRequest};

use super::paginate::Paginate;

/// The version of the API matching the module.
/// The version must be included in the canonical request of all requests
pub const VERSION: &str = "2017-01-26";
/// The prefix to use for the x-amz-target header value in POST requests
pub const X_AMZ_TARGET_PREFIX: &str = "ResourceGroupsTaggingAPI_20170126";
/// The value of the content-type header value to set in POST requests
pub const POST_CONTENT_TYPE: &str = "application/x-amz-json-1.1";

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_GetTagKeys.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTagKeysResponse {
    /// Token to use for the pagination mechanism
    pub pagination_token: String,
    /// List of tag keys
    pub tag_keys: Vec<String>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_GetTagKeys.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTagKeysRequest {
    /// Token to use for the pagination mechanism
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_token: Option<String>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_GetTagValues.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTagValuesResponse {
    /// Token to use for the pagination mechanism
    pub pagination_token: String,
    /// List of tag values associated with the request key
    pub tag_values: Vec<String>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_GetTagValues.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTagValuesRequest {
    /// Key to query tag values for
    pub key: String,
    /// Token to use for the pagination mechanism
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_token: Option<String>,
}

/// Rate limits: https://docs.aws.amazon.com/tag-editor/latest/userguide/reference.html#taged-reference-quotas
/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_GetResources.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetResourcesRequest {
    /// Exclude the resources that are compliant with the tag policy.
    ///
    /// It is only valid to include if the request also set [include_compliance_details]().
    /// The default value of nothing is equivalent to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_compliant_resources: Option<bool>,
    /// Include the tag policy compliance details of all resources
    ///
    /// The default value of nothing is equivalent to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_compliance_details: Option<bool>,
    /// Token to use for the pagination mechanism
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_token: Option<String>,
    /// Specifies a list of ARNs of resources for which you want to retrieve tag data.
    ///
    /// You can't specify both this parameter and the ResourceTypeFilters parameter
    /// in the same request. If you do, you get an Invalid Parameter exception.
    ///
    /// You can't specify both this parameter and the TagFilters parameter in the
    /// same request. If you do, you get an Invalid Parameter exception.
    ///
    /// You can't specify both this parameter and any of the pagination parameters
    /// (ResourcesPerPage, TagsPerPage, PaginationToken) in the same request. If you
    /// do, you get an Invalid Parameter exception.
    ///
    /// If a resource specified by this parameter doesn't exist, it doesn't generate
    /// an error; it simply isn't included in the response.
    ///
    /// An ARN (Amazon Resource Name) uniquely identifies a resource. For more
    /// information, see Amazon Resource Names (ARNs) and AWS Service Namespaces in
    /// the AWS General Reference.
    ///
    /// Array Members: Minimum number of 1 item. Maximum number of 100 items.
    /// Length Constraints: Minimum length of 1. Maximum length of 1011.
    #[serde(rename = "ResourceARNList")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_arn_list: Option<Vec<String>>,
    /// Specifies the maximum number of results to be returned in each page. A
    /// query can return fewer than this maximum, even if there are more results
    /// still to return. You should always check the PaginationToken response
    /// value to see if there are more results. You can specify a minimum of 1
    /// and a maximum value of 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources_per_page: Option<u32>,
    /// Specifies the resource types that you want included in the response. The
    /// format of each resource type is service[:resourceType]. For example,
    /// specifying a resource type of `ec2` returns all Amazon EC2 resources
    /// (which includes EC2 instances). Specifying a resource type of
    /// `ec2:instance` returns only EC2 instances.
    ///
    /// You can't specify both this parameter and the ResourceArnList parameter in
    /// the same request. If you do, you get an Invalid Parameter exception.
    ///
    /// The string for each service name and resource type is the same as that
    /// embedded in a resource's Amazon Resource Name (ARN).
    ///
    /// Note
    /// > For the list of services whose resources you can tag using the Resource
    /// > Groups Tagging API, see Services that support the Resource Groups Tagging
    /// > API. If an AWS service isn't listed on that page, you might still be able to
    /// > tag that service's resources by using that service's native tagging
    /// > operations instead of using Resource Groups Tagging API operations. All
    /// > tagged resources, whether the tagging used the Resource Groups Tagging API
    /// > or not, are returned by the Get* operation.
    ///
    /// You can specify multiple resource types by using an array. The array can
    /// include up to 100 items. Note that the length constraint requirement applies
    /// to each resource type filter. For example, the following string would limit
    /// the response to only Amazon EC2 instances, Amazon S3 buckets, or any AWS
    /// Audit Manager resource:
    ///
    /// `ec2:instance,s3:bucket,auditmanager`
    ///
    /// Length Constraints: Minimum length of 0. Maximum length of 256.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type_filters: Option<Vec<String>>,
    /// Specifies a list of TagFilters (keys and values) to restrict the output
    /// to only those resources that have tags with the specified keys and, if
    /// included, the specified values. Each TagFilter must contain a key with
    /// values optional. A request can include up to 50 keys, and each key can
    /// include up to 20 values.
    ///
    /// You can't specify both this parameter and the ResourceArnList parameter
    /// in the same request. If you do, you get an Invalid Parameter exception.
    ///
    /// Note the following when deciding how to use TagFilters:
    /// - If you don't specify a TagFilter, the response includes all resources
    ///   that are currently tagged or ever had a tag. Resources that currently
    ///   don't have tags are shown with an empty tag set, like this: "Tags": [].
    /// - If you specify more than one filter in a single request, the response
    ///   returns only those resources that satisfy all filters.
    /// - If you specify a filter that contains more than one value for a key,
    ///   the response returns resources that match any of the specified values
    ///   for that key.
    /// - If you don't specify a value for a key, the response returns all
    ///   resources that are tagged with that key, with any or no value.
    ///
    /// For example, for the following filters:
    /// ```
    /// filter1= {keyA,{value1}},
    /// filter2= {keyB,{value2,value3,value4}},
    /// filter3= {keyC}
    /// ```
    /// - `GetResources({filter1})` returns resources tagged with `key1=value1`
    /// - `GetResources({filter2})` returns resources tagged with `key2=value2` or
    ///   `key2=value3` or `key2=value4`
    /// - `GetResources({filter3})` returns resources tagged with any tag with the
    ///   key `key3,` and with any or no value
    /// - `GetResources({filter1,filter2,filter3})` returns resources tagged with
    ///   `(key1=value1)` and `(key2=value2 or key2=value3 or key2=value4)` and
    ///   `(key3, any or no value)`
    ///
    /// Array Members: Minimum number of 0 items. Maximum number of 50 items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_filters: Option<Vec<TagFilter>>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_TagFilter.html
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagFilter {
    /// One part of a key-value pair that makes up a tag. A key is a general
    /// label that acts like a category for more specific tag values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// One part of a key-value pair that make up a tag. A value acts as a
    /// descriptor within a tag category (key). The value can be empty or null.
    ///
    /// Array Members: Minimum number of 0 items. Maximum number of 256 items.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_GetResources.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetResourcesResponse {
    /// Token to use for the pagination mechanism
    pub pagination_token: String,
    pub resource_tag_mapping_list: Vec<ResourceTagMapping>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_ResourceTagMapping.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceTagMapping {
    /// Information that shows whether a resource is compliant with the
    /// effective tag policy, including details on any noncompliant tag keys.
    pub compliance_details: Option<ComplianceDetails>,
    #[serde(rename = "ResourceARN")]
    pub resource_arn: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_ComplianceDetails.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComplianceDetails {
    #[serde(rename = "ComplianceStatus")]
    pub is_compliant: Option<bool>,
    pub keys_with_noncompliant_values: Option<Vec<String>>,
    pub noncompliant_keys: Option<Vec<String>>,
}

/// https://docs.aws.amazon.com/resourcegroupstagging/latest/APIReference/API_Tag.html
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl From<GetResourcesRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: GetResourcesRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&req).expect("GetResourcesRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                Client::format_action_header("GetResources"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}

impl Paginate for GetResourcesRequest {
    fn next_page(self, pagination_token: Option<String>) -> Option<Self> {
        pagination_token.and_then(|token| {
            if token.is_empty() {
                return None;
            }
            Some(Self {
                pagination_token: Some(token),
                ..self
            })
        })
    }
}

impl From<GetTagKeysRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: GetTagKeysRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&req).expect("GetTagKeysRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                Client::format_action_header("GetTagKeys"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}

impl Paginate for GetTagKeysRequest {
    fn next_page(self, pagination_token: Option<String>) -> Option<Self> {
        pagination_token.and_then(|token| {
            if token.is_empty() {
                return None;
            }
            Some(Self {
                pagination_token: Some(token),
            })
        })
    }
}

impl From<GetTagValuesRequest> for CanonicalRequest<{ request_state::STEM }> {
    fn from(req: GetTagValuesRequest) -> Self {
        let method = http::Method::POST;
        let uri = "/".to_string();
        let query_params = BTreeMap::new();
        let body = serde_json::to_vec(&req).expect("GetTagValuesRequest is always serializable.");
        let headers = BTreeMap::from([
            (
                "x-amz-target".to_string(),
                Client::format_action_header("GetTagValues"),
            ),
            ("content-type".to_string(), POST_CONTENT_TYPE.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ]);

        CanonicalRequest::new(method, uri, query_params, headers, Some(body.into()))
    }
}

impl Paginate for GetTagValuesRequest {
    fn next_page(self, pagination_token: Option<String>) -> Option<Self> {
        pagination_token.and_then(|token| {
            if token.is_empty() {
                return None;
            }
            Some(Self {
                pagination_token: Some(token),
                ..self
            })
        })
    }
}
