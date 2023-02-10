//! Describe Queries query handling
use super::serialize_cells;
use crate::{
    api::cloudwatch_logs::QueryInfo, client::cloudwatch_logs::Client, config::Config,
    constants::LOG_GROUP_PARAM_NAME,
};
use fiberplane_pdk::prelude::{Blob, Cell, Error, ProviderRequest, TextCell};
use fiberplane_pdk::providers::FORM_ENCODED_MIME_TYPE;

pub async fn invoke2_handler(config: Config, request: ProviderRequest) -> Result<Blob, Error> {
    let request: DescribeQueriesInput = request.query_data.try_into()?;
    let client = Client::from(&config);

    client
        .describe_queries(request.log_group_name, None, None)
        .await
        .and_then(try_into_blob)
}

fn try_into_blob(groups: Vec<QueryInfo>) -> Result<Blob, Error> {
    serialize_cells(
        groups
            .into_iter()
            .enumerate()
            .map(|(id, query)| {
                Cell::Text(
                    TextCell::builder()
                        .id(format!("query-{id}"))
                        .content(format!("{query:#?}"))
                        .formatting(Vec::new())
                        .read_only(true)
                        .build(),
                )
            })
            .collect(),
    )
}

struct DescribeQueriesInput {
    log_group_name: Option<String>,
}

impl TryFrom<Blob> for DescribeQueriesInput {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type != FORM_ENCODED_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        let mut log_group_name = None;
        for (key, value) in form_urlencoded::parse(&blob.data) {
            if let LOG_GROUP_PARAM_NAME = key.as_ref() {
                if !value.is_empty() {
                    log_group_name = Some(value.to_string());
                }
            }
        }

        Ok(Self { log_group_name })
    }
}
