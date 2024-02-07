use crate as fiberplane_pdk; // To satisfy the `ProviderData` macro output.
use crate::bindings::Cell;
use crate::macros::ProviderData;
use crate::providers::{
    ProviderEvent, Suggestion, Timeseries, CELLS_MIME_TYPE, EVENTS_MIME_TYPE,
    SUGGESTIONS_MIME_TYPE, TIMESERIES_MIME_TYPE,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, ProviderData, Serialize)]
#[cfg_attr(
    feature = "fp-bindgen",
    derive(Serializable),
    fp(rust_module = "fiberplane_pdk::prelude")
)]
#[pdk(mime_type = CELLS_MIME_TYPE)]
pub struct Cells(pub Vec<Cell>);

#[derive(Clone, Debug, Deserialize, PartialEq, ProviderData, Serialize)]
#[cfg_attr(
    feature = "fp-bindgen",
    derive(Serializable),
    fp(rust_module = "fiberplane_pdk::prelude")
)]
#[pdk(mime_type = EVENTS_MIME_TYPE)]
pub struct Events(pub Vec<ProviderEvent>);

#[derive(Clone, Debug, Deserialize, PartialEq, ProviderData, Serialize)]
#[cfg_attr(
    feature = "fp-bindgen",
    derive(Serializable),
    fp(rust_module = "fiberplane_pdk::prelude")
)]
#[pdk(mime_type = SUGGESTIONS_MIME_TYPE)]
pub struct Suggestions(pub Vec<Suggestion>);

#[derive(Clone, Debug, Deserialize, PartialEq, ProviderData, Serialize)]
#[cfg_attr(
    feature = "fp-bindgen",
    derive(Serializable),
    fp(rust_module = "fiberplane_pdk::prelude")
)]
#[pdk(mime_type = TIMESERIES_MIME_TYPE)]
pub struct TimeseriesVector(pub Vec<Timeseries>);
