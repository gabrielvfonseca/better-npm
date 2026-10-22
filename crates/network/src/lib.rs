//! HTTP networking layer: retry, backoff, rate limiting, middleware.
//!
//! Provides utilities for URL handling, HTTP methods, retry logic,
//! and default headers for Bnpm's registry communication.

use std::time::Duration;
use url::Url;

/// Error type for network operations.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("request failed after {retries} retries: {message}")]
    RetryExhausted { retries: u32, message: String },
    #[error("timeout: {0}")]
    Timeout(String),
}

/// HTTP methods supported by Bnpm's network layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl HttpMethod {
    /// Return the method as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Join a base URL with a relative path.
pub fn url_join(base: &str, path: &str) -> Result<String, NetworkError> {
    let mut url = Url::parse(base)?;
    url.path_segments_mut()
        .map_err(|_| NetworkError::UrlParse(url::ParseError::RelativeUrlWithoutBase))?
        .push(path.trim_start_matches('/'));
    Ok(url.to_string())
}

/// Check whether a string is a valid URL.
pub fn is_valid_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}

/// Return the default HTTP headers used by Bnpm clients.
pub fn default_headers() -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        user_agent().parse().unwrap(),
    );
    headers.insert(
        http::header::ACCEPT,
        "application/json".parse().unwrap(),
    );
    headers.insert(
        http::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    headers
}

/// Return the Bnpm user-agent string.
pub fn user_agent() -> String {
    format!("Bnpm/{}", env!("CARGO_PKG_VERSION"))
}

/// Configuration for HTTP request retries with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay before the first retry (milliseconds).
    pub base_delay_ms: u64,
    /// Maximum delay between retries (milliseconds).
    pub max_delay_ms: u64,
    /// Multiplier applied to the delay after each retry.
    pub backoff_factor: f64,
    /// HTTP status codes that should trigger a retry.
    pub retryable_statuses: Vec<http::StatusCode>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 200,
            max_delay_ms: 10_000,
            backoff_factor: 2.0,
            retryable_statuses: vec![
                http::StatusCode::TOO_MANY_REQUESTS,
                http::StatusCode::SERVICE_UNAVAILABLE,
                http::StatusCode::GATEWAY_TIMEOUT,
                http::StatusCode::BAD_GATEWAY,
                http::StatusCode::INTERNAL_SERVER_ERROR,
            ],
        }
    }
}

/// Execute an async operation with retry logic.
pub async fn with_retry<T, F, Fut, E>(f: F, config: &RetryConfig) -> Result<T, NetworkError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == config.max_retries {
                    return Err(NetworkError::RetryExhausted {
                        retries: config.max_retries,
                        message: e.to_string(),
                    });
                }
                last_error = Some(e);

                let delay_ms = (config.base_delay_ms as f64
                    * config.backoff_factor.powi(attempt as i32))
                    .min(config.max_delay_ms as f64) as u64;

                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(NetworkError::RetryExhausted {
        retries: config.max_retries,
        message: last_error.map(|e| e.to_string()).unwrap_or_default(),
    })
}

/// Check whether an HTTP status code is in the retryable set.
pub fn is_retryable_status(status: http::StatusCode, config: &RetryConfig) -> bool {
    config.retryable_statuses.contains(&status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_join() {
        let result = url_join("https://registry.example.com", "/api/v1/packages").unwrap();
        assert_eq!(result, "https://registry.example.com/api/v1/packages");

        let result = url_join("https://registry.example.com/v2/", "packages").unwrap();
        assert_eq!(result, "https://registry.example.com/v2/packages");
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://localhost:3000"));
        assert!(!is_valid_url("not a url"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_default_headers() {
        let headers = default_headers();
        assert!(headers.contains_key(http::header::USER_AGENT));
        assert!(headers.contains_key(http::header::ACCEPT));
    }

    #[test]
    fn test_user_agent() {
        let ua = user_agent();
        assert!(ua.starts_with("Bnpm/"));
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    #[test]
    fn test_is_retryable_status() {
        let config = RetryConfig::default();
        assert!(is_retryable_status(http::StatusCode::SERVICE_UNAVAILABLE, &config));
        assert!(is_retryable_status(http::StatusCode::TOO_MANY_REQUESTS, &config));
        assert!(!is_retryable_status(http::StatusCode::NOT_FOUND, &config));
    }

    #[tokio::test]
    async fn test_with_retry_success() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
            max_delay_ms: 100,
            ..Default::default()
        };

        let result = with_retry(
            || async { Ok::<_, String>(42) },
            &config,
        )
        .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_retry_exhausted() {
        let config = RetryConfig {
            max_retries: 1,
            base_delay_ms: 10,
            max_delay_ms: 100,
            ..Default::default()
        };

        let result = with_retry(
            || async { Err::<i32, _>("always fails".to_string()) },
            &config,
        )
        .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NetworkError::RetryExhausted { retries, .. } => {
                assert_eq!(retries, 1);
            }
            _ => panic!("wrong error type"),
        }
    }
}
