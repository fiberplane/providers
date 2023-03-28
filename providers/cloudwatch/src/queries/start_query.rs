//! "Start Log query" query handling
use super::{serialize_cells, try_from_iso_date};
use crate::{
    client::cloudwatch_logs::Client,
    config::Config,
    constants::{
        GET_QUERY_RESULTS_QUERY_TYPE, LOG_GROUP_PARAM_NAME, PROVIDER_TYPE, QUERY_ID_PARAM_NAME,
        QUERY_PARAM_NAME, TIME_RANGE_PARAM_NAME,
    },
};
use fiberplane_pdk::prelude::{
    now, Annotation, AnnotationWithOffset, Blob, Cell, Error, ProviderRequest, TextCell, Timestamp,
};
use fiberplane_pdk::providers::FORM_ENCODED_MIME_TYPE;

pub async fn invoke2_handler(config: Config, request: ProviderRequest) -> Result<Blob, Error> {
    let request: StartQueryInput = request.query_data.try_into()?;
    let client = Client::from(&config);

    client
        .start_query(
            request.query,
            request.start_time,
            request.end_time,
            request.log_group_names,
            request.limit,
        )
        .await
        .and_then(try_into_blob)
}

fn try_into_blob(id: String) -> Result<Blob, Error> {
    let url = format!(
        "provider:{},{}?{}",
        PROVIDER_TYPE,
        GET_QUERY_RESULTS_QUERY_TYPE,
        form_urlencoded::Serializer::new(String::new())
            .append_pair(QUERY_ID_PARAM_NAME, &id)
            .finish()
    );
    let content = format!("Query Id: {id} (Click here to see results in a new cell)");
    let link_start: u32 = (content.len() - "(Click here to see results in a new cell)".len())
        .try_into()
        .expect("usize fits in u32 on all target architectures.");
    let length: u32 = content
        .len()
        .try_into()
        .expect("usize fits in u32 on all target architectures.");

    serialize_cells(vec![Cell::Text(
        TextCell::builder()
            .id("query-id".to_string())
            .formatting(vec![
                AnnotationWithOffset::new(link_start, Annotation::StartLink { url }),
                AnnotationWithOffset::new(length, Annotation::EndLink),
            ])
            .content(content)
            .read_only(true)
            .build(),
    )])
}

struct StartQueryInput {
    query: String,
    start_time: Timestamp,
    end_time: Timestamp,
    log_group_names: Vec<String>,
    limit: Option<usize>,
}

impl TryFrom<Blob> for StartQueryInput {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type != FORM_ENCODED_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        let mut query = String::new();
        let mut log_group_names = Vec::new();
        let mut from = now();
        let mut to = now();
        for (key, value) in form_urlencoded::parse(&blob.data) {
            match key.as_ref() {
                QUERY_PARAM_NAME => query = value.to_string(),
                LOG_GROUP_PARAM_NAME => {
                    log_group_names.extend(value.lines().map(ToString::to_string))
                }
                TIME_RANGE_PARAM_NAME => {
                    // TODO: Add validation error for non-compliant value
                    if let Some((ts_from, ts_to)) = value.split_once(' ') {
                        from = try_from_iso_date(ts_from)?;
                        to = try_from_iso_date(ts_to)?;
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            query,
            log_group_names,
            start_time: from,
            end_time: to,
            limit: None,
        })
    }
}
