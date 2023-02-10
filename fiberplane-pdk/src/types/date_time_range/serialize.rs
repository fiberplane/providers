use super::DateTimeRange;
use serde::{ser::Error, Serialize};
use time::format_description::well_known::Rfc3339;

impl Serialize for DateTimeRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self { from, to } = self;
        serializer.serialize_str(&format!(
            "{} {}",
            from.format(&Rfc3339).map_err(Error::custom)?,
            to.format(&Rfc3339).map_err(Error::custom)?,
        ))
    }
}
