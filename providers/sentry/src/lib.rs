mod config;
mod percent_encode;
mod sentry;

use config::SentryConfig;
use fiberplane_models::utils::char_count;
use fiberplane_pdk::prelude::*;
use futures::future;
use percent_encode::encode_uri_component;
use sentry::*;
use std::{collections::HashMap, fmt::Write};

const OVERVIEW_QUERY_TYPE: &str = "x-issues-overview";
const ISSUE_QUERY_TYPE: &str = "x-issue-details";
const STATUS_QUERY_TYPE: &str = "status";

const CELLS_MIME_TYPE: &str = "application/vnd.fiberplane.cells+msgpack";
const STATUS_MIME_TYPE: &str = "text/plain";
const QUERY_DATA_MIME_TYPE: &str = "application/x-www-form-urlencoded";

const QUERY_PARAM_NAME: &str = "q";
const TIME_RANGE_PARAM_NAME: &str = "time_range";
const ISSUE_ID_NAME: &str = "issue";

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[pdk_export]
async fn get_supported_query_types(_config: ProviderConfig) -> Vec<SupportedQueryType> {
    vec![
        SupportedQueryType::new(OVERVIEW_QUERY_TYPE)
            .with_schema(vec![
                TextField::new()
                    .with_name(QUERY_PARAM_NAME)
                    .with_label("Enter your Sentry query")
                    .into(),
                DateTimeRangeField::new()
                    .with_name(TIME_RANGE_PARAM_NAME)
                    .with_label("Specify a time range")
                    .required()
                    .into(),
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(ISSUE_QUERY_TYPE)
            .with_schema(vec![TextField::new()
                .with_name(ISSUE_ID_NAME)
                .with_label("Sentry issue ID")
                .required()
                .into()])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob> {
    log(format!(
        "Sentry provider (commit: {}, built at: {}) invoked for query type \"{}\" and query data \"{:?}\"",
        COMMIT_HASH, BUILD_TIMESTAMP, request.query_type, request.query_data
    ));

    let config: SentryConfig =
        serde_json::from_value(request.config).map_err(|err| Error::Config {
            message: format!("Error parsing config: {err:?}"),
        })?;

    match request.query_type.as_str() {
        OVERVIEW_QUERY_TYPE => query_issues_overview(request.query_data, config).await,
        ISSUE_QUERY_TYPE => query_issue_details(request.query_data, config).await,
        STATUS_QUERY_TYPE => Ok(Blob {
            mime_type: STATUS_MIME_TYPE.to_owned(),
            data: "ok".into(),
        }),
        _ => Err(Error::UnsupportedRequest),
    }
}

async fn query_issues_overview(query_data: Blob, config: SentryConfig) -> Result<Blob> {
    let query = get_overview_query(&query_data)?;
    let url = format!(
        "https://sentry.io/api/0/projects/{}/{}/issues/?query={}",
        encode_uri_component(&config.organization_slug),
        encode_uri_component(&config.project_slug),
        encode_uri_component(&query)
    );
    let headers = HashMap::from([(
        "Authorization".to_owned(),
        format!("Bearer {}", config.token),
    )]);

    let response = make_http_request(HttpRequest {
        body: None,
        headers: Some(headers),
        method: HttpRequestMethod::Get,
        url,
    })
    .await?;

    let issues =
        serde_json::from_slice(response.body.as_ref()).map_err(|err| Error::Deserialization {
            message: format!("Cannot parse Sentry response: {err}"),
        })?;

    serialize_cells(create_overview_cells(issues)?)
}

fn get_overview_query(query_data: &Blob) -> Result<String> {
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
                    write!(&mut query, "timestamp:>={from} timestamp:<{to}").map_err(|error| {
                        Error::Data {
                            message: format!("Could not write query string: {error}"),
                        }
                    })?;
                }
            }
            _ => {}
        }
    }
    Ok(query)
}

fn create_overview_cells(issues: Vec<SentryIssue>) -> Result<Vec<Cell>> {
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
                formatting,
                list_type: ListType::Unordered,
                ..Default::default()
            })
        })
        .collect();

    Ok(cells)
}

