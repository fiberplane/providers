use const_format::formatcp;
use fiberplane_pdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Query type for the provider's showcase.
///
/// Note that custom query types should be prefixed with `x-` to avoid collision
/// with built-in query types.
pub const CELLS_SHOWCASE_QUERY_TYPE: &str = "x-showcase-cells";
pub const CUSTOM_DATA_SHOWCASE_QUERY_TYPE: &str = "x-showcase-custom";

pub const SHOWCASE_JSON_MIME_TYPE: &str =
    "application/vnd.fiberplane.providers.sample.showcase+json";

// Note how we need to specify an encoding in addition to the base MIME type.
// Fiberplane Studio can decode responses using either `+json` or `+msgpack`
// encodings. Note that Fiberplane's own providers primarily use MessagePack for
// performance reasons, but we'll use JSON in this sample provider, because it's
// easier to inspect should you find yourself wishing to debug something.
pub const CELLS_JSON_MIME_TYPE: &str = formatcp!("{CELLS_MIME_TYPE}+json");
pub const SUGGESTIONS_JSON_MIME_TYPE: &str = formatcp!("{SUGGESTIONS_MIME_TYPE}+json");

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

/// This example shows how to define a struct and let the PDK generate a config
/// schema for it. This schema is used by Fiberplane Studio to render the
/// config form. The data is stored as an untyped object in the `config` field
/// of the `ProviderRequest` that is passed to `invoke2()`. The generated
/// `parse()` method can then be used to parse this object into the following
/// struct.
#[derive(ConfigSchema, Deserialize)]
struct SampleConfig {
    #[pdk(label = "Your API endpoint", placeholder = "Please specify a URL")]
    pub endpoint: String,

    #[pdk(label = "I accept the Terms of Use", checked_by_default)]
    pub accept: bool,

    #[pdk(label = "Number of retries if a request fails", max = 10)]
    pub num_retries: u8,
}

/// This example shows how to define a struct and let the PDK generate a query
/// schema for it. This schema is used by Fiberplane Studio to render a suitable
/// query form. The data will be encoded using `FORM_ENCODED_MIME_TYPE` and
/// stored in the `query_data` field of the `ProviderRequest` that is passed to
/// `invoke2()`. The generated `parse()` method can then be used to parse the
/// form encoded data into the following struct.
#[derive(QuerySchema, Deserialize, Serialize)]
struct ShowcaseQueryData {
    #[pdk(label = "Enter your sample query (anything will do :)")]
    pub query: String,

    #[pdk(label = "Specify a time range")]
    pub time_range: DateTimeRange,

    #[pdk(label = "Enable live mode")]
    pub live: bool,

    #[pdk(
        select,
        label = "Select one or more tags",
        option = "1.0.0",
        option = "1.0.0-alpha.1",
        option = "1.0.1"
    )]
    pub tags: Vec<String>,
}

