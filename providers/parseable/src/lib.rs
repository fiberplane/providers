use std::collections::BTreeMap;

use base64::{prelude::BASE64_STANDARD, Engine};
use fiberplane_pdk::{prelude::*, provider_data::ProviderData};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::{
    format_description::FormatItem, macros::format_description, parsing::Parsed, OffsetDateTime,
};
use url::Url;

pub const QUERY_API: &str = "/api/v1/query";
pub const LIVENESS_API: &str = "/api/v1/liveness";

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(ConfigSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    #[pdk(label = "Parseable endpoint", placeholder = "Please specify a URL")]
    pub endpoint: String,

    #[pdk(label = "Username", placeholder = "admin")]
    pub username: String,

    #[pdk(label = "Password", placeholder = "admin")]
    pub password: String,
}

#[derive(QuerySchema, Deserialize, Serialize, Debug, Clone)]
struct Query {
    #[pdk(label = "query", placeholder = "select * from stream")]
    pub query: String,

    #[pdk(label = "Specify a time range")]
    pub time_range: DateTimeRange,
}

pdk_query_types! {
    EVENTS_QUERY_TYPE => {
        label: "Parseable: Run a query",
        handler: query_handler(Query, Config).await,
        supported_mime_types: [EVENTS_MIME_TYPE],
    },
    STATUS_QUERY_TYPE => {
        handler: check_status(ProviderRequest).await,
        supported_mime_types: [STATUS_MIME_TYPE]
    }
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>> {
    match query_type.as_str() {
        EVENTS_QUERY_TYPE => create_log_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
}

pub fn create_log_cell() -> Result<Vec<Cell>> {
    let logs_cell = Cell::Log(
        LogCell::builder()
            .id("query-results".to_string())
            .data_links(vec![format!("cell-data:{EVENTS_MIME_TYPE},self")])
            .hide_similar_values(false)
            .build(),
    );
    Ok(vec![logs_cell])
}

async fn query_handler(query: Query, config: Config) -> Result<Blob> {
    let events = run_query(&query, &config).await?;
    Events(events).to_blob()
}

async fn check_status(request: ProviderRequest) -> Result<Blob> {
    let config = Config::parse(request.config)?;
    let url = get_url(LIVENESS_API, &config)?;
    let response = make_http_request(HttpRequest::get(url)).await?;
    if response.status_code == 200 {
        ProviderStatus::builder()
            .status(Ok(()))
            .version(COMMIT_HASH.to_owned())
            .built_at(BUILD_TIMESTAMP.to_owned())
            .build()
            .to_blob()
    } else {
        Err(Error::Http {
            error: HttpRequestError::Offline,
        })
    }
}

async fn run_query(query: &Query, config: &Config) -> Result<Vec<ProviderEvent>> {
    let query = json!({
        "query": query.query,
        "startTime": query.time_range.from.to_string(),
        "endTime": query.time_range.to.to_string()
    });

    let url = get_url(QUERY_API, config)?;
    let mut headers = BTreeMap::new();
    headers.insert(
        "Authorization".to_string(),
        basic_auth_header(&config.username, &config.password),
    );
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let request = HttpRequest::post(url, query.to_string()).with_headers(headers);
    let response = make_http_request(request).await?;
    let body: Value = serde_json::from_slice(&response.body)?;

    if response.status_code == 200 {
        let Value::Array(arr) = body else {
            return Err(Error::Other {
                message: format!("Expected an array, received: {}", body),
            });
        };
        let mut rows = Vec::with_capacity(arr.len());
        for value in arr {
            rows.push(parse_row(value)?)
        }
        Ok(rows)
    } else {
        // Return error message returned by Parseable API.
        Err(Error::Other {
            message: String::from_utf8_lossy(&response.body).to_string(),
        })
    }
}

fn parse_row(value: Value) -> Result<ProviderEvent> {
    let Value::Object(mut object) = value else {
        return Err(Error::Other {
            message: format!("Expected object, found {}", value),
        });
    };
    let timestamp = object.remove("p_timestamp");
    let timestamp = timestamp.as_ref().and_then(|value| value.as_str());

    let timestamp = match timestamp {
        Some(timestamp) => parse_time(timestamp).map_err(|err| Error::Other {
            message: format!("Failed to parse timestamp: {}", err),
        })?,
        None => Timestamp::from(OffsetDateTime::UNIX_EPOCH),
    };

    let otel = OtelMetadata::builder()
        .attributes(BTreeMap::from_iter(object))
        .resource(BTreeMap::default())
        .build();

    let event = ProviderEvent::builder()
        .otel(otel)
        .time(timestamp)
        .title("".to_string())
        .build();

    Ok(event)
}

fn get_url(api: &str, config: &Config) -> Result<Url> {
    let base_url: Url = config
        .endpoint
        .parse()
        .map_err(|e: url::ParseError| Error::Config {
            message: format!("Invalid URL in configuration: {e:?}"),
        })?;
    base_url.join(api).map_err(|e| Error::Config {
        message: format!("Invalid URL in configuration: {e:?}"),
    })
}

fn basic_auth_header(username: &str, password: &str) -> String {
    format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("{}:{}", username, password))
    )
}

// Parseable's p_timestamp value is not sufficient for time parser.
// Offset is missing though it is understood as UTC. So we add it manually
fn parse_time(time: &str) -> std::result::Result<Timestamp, time::Error> {
    const TIME_FORMAT: &[FormatItem<'_>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]");

    let mut parser = Parsed::new();
    parser.parse_items(time.as_bytes(), TIME_FORMAT)?;
    parser.set_offset_hour(0);
    parser.set_offset_minute_signed(0);

    let datetime = OffsetDateTime::try_from(parser)?;
    Ok(Timestamp::from(datetime))
}
