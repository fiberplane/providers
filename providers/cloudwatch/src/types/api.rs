//! Payloads read and returned by the API

pub mod cloudwatch;
pub mod cloudwatch_logs;
pub mod paginate;
pub mod resource_groups_tagging;

use fiberplane_pdk::prelude::Timestamp as FpTimestamp;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// It seems that all 'GET' requests get tagged, and all 'POST' requests aren't
/// An API response from the SDK
///
/// Variants are named after the API Action they represent a Response to.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(untagged)]
pub enum SdkResponse {
    #[allow(missing_docs)]
    Tagged(TaggedSdkResponse),
    #[allow(missing_docs)]
    GetMetricData(cloudwatch::GetMetricDataResponse),
    #[allow(missing_docs)]
    GetResources(resource_groups_tagging::GetResourcesResponse),
    #[allow(missing_docs)]
    GetTagKeys(resource_groups_tagging::GetTagKeysResponse),
    #[allow(missing_docs)]
    GetTagValues(resource_groups_tagging::GetTagValuesResponse),
}

/// A tagged API response from the SDK
///
/// Variants are named after the API Action they represent a Response to.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TaggedSdkResponse {
    #[allow(missing_docs)]
    ListMetricsResponse(cloudwatch::ListMetricsResponse),
}

/// Representation of a timestamp in AWS API
///
/// AWS uses "integers" representing the unix timestamp (in milliseconds),
/// but some of AWS products use scientific notation to represent these integers, which is
/// against the specification of JSON, so a lot of parsers (including this one) will
/// deserialize the timestamp to a floating point number. Therefore we ser/de the timestamps
/// to f64 and then perform conversion.
///
/// More context in https://github.com/serde-rs/json/issues/774
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timestamp(f64);

impl From<Timestamp> for FpTimestamp {
    fn from(ts: Timestamp) -> Self {
        Self(OffsetDateTime::from_unix_timestamp(ts.0.round() as i64).unwrap())
    }
}

impl From<FpTimestamp> for Timestamp {
    fn from(ts: FpTimestamp) -> Self {
        // Lossy conversion
        Self(ts.0.unix_timestamp() as f64)
    }
}

// But then again, some AWS API actually use integers for their timestamps
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntTimestamp(i64);

impl From<IntTimestamp> for FpTimestamp {
    fn from(ts: IntTimestamp) -> Self {
        Self(OffsetDateTime::from_unix_timestamp(ts.0).unwrap())
    }
}

