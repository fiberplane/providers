mod config;
mod percent_encode;
mod sentry;

use config::SentryConfig;
use const_format::formatcp;
use fiberplane_models::{
    notebooks::{TableColumnId, TableRow, TableRowId, TableRowValue},
    providers::{STATUS_MIME_TYPE, STATUS_QUERY_TYPE},
    utils::content_writer::ContentWriter,
};
use fiberplane_pdk::prelude::*;
use percent_encode::encode_uri_component;
use sentry::*;
use std::{fmt::Write, str::FromStr};

const OVERVIEW_QUERY_TYPE: &str = "x-issues-overview";

const CELLS_MSGPACK_MIME_TYPE: &str = formatcp!("{CELLS_MIME_TYPE}+msgpack");

const QUERY_PARAM_NAME: &str = "q";
const TIME_RANGE_PARAM_NAME: &str = "time_range";
const LIVE_PARAM_NAME: &str = "live";

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
                CheckboxField::new()
                    .with_name(LIVE_PARAM_NAME)
                    .with_label("Enable live mode")
                    .with_value("true")
                    .into(),
            ])
            .supporting_mime_types(&[CELLS_MIME_TYPE]),
        SupportedQueryType::new(STATUS_QUERY_TYPE).supporting_mime_types(&[STATUS_MIME_TYPE]),
    ]
}

#[pdk_export]
async fn invoke2(request: ProviderRequest) -> Result<Blob> {
    init_panic_hook();
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
        STATUS_QUERY_TYPE => Ok(Blob::builder()
            .mime_type(STATUS_MIME_TYPE.to_owned())
            .data("ok")
            .build()),
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

    let response = make_http_request(HttpRequest::get(url).with_headers([(
        "Authorization".to_owned(),
        format!("Bearer {}", config.token),
    )]))
    .await?;

    let issues =
        serde_json::from_slice(response.body.as_ref()).map_err(|err| Error::Deserialization {
            message: format!("Cannot parse Sentry response: {err}"),
        })?;

    serialize_cells(create_overview_cells(issues)?)
}

fn get_overview_query(query_data: &Blob) -> Result<String> {
    if query_data.mime_type != FORM_ENCODED_MIME_TYPE {
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

fn create_name_value(issue: &SentryIssue) -> TableRowValue {
    let mut writer = ContentWriter::new();

    if let Some(issue_type) = issue.metadata.issue_type.to_owned() {
        writer = writer.write_bold_text(issue_type);
    }

    writer = writer
        .write_text(format!(" {} ", issue.culprit))
        .set_bold()
        .write_link("more >", issue.permalink.clone())
        .unset_bold()
        .write_text("\n");

    if let Some(value) = issue.metadata.value.to_owned() {
        writer = writer.write_italics_text(value).write_text("\n");
    }

    writer = writer
        .write_text(" Last seen: ")
        .write_timestamp(issue.last_seen)
        .write_text("\n");

    if !issue.has_seen {
        writer = writer.write_highlight_text(" New issue ").write_text(" ");
    }

    writer = writer.write_highlight_text(format!(" {} ", issue.project.name.clone()));

    if issue.is_unhandled {
        writer = writer
            .write_text(" ")
            .write_highlight_text(" Unhandled ")
            .write_text(" ");
    }

    writer.to_table_value()
}

fn create_overview_cells(issues: Vec<SentryIssue>) -> Result<Vec<Cell>> {
    let mut table_cell = TableCell::builder()
        .id("table".to_owned())
        .column_defs(vec![
            TableColumnDefinition::builder()
                .id(TableColumnId::from_str("name")?)
                .title("Name".to_owned())
                .build(),
            TableColumnDefinition::builder()
                .id(TableColumnId::from_str("events")?)
                .title("Events".to_owned())
                .build(),
        ])
        .build();

    table_cell.rows = issues
        .iter()
        .map(|issue| {
            let name_value = create_name_value(issue);

            let events_value = ContentWriter::new()
                .write_bold_text(issue.user_count.to_string())
                .to_table_value();

            Ok(TableRow::builder()
                .id(table_row_id_from_sentry_id(&issue.id)?)
                .values(vec![name_value, events_value])
                .build())
        })
        .collect::<Result<Vec<TableRow>>>()?;

    Ok(vec![Cell::Table(table_cell)])
}

fn serialize_cells(cells: Vec<Cell>) -> Result<Blob> {
    Ok(Blob::builder()
        .data(rmp_serde::to_vec_named(&cells)?)
        .mime_type(CELLS_MSGPACK_MIME_TYPE.to_owned())
        .build())
}

fn table_row_id_from_sentry_id(sentry_id: &str) -> Result<TableRowId> {
    TableRowId::from_str(
        &sentry_id
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>(),
    )
    .map_err(Error::from)
}
