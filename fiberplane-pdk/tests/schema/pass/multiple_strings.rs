//! This test ensures that a structure containing a sequence of String
//! can generate a usable schema for provider queries.

// Note: users of the macro should never have to pick imports like that.
// Using simply `use fiberplane_pdk::prelude::*` should be enough.
use fiberplane_models::{
    blobs::Blob,
    providers::{QueryField, TextField, FORM_ENCODED_MIME_TYPE},
};
use fiberplane_pdk_macros::QuerySchema;
use serde::Deserialize;

#[derive(Deserialize, QuerySchema, PartialEq, Debug)]
pub struct BunchOfStrings {
    list: Vec<String>,
}

pub fn main() {
    let schema = BunchOfStrings::schema();
    assert_eq!(
        schema,
        vec![QueryField::from(
            TextField::new().with_name("list").multiple().required()
        )]
    );

    let input = Blob::builder()
        .mime_type(FORM_ENCODED_MIME_TYPE)
        .data("list[1]=first&list[2]=second")
        .build();

    let actual = BunchOfStrings::parse(input).unwrap();

    assert_eq!(
        actual,
        BunchOfStrings {
            list: vec!["first".to_string(), "second".to_string()]
        }
    );
}
