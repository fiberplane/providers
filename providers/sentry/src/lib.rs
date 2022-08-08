mod config;
mod sentry;

use config::SentryConfig;
use fiberplane::text_util::char_count;
use fp_provider::*;
use sentry::*;
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const OVERVIEW_QUERY_TYPE: &str = "x-issues-overview";
const ISSUE_QUERY_TYPE: &str = "x-issue-details";
const STATUS_QUERY_TYPE: &str = "status";

const OVERVIEW_MIME_TYPE: &str = "application/vnd.fiberplane.x-sentry/events+json";
const ISSUE_MIME_TYPE: &str = "application/vnd.fiberplane.x-sentry/issue+json";
const STATUS_MIME_TYPE: &str = "text/plain";
const QUERY_DATA_MIME_TYPE: &str = "application/x-www-form-urlencoded";

const QUERY_PARAM_NAME: &str = "q";
const TIME_RANGE_PARAM_NAME: &str = "time_range";
const ISSUE_ID_NAME: &str = "issue";

#[fp_export_impl(fp_provider)]
async fn get_supported_query_types(config: rmpv::Value) -> Vec<SupportedQueryType> {
    vec![
        SupportedQueryType {
            query_type: OVERVIEW_QUERY_TYPE.to_owned(),
            schema: vec![
                QueryField::Text(TextField {
                    name: QUERY_PARAM_NAME.to_owned(),
                    label: "Enter your Sentry query".to_owned(),
                    multiline: false,
                    prerequisites: Vec::new(),
                    required: false,
                    supports_highlighting: false,
                }),
                QueryField::DateTimeRange(DateTimeRangeField {
                    name: TIME_RANGE_PARAM_NAME.to_owned(),
                    label: "Specify a time range".to_owned(),
                    required: true,
                }),
            ],
            mime_types: vec![OVERVIEW_MIME_TYPE.to_owned()],
        },
        SupportedQueryType {
            query_type: ISSUE_QUERY_TYPE.to_owned(),
            schema: vec![QueryField::Number(NumberField {
                name: ISSUE_ID_NAME.to_owned(),
                label: "Sentry issue ID".to_owned(),
                required: true,
                max: None,
                min: None,
                step: None,
            })],
            mime_types: vec![ISSUE_MIME_TYPE.to_owned()],
        },
        SupportedQueryType {
            query_type: STATUS_QUERY_TYPE.to_owned(),
            schema: Vec::new(),
            mime_types: vec![STATUS_MIME_TYPE.to_owned()],
        },
    ]
}

#[fp_export_impl(fp_provider)]
async fn invoke2(request: ProviderRequest) -> Result<Blob, Error> {
    log(format!(
        "sentry provider invoked with request: {:?}",
        request
    ));

    let config: SentryConfig =
        rmpv::ext::from_value(request.config).map_err(|err| Error::Config {
            message: format!("Error parsing config: {:?}", err),
        })?;

    match request.query_type.as_str() {
        OVERVIEW_QUERY_TYPE => query_issues_overview(request.query_data, config).await,
        ISSUE_QUERY_TYPE => query_issue_details(request.query_data, config).await,
        STATUS_QUERY_TYPE => Ok(Blob {
            mime_type: STATUS_MIME_TYPE.to_owned(),
            data: ByteBuf::from("ok"),
        }),
        _ => {
            log(format!(
                "sentry provider received unsupported query type: {:?}",
                &request.query_type
            ));
            Err(Error::UnsupportedRequest)
        }
    }
}

#[fp_export_impl(fp_provider)]
fn create_cells(query_type: String, response: Blob) -> Result<Vec<Cell>, Error> {
    match query_type.as_str() {
        OVERVIEW_QUERY_TYPE => create_overview_cells(response),
        ISSUE_QUERY_TYPE => create_issue_cells(response),
        _ => {
            log(format!(
                "sentry provider cannot create cells for query type: {:?}",
                &query_type
            ));
            Err(Error::UnsupportedRequest)
        }
    }
}