async fn query_issue_details(query_data: Blob, config: SentryConfig) -> Result<Blob> {
    let issue_id = encode_uri_component(&get_issue_id(&query_data)?);
    let issue_url = format!("https://sentry.io/api/0/issues/{issue_id}/");
    let event_url = format!("{issue_url}events/latest/");

    let headers = HashMap::from([(
        "Authorization".to_owned(),
        format!("Bearer {}", config.token),
    )]);

    let issue_result = make_http_request(HttpRequest {
        body: None,
        headers: Some(headers.clone()),
        method: HttpRequestMethod::Get,
        url: issue_url,
    });
    let event_result = make_http_request(HttpRequest {
        body: None,
        headers: Some(headers),
        method: HttpRequestMethod::Get,
        url: event_url,
    });

    let (issue_response, event_response) = future::join(issue_result, event_result).await;
    let issue: SentryIssue =
        serde_json::from_slice(issue_response?.body.as_ref()).map_err(|err| {
            Error::Deserialization {
                message: format!("Cannot parse Sentry issue: {err}"),
            }
        })?;

    let event: SentryEvent =
        serde_json::from_slice(event_response?.body.as_ref()).map_err(|err| {
            Error::Deserialization {
                message: format!("Cannot parse Sentry event: {err}"),
            }
        })?;

    // TODO: We need to query the values for the tags separately:
    // let tag_keys = issue
    //     .tags
    //     .iter()
    //     .map(|tag| {
    //         format!(
    //             "key={}",
    //             percent_encode(tag.key.as_bytes(), NON_ALPHANUMERIC)
    //         )
    //     })
    //     .collect::<Vec<_>>()
    //     .join("&");
    // let url = format!("https://sentry.io/api/0/issues/{issue_id}/tags/?{tag_keys}");

    serialize_cells(create_issue_cells(issue, event, config)?)
}

fn get_issue_id(query_data: &Blob) -> Result<String> {
    if query_data.mime_type != QUERY_DATA_MIME_TYPE {
        return Err(Error::UnsupportedRequest);
    }

    for (key, value) in form_urlencoded::parse(&query_data.data) {
        match key.as_ref() {
            ISSUE_ID_NAME if !value.is_empty() => return Ok(value.into()),
            _ => {}
        }
    }
    Err(Error::ValidationError {
        errors: vec![ValidationError {
            field_name: ISSUE_ID_NAME.to_owned(),
            message: "No issue ID given".to_owned(),
        }],
    })
}

fn create_issue_cells(
    issue: SentryIssue,
    event: SentryEvent,
    config: SentryConfig,
) -> Result<Vec<Cell>> {
    let mut cells = vec![Cell::Heading(HeadingCell {
        id: "heading".to_owned(),
        heading_type: HeadingType::H3,
        content: format!("Issue {}: {}", issue.id, issue.title),
        ..Default::default()
    })];

    cells.push(Cell::Text(TextCell {
        id: "tags".to_owned(),
        content: "Tags: TODO".to_owned(),
        ..Default::default()
    }));

    let stacktrace = event
        .entries
        .iter()
        .find_map(|entry| match entry {
            SentryEventEntry::Exception { data } => Some(data),
            _ => None,
        })
        .and_then(|exception_data| exception_data.values.first())
        .and_then(|exception| exception.stacktrace.as_ref());
    if let Some(stacktrace) = stacktrace {
        let frames: Vec<_> = stacktrace
            .frames
            .iter()
            .map(|frame| {
                match (
                    frame.filename.as_ref(),
                    frame.function.as_ref(),
                    frame.line_no,
                    frame.col_no,
                ) {
                    (Some(filename), Some(function), Some(line_no), Some(col_no))
                        if line_no != 0 =>
                    {
                        format!("{filename}: {function} at line {line_no}:{col_no}")
                    }
                    (Some(filename), Some(function), _, _) => {
                        format!("{filename}: {function}")
                    }
                    (Some(filename), _, Some(line_no), Some(col_no)) if line_no != 0 => {
                        format!("{filename} at line {line_no}:{col_no}")
                    }
                    _ => "(unknown)".to_owned(),
                }
            })
            .collect();

        cells.push(Cell::Code(CodeCell {
            id: "stacktrace".to_owned(),
            content: format!("Stack trace:\n{}", frames.join("\n")),
            ..Default::default()
        }));
    }

    cells.push(Cell::Text(TextCell {
        id: "reported".to_owned(),
        content: format!("Reported: {}", issue.first_seen),
        formatting: vec![
            AnnotationWithOffset {
                annotation: Annotation::StartBold,
                offset: 0,
            },
            AnnotationWithOffset {
                annotation: Annotation::EndBold,
                offset: 9,
            },
        ],
        ..Default::default()
    }));

    let breadcrumbs_url = format!(
        "https://sentry.io/organizations/{}/issues/{}/#breadcrumbs",
        config.organization_slug, issue.id
    );
    cells.push(Cell::Text(TextCell {
        id: "breadcrumbs".to_owned(),
        content: format!("Breadcrumbs: {breadcrumbs_url}"),
        formatting: vec![AnnotationWithOffset {
            annotation: Annotation::StartLink {
                url: breadcrumbs_url,
            },
            offset: 13,
        }],
        ..Default::default()
    }));

    Ok(cells)
}

fn serialize_cells(cells: Vec<Cell>) -> Result<Blob> {
    Ok(Blob {
        data: rmp_serde::to_vec_named(&cells)?.into(),
        mime_type: CELLS_MIME_TYPE.to_owned(),
    })
}
