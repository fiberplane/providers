use fiberplane_pdk::bindings::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::constants::CELLS_MSGPACK_MIME_TYPE;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct HttpsProviderResponse {
    pub(crate) status: String,
    pub(crate) headers: Option<BTreeMap<String, String>>,
    pub(crate) payload: Vec<u8>,
}

impl TryFrom<HttpResponse> for HttpsProviderResponse {
    fn try_from(resp: HttpResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            status: resp.status_code.to_string(),
            headers: Some(resp.headers),
            payload: resp.body.to_vec(),
        })
    }

    type Error = Error;
}

impl TryFrom<HttpRequestError> for HttpsProviderResponse {
    fn try_from(err: HttpRequestError) -> Result<Self, Self::Error> {
        let payload = if let HttpRequestError::ServerError { ref response, .. } = err {
            response.to_vec()
        } else {
            Vec::new()
        };

        let status = if let HttpRequestError::ServerError {
            ref status_code, ..
        } = err
        {
            status_code.to_string()
        } else {
            serde_json::to_value(err)
                .map_err(|e| Error::Data {
                    message: format!("Error serializing http error: {e:?}"),
                })?
                .to_string()
        };

        Ok(Self {
            payload,
            status,
            headers: None,
        })
    }
    type Error = Error;
}

impl TryFrom<Result<HttpResponse, HttpRequestError>> for HttpsProviderResponse {
    fn try_from(result: Result<HttpResponse, HttpRequestError>) -> Result<Self, Self::Error> {
        match result {
            Ok(response) => response.try_into(),
            Err(error) => error.try_into(),
        }
    }
    type Error = Error;
}

impl HttpsProviderResponse {
    pub(crate) fn try_into_blob(self, with_headers: bool) -> Result<Blob, Error> {
        let filtered_result = Self {
            headers: if with_headers { self.headers } else { None },
            ..self
        };
        serialize_cells(filtered_result.try_into_cells()?)
    }

    fn text_cell(id: String, content: String) -> Cell {
        Cell::Text(
            TextCell::builder()
                .id(id)
                .content(content)
                .formatting(Vec::new())
                .read_only(true)
                .build(),
        )
    }

    pub(crate) fn try_into_cells(self) -> Result<Vec<Cell>, Error> {
        let status_cell = Cell::Code(
            CodeCell::builder()
                .id("status".to_string())
                .content(self.status)
                .syntax("json".to_string())
                .read_only(true)
                .build(),
        );
        let headers_cell = self.headers.as_ref().map(|headers| {
            Cell::Code(
                CodeCell::builder()
                    .id("headers".to_string())
                    .content(format!("{headers:#?}"))
                    .syntax("json".to_string())
                    .read_only(true)
                    .build(),
            )
        });
        let response_cell = Cell::Code(
            CodeCell::builder()
                .id("response".to_string())
                .content(
                    serde_json::from_slice::<Value>(self.payload.as_slice())
                        .and_then(|value| serde_json::to_string_pretty(&value))
                        .unwrap_or_else(|_| {
                            String::from_utf8_lossy(self.payload.as_slice()).to_string()
                        }),
                )
                .syntax("json".to_string())
                .read_only(true)
                .build(),
        );
        let results = {
            let mut accumulator = vec![
                Self::text_cell("status-heading".to_string(), "Status Code".to_string()),
                status_cell,
                Self::text_cell("response-heading".to_string(), "Response".to_string()),
                response_cell,
            ];
            if let Some(headers_cell) = headers_cell {
                accumulator.push(Self::text_cell(
                    "headers-heading".to_string(),
                    "Response Headers".to_string(),
                ));
                accumulator.push(headers_cell);
            }
            accumulator
        };
        Ok(results)
    }
}

fn serialize_cells(cells: Vec<Cell>) -> Result<Blob, Error> {
    Ok(Blob::builder()
        .data(rmp_serde::to_vec_named(&cells)?)
        .mime_type(CELLS_MSGPACK_MIME_TYPE.to_owned())
        .build())
}
