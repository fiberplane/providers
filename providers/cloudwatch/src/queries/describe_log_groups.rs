//! Describe Log Groups query handling
use crate::{api::cloudwatch_logs::LogGroup, client::cloudwatch_logs::Client, config::Config};
use fiberplane_provider_bindings::{Blob, Cell, Error, ProviderRequest, TextCell};

use super::serialize_cells;

pub async fn invoke2_handler(config: Config, _request: ProviderRequest) -> Result<Blob, Error> {
    let client = Client::from(&config);

    client
        .list_log_groups(None, None)
        .await
        .map_err(|e| Error::Invocation {
            message: format!("failed to list metrics: {e}"),
        })
        .and_then(try_into_blob)
}

fn try_into_blob(groups: Vec<LogGroup>) -> Result<Blob, Error> {
    serialize_cells(
        groups
            .into_iter()
            .enumerate()
            .map(|(id, group)| {
                Cell::Text(TextCell {
                    id: format!("log-group-{id}"),
                    content: format!("{group:#?}"),
                    formatting: Vec::new(),
                    read_only: Some(true),
                })
            })
            .collect(),
    )
}
