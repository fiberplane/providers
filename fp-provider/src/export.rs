use crate::types::*;

#[fp_bindgen_support::fp_export_signature]
/// Fetches a data instant based on the given query and options.
pub async fn fetch_instant(query: String, opts: QueryInstantOptions) -> Result<Vec<Instant>, FetchError>;

#[fp_bindgen_support::fp_export_signature]
/// Fetches a series of data based on the given query and options.
pub async fn fetch_series(query: String, opts: QuerySeriesOptions) -> Result<Vec<Series>, FetchError>;