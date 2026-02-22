use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// HTTP header with enable/disable toggle
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// HTTP request configuration
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub body: String,
}

/// HTTP response data
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    pub status_code: u16,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub response_time_ms: u32,
    pub error: Option<String>,
}

#[derive(Debug, thiserror::Error, specta::Type, Serialize)]
pub enum HttpClientError {
    #[error("Request failed: {0}")]
    RequestError(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
}

/// Send an HTTP request and return the response
#[tauri::command]
#[specta::specta]
pub async fn send_http_request(request: HttpRequest) -> Result<HttpResponse, HttpClientError> {
    // Validate URL
    if request.url.is_empty() {
        return Err(HttpClientError::InvalidUrl("URL is required".to_string()));
    }

    // Add http:// if no protocol specified
    let url = if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        format!("http://{}", request.url)
    } else {
        request.url.clone()
    };

    // Build headers
    let mut header_map = HeaderMap::new();
    for h in &request.headers {
        if h.enabled && !h.key.is_empty() {
            let name = HeaderName::try_from(h.key.as_str())
                .map_err(|e| HttpClientError::InvalidHeader(format!("{}: {}", h.key, e)))?;
            let value = HeaderValue::try_from(h.value.as_str())
                .map_err(|e| HttpClientError::InvalidHeader(format!("{}: {}", h.value, e)))?;
            header_map.insert(name, value);
        }
    }

    // Set default Content-Type for requests with body
    if !request.body.is_empty() && !header_map.contains_key("content-type") {
        header_map.insert(
            HeaderName::try_from("content-type").unwrap(),
            HeaderValue::try_from("application/json").unwrap(),
        );
    }

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HttpClientError::RequestError(e.to_string()))?;

    // Build request
    let method = request.method.to_uppercase();
    let mut req_builder = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "PATCH" => client.patch(&url),
        "DELETE" => client.delete(&url),
        "HEAD" => client.head(&url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
        _ => client.get(&url),
    };

    req_builder = req_builder.headers(header_map);

    // Add body for methods that support it
    if !request.body.is_empty() && matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
        req_builder = req_builder.body(request.body);
    }

    // Send request and measure time
    let start_time = Instant::now();
    let response = req_builder
        .send()
        .await
        .map_err(|e| HttpClientError::RequestError(e.to_string()))?;
    let response_time_ms = start_time.elapsed().as_millis() as u32;

    // Extract response info
    let status_code = response.status().as_u16();
    let status = response.status().to_string();

    // Extract headers
    let mut headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.to_string(), v.to_string());
        }
    }

    // Read body
    let body = response
        .text()
        .await
        .map_err(|e| HttpClientError::RequestError(format!("Failed to read body: {}", e)))?;

    // Format JSON if possible
    let formatted_body = format_json(&body);

    Ok(HttpResponse {
        status_code,
        status,
        headers,
        body: formatted_body,
        response_time_ms,
        error: None,
    })
}

/// Format JSON string with indentation
fn format_json(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| input.to_string()),
        Err(_) => input.to_string(),
    }
}
