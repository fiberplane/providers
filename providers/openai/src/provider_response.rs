use fiberplane_pdk::bindings::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// use serde_aux::field_attributes::deserialize_number_from_string;
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
    pub(crate) fn try_into_blob(self) -> Result<Blob, Error> {
        let filtered_result = Self {
            headers: self.headers,
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
        let completion_cell = Cell::Text(
            TextCell::builder()
                .id("completion".to_string())
                .content(
                    match serde_json::from_slice::<OpenAiCompletionResponse>(self.payload.as_slice()) {
                        Ok(value) => {
                            if let Some(first_choice) = value.choices.get(0) {
                                first_choice.text.clone()
                            } else {
                                String::new()
                            }
                        }
                        Err(err) => {
                            log(format!("Error deserializing completion response: {err:?}"));
                            String::from_utf8_lossy(self.payload.as_slice()).to_string()
                        },
                    }
                )
                .read_only(true)
                .build(),
        );
        let results = {
            let mut accumulator = vec![
                completion_cell,
                // Self::text_cell("status-heading".to_string(), "Status Code".to_string()),
                // status_cell,
                // Self::text_cell("response-heading".to_string(), "Response".to_string()),
                // response_cell,
            ];
            // if let Some(headers_cell) = headers_cell {
            //     accumulator.push(Self::text_cell(
            //         "headers-heading".to_string(),
            //         "Response Headers".to_string(),
            //     ));
            //     accumulator.push(headers_cell);
            // }

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



#[derive(Deserialize, Serialize, Debug, Clone)]
struct OpenAiCompletionResponse {
    id: String,
    object: String,
    created: u32,
    model: String,
    choices: [Choice; 1],
    usage: Usage,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Choice {
    text: String,
    index: i32,
    finish_reason: String,
    // logprobs: Option<i32>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Usage {
    prompt_tokens: u16,
    completion_tokens: u16,
    total_tokens: u16
}

#[test]
fn test_deserialization() {
let sample = r#"
{
  "id": "cmpl-uqkvlQyYK7bGYrRHQ0eXlWi7",
  "object": "text_completion",
  "created": 1589478378,
  "model": "text-davinci-003",
  "choices": [
    {
      "text": "\n\nThis is indeed a test",
      "index": 0,
      "logprobs": null,
      "finish_reason": "length"
    }
  ],
  "usage": {
    "prompt_tokens": 5,
    "completion_tokens": 7,
    "total_tokens": 12
  }
}"#;
  
  let _ = serde_json::from_str::<OpenAiCompletionResponse>(sample).unwrap();
}