#[pdk_export]
async fn get_supported_query_types(_config: ProviderConfig) -> Vec<SupportedQueryType> {
    vec![
        SupportedQueryType::new(CELLS_SHOWCASE_QUERY_TYPE)
            .with_label("Showcase query (cells)")
            .with_schema(ShowcaseQueryData::schema())
            .supporting_mime_types(&[CELLS_JSON_MIME_TYPE]),
        SupportedQueryType::new(CUSTOM_DATA_SHOWCASE_QUERY_TYPE)
            .with_label("Showcase query (custom data)")
            .with_schema(ShowcaseQueryData::schema())
            .supporting_mime_types(&[SHOWCASE_JSON_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
        SupportedQueryType::new(SUGGESTIONS_QUERY_TYPE)
            .supporting_mime_types(&[SUGGESTIONS_JSON_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob, Error> {
    log(format!(
        "Sample provider (commit: {COMMIT_HASH}, built at: {BUILD_TIMESTAMP}) \
        invoked for query type \"{}\" and query data \"{:?}\"",
        request.query_type, request.query_data
    ));

    let config = SampleConfig::parse(request.config)?;

    match request.query_type.as_str() {
        CELLS_SHOWCASE_QUERY_TYPE => query_cells_showcase(request.query_data, config),
        CUSTOM_DATA_SHOWCASE_QUERY_TYPE => query_custom_data_showcase(request.query_data, config),
        STATUS_QUERY_TYPE => check_status(),
        _ => Err(Error::UnsupportedRequest),
    }
}

/// Creates cells reflecting the data we entered in the original form.
///
/// In this example, `invoke2()` returned a Blob with data encoded in a custom
/// format (indicated using the `SHOWCASE_JSON_MIME_TYPE`), and then
/// `create_cells()` is invoked with that Blob to create the cells.
///
/// Note that if you only intend to create cells and no longer have any use for
/// the data from which the cells are created, this process can be simplified:
/// `invoke2()` can also return a Blob that directly contains the cells. If so,
/// you should encode the `Vec<Cell>` data using JSON or MessagePack encoding,
/// and specify the `CELLS_MIME_TYPE` with a `+json` or `+msgpack` suffix,
/// respectively. In this scenario, `create_cells()` doesn't need to be
/// implemented at all.
#[pdk_export]
fn create_cells(query_type: String, response: Blob) -> Result<Vec<Cell>, Error> {
    log(format!("Creating cells for query type: {query_type}"));

    if response.mime_type != SHOWCASE_JSON_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }

    let ShowcaseQueryData {
        query,
        time_range: DateTimeRange { from, to },
        live,
        tags,
    } = fiberplane_pdk::serde_json::from_slice(&response.data)?;

    Ok(vec![Cell::Text(TextCell {
        id: "result".to_owned(),
        content: format!(
            "Your query was: {query}\n\
            Your time range: {from} - {to}\n\
            Live mode was {live}\n
            Selected tags: {tags:?}"
        ),
        formatting: Formatting::default(),
        read_only: None,
    })])
}

/// The most basic of health-check functions: Always returns "ok".
///
/// The Fiberplane Proxy server regularly submits status checks on its providers
/// to determine whether they can still reach whatever service they are
/// connected to. This is done by submitting a request with a query type of
/// `STATUS_QUERY_TYPE`. A response of "ok" with the `STATUS_MIME_TYPE`
/// indicates the provider is still available.
///
/// If the provider is not available, an `Error` should be returned.
///
/// Note that `STATUS_QUERY_TYPE` needs to be present in the response from
/// `get_supported_query_types()`. If the query type is omitted there, it means
/// the provider doesn't support health checks, and the provider is assumed to
/// be always available.
fn check_status() -> Result<Blob, Error> {
    Ok(Blob {
        mime_type: STATUS_MIME_TYPE.to_owned(),
        data: "ok".into(),
    })
}

/// This showcase shows how to return cells directly, without the need for
/// returning custom data first. In this case, we directly encode the
/// `Vec<Cell>` data using a JSON encoding.
///
/// In many cases, this also allows you to omit implementing `create_cells()`
/// entirely. But for this provider, we still need to implement it to support
/// the custom data showcase.
fn query_cells_showcase(query_data: Blob, config: SampleConfig) -> Result<Blob, Error> {
    let response = query_custom_data_showcase(query_data, config)?;
    let cells = create_cells(CELLS_SHOWCASE_QUERY_TYPE.to_owned(), response)?;

    Ok(Blob {
        mime_type: CELLS_JSON_MIME_TYPE.to_owned(),
        data: fiberplane_pdk::serde_json::to_vec_pretty(&cells)?.into(),
    })
}

/// For this showcase, we simply re-encode the query data to JSON, so that we
/// can conveniently use it for other purposes again. In a real-world scenario,
/// this is where we could perform some HTTP request and use the response to
/// either generate a custom response, or to directly generate notebook cells
/// using the `CELLS_MIME_TYPE` format (see the cells showcase).
fn query_custom_data_showcase(query_data: Blob, _config: SampleConfig) -> Result<Blob, Error> {
    let query_data = ShowcaseQueryData::parse(query_data)?;

    Ok(Blob {
        mime_type: SHOWCASE_JSON_MIME_TYPE.to_owned(),
        data: fiberplane_pdk::serde_json::to_vec_pretty(&query_data)?.into(),
    })
}
