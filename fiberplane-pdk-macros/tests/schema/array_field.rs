//! This test makes sure that code does not compile when using a Vec<T>
//! in an array field, where T is a non-builtin structure that doesn't
//! have the "schema" class function.

use fiberplane_pdk_macros::QuerySchema;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RowItem {
    name: String,
    value: String,
}

#[derive(Deserialize, QuerySchema)]
pub struct MissingRowSchema {
    table: Vec<RowItem>,
}

pub fn main() {}
