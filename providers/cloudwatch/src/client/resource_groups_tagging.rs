//! Resource Groups Tagging API client
//!
//! To use Resource Groups Tagging API operations, you must add the following permissions to your IAM policy:
//! - tag:GetResources
//! - tag:TagResources (unused by this fiberplane provider)
//! - tag:UntagResources (unused by this fiberplane provider)
//! - tag:GetTagKeys
//! - tag:GetTagValues
//! You'll also need permissions to access the resources of individual services so that you can tag and untag those resources.

use super::ClientCommon;
use crate::{
    api::{paginate::paginate_vec, SdkResponse},
    config::Config,
    types::api::resource_groups_tagging::*,
};
use fiberplane_pdk::providers::Error;

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) common: ClientCommon,
}

impl Client {
    fn from_config(conf: &Config) -> Self {
        Self {
            common: ClientCommon {
                service: "tagging".to_string(),
                host: format!("tagging.{}.amazonaws.com", conf.region),
                endpoint: format!("https://tagging.{}.amazonaws.com", conf.region),
                region: conf.region.clone(),
                access_key_id: conf.access_key_id.clone(),
                secret_access_key: conf.secret_access_key.clone(),
            },
        }
    }

    pub fn format_action_header(action: &str) -> String {
        format!("{X_AMZ_TARGET_PREFIX}.{action}")
    }

    /// List all tag keys, up to an optional limit
    pub async fn get_tag_keys(&self, limit: Option<usize>) -> Result<Vec<String>, Error> {
        let init_request = GetTagKeysRequest {
            pagination_token: None,
        };
        paginate_vec(
            &self.common,
            init_request,
            |response: SdkResponse| {
                if let SdkResponse::GetTagKeys(get_tag_keys_payload) = response {
                    Some(get_tag_keys_payload.tag_keys.into_iter())
                } else {
                    None
                }
            },
            |response| {
                if let SdkResponse::GetTagKeys(get_tag_keys_payload) = response {
                    Some(get_tag_keys_payload.pagination_token.clone())
                } else {
                    None
                }
            },
            limit,
        )
        .await
    }

    /// List all tag values associated with a given key, up to an optional limit
    pub async fn get_tag_values(
        &self,
        key: String,
        limit: Option<usize>,
    ) -> Result<Vec<String>, Error> {
        let init_request = GetTagValuesRequest {
            key,
            pagination_token: None,
        };
        paginate_vec(
            &self.common,
            init_request,
            |response: SdkResponse| {
                if let SdkResponse::GetTagValues(get_tag_values_payload) = response {
                    Some(get_tag_values_payload.tag_values.into_iter())
                } else {
                    None
                }
            },
            |response| {
                if let SdkResponse::GetTagValues(get_tag_values_payload) = response {
                    Some(get_tag_values_payload.pagination_token.clone())
                } else {
                    None
                }
            },
            limit,
        )
        .await
    }

    /// List all visible resources matching the optional type filter.
    ///
    /// This is useful to build a referential of resources, which can be
    /// used later to attach metadata to ARNs or ARN substrings
    pub async fn list_all_resources(
        &self,
        resource_type_filters: Option<Vec<String>>,
    ) -> Result<Vec<ResourceTagMapping>, Error> {
        self.get_resources(None, None, None, resource_type_filters, None, None)
            .await
    }

    /// Look at [GetResourcesRequest]() documentation to see what the arguments do.
    pub async fn get_resources(
        &self,
        exclude_compliant_resources: Option<bool>,
        include_compliance_details: Option<bool>,
        resource_arn_list: Option<Vec<String>>,
        resource_type_filters: Option<Vec<String>>,
        tag_filters: Option<Vec<TagFilter>>,
        limit: Option<usize>,
    ) -> Result<Vec<ResourceTagMapping>, Error> {
        let init_request = GetResourcesRequest {
            exclude_compliant_resources,
            include_compliance_details,
            pagination_token: None,
            resource_arn_list,
            resources_per_page: Some(100),
            resource_type_filters,
            tag_filters,
        };
        paginate_vec(
            &self.common,
            init_request,
            |response: SdkResponse| {
                if let SdkResponse::GetResources(get_resources_response_payload) = response {
                    Some(
                        get_resources_response_payload
                            .resource_tag_mapping_list
                            .into_iter(),
                    )
                } else {
                    None
                }
            },
            |response: &SdkResponse| {
                if let SdkResponse::GetResources(get_resources_response_payload) = response {
                    Some(get_resources_response_payload.pagination_token.clone())
                } else {
                    None
                }
            },
            limit,
        )
        .await
    }
}

impl From<&Config> for Client {
    fn from(conf: &Config) -> Self {
        Self::from_config(conf)
    }
}
