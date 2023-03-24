use super::DateTimeRange;
use serde::Serialize;

impl Serialize for DateTimeRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self { from, to } = self;
        serializer.serialize_str(&format!("{from} {to}",))
    }
}
