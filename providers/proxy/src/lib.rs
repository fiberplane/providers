use fp_provider::*;
use std::collections::HashMap;

#[fp_export_impl(fp_provider)]
async fn invoke(request: ProviderRequest, _config: Config) -> ProviderResponse {
    match request {
        ProviderRequest::Proxy(request) => proxy_request(request).await,
        _ => ProviderResponse::Error {
            error: Error::UnsupportedRequest,
        },
    }
}

async fn proxy_request(request: ProxyRequest) -> ProviderResponse {
    let mut headers = HashMap::new();
    headers.insert("Accept".to_owned(), "application/x-msgpack".to_owned());
    headers.insert(
        "Content-Type".to_owned(),
        "application/x-msgpack".to_owned(),
    );

    match make_http_request(HttpRequest {
        body: Some(request.request),
        headers: Some(headers),
        method: HttpRequestMethod::Post,
        url: format!(
            "/api/proxies/{}/relay?dataSourceName={}",
            urlencoding::encode(&request.proxy_id),
            urlencoding::encode(&request.data_source_name)
        ),
    })
    .await
    {
        Ok(response) => {
            rmp_serde::from_slice(&response.body).unwrap_or_else(|err| ProviderResponse::Error {
                error: Error::Deserialization {
                    message: format!("Error deserializing proxy response: {:?}", err),
                },
            })
        }
        Err(error) => ProviderResponse::Error {
            error: Error::Http { error },
        },
    }
}
