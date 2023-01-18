/// `Result` type that uses `fiberplane_pdk::prelude::Error` as Error type.
pub type Result<T> = core::result::Result<T, crate::bindings::Error>;
