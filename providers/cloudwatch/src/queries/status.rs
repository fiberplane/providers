use crate::{client::cloudwatch::Client, config::Config};
use fiberplane_pdk::prelude::{Blob, Result};
use fiberplane_pdk::providers::STATUS_MIME_TYPE;

pub async fn check_status(config: Config) -> Result<Blob> {
    let client = Client::from(&config);
    let blob = client
        .list_metrics(None, None, None, Some(0))
        .await
        .map(|_| {
            Blob::builder()
                .mime_type(STATUS_MIME_TYPE.to_owned())
                .data("ok")
                .build()
        })?;
    Ok(blob)
}
