use super::DateTimeRange;
use fiberplane_provider_bindings::Timestamp;
use serde::de::{self, Visitor};
use serde::Deserialize;
use std::fmt;

impl<'de> Deserialize<'de> for DateTimeRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeRangeVisitor)
    }
}

struct DateTimeRangeVisitor;

impl<'de> Visitor<'de> for DateTimeRangeVisitor {
    type Value = DateTimeRange;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a string containing 2 timestamps in RFC3339 format separated by a space")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Some((from_ts, to_ts)) = value.split_once(' ') {
            let from = Timestamp::parse(from_ts)
                .map_err(|e| E::custom(format!("could not parse the 'from' timestamp: {e}")))?;
            let to = Timestamp::parse(to_ts)
                .map_err(|e| E::custom(format!("could not parse the 'to' timestamp: {e}")))?;
            Ok(DateTimeRange { from, to })
        } else {
            Err(E::custom("wrong format. The correct format is '{from_rfc_3339_timestamp} {to_rfc_3339_timestamp}'. There is a single space between the timestamps."))
        }
    }
}
