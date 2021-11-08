use fp_bindgen::prelude::Serializable;

#[derive(Serializable, Debug)]
#[fp(rename_all = "camelCase")]

pub struct Config {
    url: Option<String>,
}
