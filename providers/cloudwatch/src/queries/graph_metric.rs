//! Graph Metric query handling
//!
//! # Good queries to save for snippets
//! - `select avg(CPUUtilization) from schema("AWS/RDS", DBInstanceIdentifier) group by DBInstanceIdentifier`
//! - `select max(ReadLatency) from schema("AWS/RDS", DBClusterIdentifier) group by DBClusterIdentifier`
//! - `select avg(DatabaseConnections) from schema("AWS/RDS", DBClusterIdentifier) group by DBClusterIdentifier`
//!   + ReadThroughput (also works by EngineName)
//!   + NetworkThroughput
//!   + StorageNetworkThroughput
//!   + CommitThroughput
//! - `select avg(CPUUtilization) from schema("AWS/EC2", InstanceId) group by InstanceId`
//! - `select avg("EBSIOBalance%") from schema("AWS/EC2", AutoScalingGroupName) group by AutoScalingGroupName`
//! - `select sum(HTTPCode_Target_2XX_Count) from schema("AWS/ApplicationELB", LoadBalancer) group by LoadBalancer`
//! - `select sum(HTTPCode_ELB_4XX_Count) from schema("AWS/ApplicationELB", LoadBalancer) group by LoadBalancer`
//!   + RequestCount
//! - `select avg(TargetResponseTime) from schema("AWS/ApplicationELB", LoadBalancer) group by LoadBalancer`
//! - `select sum(Invocations) from schema("AWS/Lambda", FunctionName) group by FunctionName`
//! - `select avg(NumberOfObjects) from schema("AWS/S3", BucketName) group by BucketName`
//!   + BucketSizeBytes
use crate::{
    api::{
        cloudwatch::{MetricDataResult, ScanOrder},
        resource_groups_tagging::ResourceTagMapping,
    },
    client::{cloudwatch::Client, resource_groups_tagging::Client as TagsClient},
    config::Config,
    constants::{
        EXPRESSION_PARAM_NAME, LABEL_PARAM_NAME, PERIOD_PARAM_NAME, TIMESERIES_MIME_TYPE,
        TIMESERIES_MSGPACK_MIME_TYPE, TIME_RANGE_PARAM_NAME,
    },
};
use fiberplane_models::providers::{Timeseries, FORM_ENCODED_MIME_TYPE};
use fiberplane_provider_bindings::{
    now, Blob, Cell, Error, GraphCell, GraphType, ProviderRequest, StackingType, Timestamp,
};
use std::collections::BTreeMap;

use super::try_from_iso_date;

pub async fn invoke2_handler(config: Config, request: ProviderRequest) -> Result<Blob, Error> {
    let request: GraphMetricRequest = request.query_data.try_into()?;
    let all_resources = TagsClient::from(&config)
        .list_all_resources(None)
        .await
        .unwrap_or_default();
    let client = Client::from(&config);
    client
        .get_metric_data(
            request.expressions,
            request.start_time,
            request.end_time,
            request.max_datapoints,
            request.order_points_by,
        )
        .await
        .and_then(|data| {
            let series: Vec<Timeseries> = data
                .into_iter()
                .map(|mdr| mdr_to_ts(mdr, &all_resources))
                .collect();
            Ok(Blob {
                mime_type: TIMESERIES_MSGPACK_MIME_TYPE.to_string(),
                data: rmp_serde::to_vec_named(&series)?.into(),
            })
        })
}

pub fn create_cells_handler(_response: Blob) -> Result<Vec<Cell>, Error> {
    let graph_cell = Cell::Graph(GraphCell {
        id: "graph".to_owned(),
        data_links: vec![format!("cell-data:{TIMESERIES_MIME_TYPE},self")],
        graph_type: GraphType::Line,
        read_only: None,
        stacking_type: StackingType::None,
    });

    Ok(vec![graph_cell])
}

struct GraphMetricRequest {
    expressions: Vec<(String, Option<String>, usize)>,
    start_time: Timestamp,
    end_time: Timestamp,
    max_datapoints: Option<u64>,
    order_points_by: Option<ScanOrder>,
}

impl TryFrom<Blob> for GraphMetricRequest {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self, Self::Error> {
        if blob.mime_type != FORM_ENCODED_MIME_TYPE {
            return Err(Error::UnsupportedRequest);
        }

        let mut expression = String::new();
        let mut label = None;
        let mut from = now();
        let mut to = now();
        let mut period = 0;
        for (key, value) in form_urlencoded::parse(&blob.data) {
            match key.as_ref() {
                EXPRESSION_PARAM_NAME => expression = value.to_string(),
                LABEL_PARAM_NAME => label = Some(value.to_string()),
                TIME_RANGE_PARAM_NAME => {
                    // TODO: Add validation error for non-compliant value
                    if let Some((ts_from, ts_to)) = value.split_once(' ') {
                        from = try_from_iso_date(ts_from)?;
                        to = try_from_iso_date(ts_to)?;
                    }
                }
                PERIOD_PARAM_NAME => {
                    // TODO: Add validation error for non-compliant value
                    period = value.parse().map_err(|err| Error::Deserialization {
                        message: format!("Invalid period: {err}"),
                    })?;
                }
                _ => {}
            }
        }

        Ok(Self {
            expressions: vec![(expression, label, period)],
            start_time: from,
            end_time: to,
            max_datapoints: None,
            order_points_by: None,
        })
    }
}

pub fn mdr_to_ts(res: MetricDataResult, resource_referential: &[ResourceTagMapping]) -> Timeseries {
    let labels = if let Some(label) = res.label {
        let mut acc: BTreeMap<String, String> = [("title".to_string(), label.clone())].into();

        if let Some(messages) = res.messages {
            for (id, message) in messages.iter().enumerate() {
                acc.insert(
                    message
                        .code
                        .clone()
                        .unwrap_or_else(|| format!("message {}", id + 1)),
                    message.value.clone().unwrap_or_default(),
                );
            }
        }

        if let Some(resource) = resource_referential.iter().find(|resource| {
            resource.resource_arn.as_ref().map_or(false, |arn| {
                // Here we assume that the label returned by AWS API will have the "Group By" argument value as the last
                // word.
                // So if
                // - the "pretty" label used as argument in the Provider request is "CPU Usage", and
                // - the query has been grouped by InstanceId in the AWS/EC2 namespace, then
                // we assume the label returned will be "CPU Usage i-beef2345571".
                //
                // Notably, this approach won't work if we group results by AvailabilityZone for example
                let test_label = label.rsplit(' ').next().unwrap();
                arn.ends_with(&test_label)
            })
        }) {
            if let Some(ref tags) = resource.tags {
                for tag in tags {
                    acc.insert(tag.key.to_string(), tag.value.to_string());
                }
            }
        }
        acc
    } else {
        Default::default()
    };

    let ts_iter = res.timestamps.unwrap().into_iter();
    let values_iter = res.values.unwrap().into_iter();

    let metrics = ts_iter
        .zip(values_iter)
        .map(|(time, value)| fiberplane_models::providers::Metric {
            time: time.into(),
            value,
            otel: Default::default(),
        })
        .collect();

    Timeseries {
        name: res.id.unwrap_or_else(|| "unnamed".into()),
        labels,
        metrics,
        otel: Default::default(),
        visible: true,
    }
}
