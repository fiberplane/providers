use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

mod bindings;
mod types;

pub use bindings::*;
use types::*;

pub async fn make_request(req: Request) -> Result<Response, RequestError> {
    let url = req.url;

    let client = reqwest::Client::new();
    let mut builder = match req.method {
        RequestMethod::Delete => client.delete(url.clone()),
        RequestMethod::Get => client.get(url.clone()),
        RequestMethod::Head => client.head(url.clone()),
        RequestMethod::Post => client.post(url.clone()),
    };
    if let Some(body) = req.body {
        builder = builder.body(body);
    }
    if let Some(headers) = req.headers {
        for (key, value) in headers.iter() {
            builder = builder.header(key, value);
        }
    }

    match builder.send().await {
        Ok(res) => {
            let status_code = res.status().as_u16();

            let mut headers = HashMap::new();
            for (key, value) in res.headers().iter() {
                if let Ok(value) = value.to_str() {
                    headers.insert(key.to_string(), value.to_owned());
                } else {
                    eprintln!(
                        "HTTP header containing invalid utf8 omitted in response from \"{}\"",
                        url
                    );
                }
            }

            match res.bytes().await {
                Ok(body) => {
                    let body = body.to_vec();
                    if (200..300).contains(&status_code) {
                        Ok(Response {
                            body,
                            headers,
                            status_code,
                        })
                    } else {
                        Err(RequestError::ServerError {
                            response: body,
                            status_code,
                        })
                    }
                }
                Err(error) => {
                    eprintln!("Could not read HTTP response from \"{}\": {:?}", url, error);
                    Err(RequestError::Other {
                        reason: "Unexpected end of data".to_owned(),
                    })
                }
            }
        }
        Err(error) => Err(if error.is_timeout() {
            RequestError::Timeout
        } else {
            RequestError::Other {
                reason: error.to_string(),
            }
        }),
    }
}

fn now() -> Timestamp {
    let duration = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(_) => {
            eprintln!("System time is set before epoch! Returning epoch as fallback.");
            return 0.0;
        }
    };

    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}

fn log(message: String) {
    eprintln!("Provider log: {}", message);
}

fn random(len: u32) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(len as usize);
    for _ in 0..vec.capacity() {
        vec.push(rng.gen());
    }
    vec
}
