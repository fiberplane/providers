use const_format::formatcp;
use fiberplane_pdk::prelude::*;

/// Query type for the provider's showcase.
///
/// Note that custom query types should be prefixed with `x-` to avoid collision
/// with built-in query types.
pub const SHOWCASE_QUERY_TYPE: &str = "x-showcase";

pub const SHOWCASE_MSGPACK_MIME_TYPE: &str =
    "application/vnd.fiberplane.providers.sample.showcase+msgpack";

// Note how we need to specify an encoding in addition to the base MIME types.
// Fiberplane Studio can decode responses using either `+msgpack` or `+json`
// encodings.
pub const CELLS_MSGPACK_MIME_TYPE: &str = formatcp!("{CELLS_MIME_TYPE}+msgpack");
pub const SUGGESTIONS_MSGPACK_MIME_TYPE: &str = formatcp!("{SUGGESTIONS_MIME_TYPE}+msgpack");

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

/// This example shows how to define a struct and let the PDK generate a config
/// schema for it. This schema is used by Fiberplane Studio to render the
/// config form. The data will be encoded using JSON when the provider is
/// configured inside Studio, though it may also be encoded using YAML if the
/// provider is configured inside a Proxy. In either case, the data is stored as
/// an untyped object in the `config` field of the `ProviderRequest` that is
/// passed to `invoke2()`. The generated `parse()` method can then be used to
/// parse this object into the following struct.
#[derive(ConfigSchema)]
struct SampleConfig {
    #[pdk(label = "Specify your endpoint")]
    pub endpoint: String,
}

/// This example shows how to define a struct and let the PDK generate a query
/// schema for it. This schema is used by Fiberplane Studio to render a suitable
/// query form. The data will be encoded using `FORM_ENCODED_MIME_TYPE` and
/// stored in the `query_data` field of the `ProviderRequest` that is passed to
/// `invoke2()`. The generated `parse()` method can then be used to parse the
/// form encoded data into the following struct.
#[derive(QuerySchema)]
struct ShowcaseQueryData {
    #[pdk(label = "Enter your Prometheus query")]
    pub query: String,

    #[pdk(label = "Specify a time range")]
    pub time_range: DateTimeRange,

    #[pdk(label = "Enable live mode")]
    pub live: bool,
}

#[pdk_export]
async fn get_supported_query_types(_config: ProviderConfig) -> Vec<SupportedQueryType> {
    vec![
        SupportedQueryType::new(SHOWCASE_QUERY_TYPE)
            .with_schema(ShowcaseQueryData::schema())
            .supporting_mime_types(&[CELLS_MSGPACK_MIME_TYPE, SHOWCASE_MSGPACK_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
        SupportedQueryType::new(SUGGESTIONS_QUERY_TYPE)
            .supporting_mime_types(&[SUGGESTIONS_MSGPACK_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob, Error> {
    log(format!(
        "Sample provider (commit: {COMMIT_HASH}, built at: {BUILD_TIMESTAMP}) \
        invoked for query type \"{}\" and query data \"{:?}\"",
        request.query_type, request.query_data
    ));

    let config = SampleConfig::parse(request.config).map_err(|err| Error::Config {
        message: format!("Error parsing config: {:?}", err),
    })?;

    match request.query_type.as_str() {
        TIMESERIES_QUERY_TYPE => query_series(request.query_data, config).await,
        SUGGESTIONS_QUERY_TYPE => query_suggestions(request.query_data, config).await,
        STATUS_QUERY_TYPE => check_status().await,
        _ => Err(Error::UnsupportedRequest),
    }
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>, Error> {
    log(format!("Creating cells for query type: {query_type}"));

    match query_type.as_str() {
        INSTANTS_QUERY_TYPE => todo!("Instants support is not currently implemented"),
        TIMESERIES_QUERY_TYPE => create_graph_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
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
/// be available.
async fn check_status() -> Result<Blob, Error> {
    Ok(Blob {
        mime_type: STATUS_MIME_TYPE.to_owned(),
        data: "ok".into(),
    })
}
