use crate::{client::cloudwatch::Client, config::Config};
use fiberplane_models::providers::STATUS_MIME_TYPE;
use fiberplane_provider_bindings::{Blob, Error};

pub async fn check_status(config: Config) -> Result<Blob, Error> {
    let client = Client::from(&config);
    let blob = client
        .list_metrics(None, None, None, Some(0))
        .await
        .map(|_| Blob {
            mime_type: STATUS_MIME_TYPE.to_owned(),
            data: "ok".into(),
        })?;
    Ok(blob)
}