fn create_overview_cells(response: Blob) -> Result<Vec<Cell>, Error> {
    if response.mime_type != OVERVIEW_MIME_TYPE {
        log(format!(
            "sentry provider cannot overview cells for MIME type: {:?}",
            &response.mime_type
        ));
        return Err(Error::UnsupportedRequest);
    }

    let issues: Vec<SentryIssue> =
        serde_json::from_slice(response.data.as_ref()).map_err(|err| Error::Deserialization {
            message: format!("Cannot parse Sentry overview issues: {err}"),
        })?;

    let cells: Vec<_> = issues
        .into_iter()
        .map(|issue| {
            let id = issue.id;
            let issue_url = format!("provider:sentry,{ISSUE_QUERY_TYPE}?issue={id}");
            let issue_link_text = format!("Issue {id}: {}", issue.title);
            let content = format!("{issue_link_text}\nLast reported: {}", issue.last_seen);
            let formatting = vec![
                AnnotationWithOffset {
                    annotation: Annotation::StartLink { url: issue_url },
                    offset: 0,
                },
                AnnotationWithOffset {
                    annotation: Annotation::EndLink,
                    offset: char_count(&issue_link_text),
                },
            ];

            Cell::ListItem(ListItemCell {
                id,
                content,
                formatting: Some(formatting),
                list_type: ListType::Unordered,
                ..Default::default()
            })
        })
        .collect();

    Ok(cells)
}

async fn query_issues_overview(query_data: Blob, config: SentryConfig) -> Result<Blob, Error> {
    let query = get_overview_query(&query_data)?;
    let url = format!(
        "https://sentry.io/api/0/projects/{}/{}/issues/?query={}",
        config.organization_slug, config.project_slug, query
    );
    let headers = HashMap::from([(
        "Authorization".to_owned(),
        format!("Bearer {}", config.token),
    )]);

    let result = make_http_request(HttpRequest {
        body: None,
        headers: Some(headers),
        method: HttpRequestMethod::Get,
        url,
    })
    .await;
    match result {
        Ok(response) => convert_response(&response.body),
        Err(error) => Err(Error::Http { error }),
    }
}

fn convert_response(response: &ByteBuf) -> Result<Blob, Error> {
    let issues: Vec<SentryIssue> =
        serde_json::from_slice(response.as_slice()).map_err(|err| Error::Deserialization {
            message: format!("Cannot parse Sentry response: {err}"),
        })?;

    let response = Blob {
        data: ByteBuf::from(
            serde_json::to_vec(&issues).map_err(|err| Error::Deserialization {
                message: format!("Cannot parse Sentry response: {err}"),
            })?,
        ),
        mime_type: OVERVIEW_MIME_TYPE.to_owned(),
    };

    Ok(response)
}

fn get_overview_query(query_data: &Blob) -> Result<String, Error> {
    if query_data.mime_type != QUERY_DATA_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }

    let mut query = String::new();
    for (key, value) in form_urlencoded::parse(&query_data.data) {
        match key.as_ref() {
            QUERY_PARAM_NAME => {
                if !query.is_empty() {
                    query.push(' ');
                }

                query.push_str(value.as_ref());
            }
            TIME_RANGE_PARAM_NAME => {
                if !query.is_empty() {
                    query.push(' ');
                }

                if let Some((from, to)) = value.split_once(' ') {
                    query.push_str(&format!("timestamp:>={from} timestamp:<{to}"));
                }
            }
            _ => {}
        }
    }
    Ok(query)
}

async fn query_issue_details(_query_data: Blob, _config: SentryConfig) -> Result<Blob, Error> {
    todo!("Querying issue details not yet implemented")
}

fn create_issue_cells(response: Blob) -> Result<Vec<Cell>, Error> {
    todo!("Creating issue cells not yet implemented")
}

fn iso_time_to_f64(timestamp: &str) -> f64 {
    match OffsetDateTime::parse(timestamp, &Rfc3339) {
        Ok(time) => time.unix_timestamp_nanos() as f64 / 1_000_000_000.0,
        Err(_) => f64::NAN,
    }
}
