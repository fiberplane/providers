use crate::bindings::{Blob, Error};
use crate::providers::{ProviderStatus, STATUS_MIME_TYPE};
use crate::types::Result;
use serde::{de::DeserializeOwned, Serialize};

/// Trait to be implemented by custom data types for convenient parsing and
/// serialization to/from `Blob`s.
///
/// Implementations can be derived using the `ProviderData` derive macro.
pub trait ProviderData: Sized {
    /// Parses a `Blob` with the correct MIME type into the struct implementing
    /// this trait.
    fn parse_blob(blob: Blob) -> Result<Self>;

    /// Serializes an instance of this type and stores the result in a `Blob`.
    fn to_blob(&self) -> Result<Blob>;
}

impl ProviderData for ProviderStatus {
    fn parse_blob(blob: Blob) -> Result<Self> {
        parse_blob(STATUS_MIME_TYPE, blob)
    }

    fn to_blob(&self) -> Result<Blob> {
        to_blob(STATUS_MIME_TYPE, &self)
    }
}

/// Parses a `Blob` with the correct MIME type into a custom struct.
///
/// You probably want to use the `ProviderData` derive macro and use the
/// struct's `parse()` method.
pub fn parse_blob<T: DeserializeOwned>(mime_type: &str, blob: Blob) -> Result<T> {
    if blob.mime_type == format!("{mime_type}+msgpack") {
        return rmp_serde::from_read(blob.data.as_ref()).map_err(|err| Error::Data {
            message: format!("Could not parse blob: {err}"),
        });
    }

    if blob.mime_type == format!("{mime_type}+json") {
        return serde_json::from_reader(blob.data.as_ref()).map_err(|err| Error::Data {
            message: format!("Could not parse blob: {err}"),
        });
    }

    return Err(Error::Data {
        message: format!("Incorrect MIME type: {}", blob.mime_type),
    });
}

/// Serializes a custom struct and stores the result in a `Blob`.
///
/// You probably want to use the `ProviderData` derive macro and use the
/// struct's `serialize()` method.
pub fn to_blob<T: Serialize>(mime_type: &str, data: &T) -> Result<Blob> {
    if cfg!(debug_assertions) {
        Ok(Blob {
            data: serde_json::to_vec(data)?.into(),
            mime_type: format!("{mime_type}+json"),
        })
    } else {
        Ok(Blob {
            data: rmp_serde::to_vec_named(data)?.into(),
            mime_type: format!("{mime_type}+msgpack"),
        })
    }
}