impl From<FpTimestamp> for IntTimestamp {
    fn from(ts: FpTimestamp) -> Self {
        Self(ts.0.unix_timestamp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_metrics() {
        let payload = r#"
{
  "ListMetricsResponse": {
    "ListMetricsResult": {
      "Metrics": [
        {
          "Dimensions": [
            {
              "Name": "TargetGroup",
              "Value": "targetgroup/k8s-default-web-1111111111/abbaabbaabbaabba"
            },
            {
              "Name": "LoadBalancer",
              "Value": "app/k8s-productioncluster-1111111111/abbaabbaabbaabba"
            },
            {
              "Name": "AvailabilityZone",
              "Value": "eu-central-1b"
            }
          ],
          "MetricName": "HealthyStateRouting",
          "Namespace": "AWS/ApplicationELB"
        },
        {
          "Dimensions": [
            {
              "Name": "TargetGroup",
              "Value": "targetgroup/k8s-default-api-1111111111/abbaabbaabbaabba"
            },
            {
              "Name": "AvailabilityZone",
              "Value": "eu-central-1c"
            }
          ],
          "MetricName": "RequestCountPerTarget",
          "Namespace": "AWS/ApplicationELB"
        }
      ],
      "NextToken": "WJkbeW7IzJT+dApzTeXEUwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+uA7lsXyGLh9L4tYRsJ3ljZdrnrqwPNim4RY389mKsJmiO+XMX2QeANHwdpssF5LeT4xaY20JGUuq6NKCoIXs3LIND36inpMrIBG3+la1AGnW7rq+rnM3lzb40oifI79XTJzhM6bGh92Jq0GuPa4sva0B2flEGs5DyYH0IWPIyvn2Uj11XTjHtDXxf3u+BK6ayGMXhzpfTx6sMG2RejmALEoVxgDUmv/uJctTmkNnUCPNk+3VH0XB00E8JuBO51x5+E9XUC",
      "OwningAccounts": null,
      "TotalMetricsCount": null
    },
    "ResponseMetadata": {
      "RequestId": "de069e92-bd42-41ff-8367-13f0b6e74fb8"
    }
  }
}
"#;

        let response: SdkResponse = serde_json::from_str(payload).unwrap();
        assert!(matches!(
            response,
            SdkResponse::Tagged(TaggedSdkResponse::ListMetricsResponse(_))
        ));
    }

    #[test]
    fn get_resources() {
        let payload = r#"
{
  "PaginationToken": "",
  "ResourceTagMappingList": [
    {
      "ResourceARN": "arn:aws:ec2:eu-central-1:1111111111111:vpc/vpc-beef0000abba11112",
      "Tags": [
        {
          "Key": "env",
          "Value": "demo"
        },
        {
          "Key": "Name",
          "Value": "demo-cluster"
        }
      ]
    },
    {
      "ResourceARN": "arn:aws:eks:eu-central-1:1111111111111:cluster/demo-cluster",
      "Tags": [
        {
          "Key": "env",
          "Value": "demo"
        }
      ]
    },
    {
      "ResourceARN": "arn:aws:rds:eu-central-1:1111111111111:db:demo-1",
      "Tags": [
        {
          "Key": "env",
          "Value": "demo"
        }
      ]
    },
    {
      "ResourceARN": "arn:aws:s3:::battery-staple",
      "Tags": [
        {
          "Key": "env",
          "Value": "demo"
        }
      ]
    }
  ]
}
"#;
        let response: SdkResponse = serde_json::from_str(payload).unwrap();
        assert!(matches!(response, SdkResponse::GetResources(_)));
    }

    #[test]
    fn get_metric_data() {
        let payload = r#"
{"Messages":[],"MetricDataResults":[{"Id":"expr_0","Label":"expr_0","StatusCode":"Complete","Timestamps":[1.67119356E9,1.6711935E9,1.67119344E9,1.67119338E9,1.67119332E9,1.67119326E9,1.6711932E9,1.67119314E9,1.67119308E9,1.67119302E9,1.67119296E9,1.6711929E9,1.67119284E9,1.67119278E9,1.67119272E9,1.67119266E9,1.6711926E9,1.67119254E9,1.67119248E9,1.67119242E9,1.67119236E9,1.6711923E9,1.67119224E9,1.67119218E9,1.67119212E9,1.67119206E9,1.671192E9,1.67119194E9,1.67119188E9,1.67119182E9,1.67119176E9,1.6711917E9,1.67119164E9,1.67119158E9,1.67119152E9,1.67119146E9,1.6711914E9,1.67119134E9,1.67119128E9,1.67119122E9,1.67119116E9,1.6711911E9,1.67119104E9,1.67119098E9,1.67119092E9,1.67119086E9,1.6711908E9,1.67119074E9,1.67119068E9,1.67119062E9,1.67119056E9,1.6711905E9,1.67119044E9,1.67119038E9,1.67119032E9,1.67119026E9,1.6711902E9,1.67119014E9,1.67119008E9,1.67119002E9],"Values":[8.11615297282456,8.48669495866197,5.9882853853397995,7.741259862836311,8.165583063001643,7.834054608941809,7.968104739688233,6.1245171902318996,8.176842676225878,7.907111423933654,9.099355666933972,8.013197798188195,5.846608526859784,7.888115065808758,7.930625095265094,7.846498685217828,7.871573871668589,6.028316661339242,8.342462192768142,7.844317254561598,8.116079184337716,8.042344788961719,6.016569082606069,8.179928952371421,8.051965130593459,7.941692702035872,8.111849286709116,5.990317419368319,7.792370954113576,7.922187504342194,7.950781253463696,7.692942367153502,6.182383065784908,7.70971551249459,7.854187146293937,7.913163638701207,8.016058489163218,5.765580797995421,8.049264482079648,7.950118537599551,7.866049256344901,8.014343092845174,5.934336351741884,7.814694195718954,7.894344923195216,7.933368324974029,7.857665031100332,5.946973570009301,7.9972605230414695,7.832625875432595,8.129922365284838,7.7097240563604466,5.9665788914942945,8.032609760343844,7.957676972719159,8.083440372108177,8.119438147093925,5.822506119794508,8.11312603804827,8.070816578348678]}]}
"#;
        let response: SdkResponse = serde_json::from_str(payload).unwrap();
        assert!(matches!(response, SdkResponse::GetMetricData(_)));
        let _: cloudwatch::GetMetricDataResponse = serde_json::from_str(payload).unwrap();
    }
}
