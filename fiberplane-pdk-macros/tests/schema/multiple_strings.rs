use fiberplane_models::providers::{QueryField, TextField};
use fiberplane_pdk_macros::QuerySchema;
use serde::Deserialize;

#[derive(Deserialize, QuerySchema)]
pub struct BunchOfStrings {
    list: Vec<String>,
}

pub fn main() {
    let schema = BunchOfStrings::schema();
    assert_eq!(
        schema,
        vec![QueryField::from(
            TextField::builder()
                .name("list".to_string())
                .multiple(true)
                .label(String::new())
                .multiline(false)
                .placeholder(String::new())
                .prerequisites(vec![])
                .required(false)
                .supports_suggestions(false)
                .build()
        )]
    )
}
