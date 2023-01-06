use time::OffsetDateTime;

/// Type to use for query data fields of type "date_time_range".
pub struct DateTimeRange {
    /// Start time of the range, inclusive.
    pub from: OffsetDateTime,

    /// End time of the range, exclusive.
    pub to: OffsetDateTime,
}
