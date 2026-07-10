//! HTTP client for communicating with Dash package registries.
//!
//! Provides methods for fetching package metadata, searching,
//! publishing packages, and managing authentication with registries.

use std::sync::Arc;
use std::time::Duration;

/// Error type for registry client operations.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("HTTP error: {status} {message}")]
    Http { status: u16, message: String },
    #[error("request failed: {0}")]
    Request(String),
    #[error("authentication error: {0}")]
    Auth(#[from] dash_auth::AuthError),
    #[error("network error: {0}")]
    Network(#[from] dash_network::NetworkError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("rate limited: retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },
}

/// Response from a successful package publish.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublishResponse {
    /// Whether the publish was successful.
    pub ok: bool,
    /// Registry revision ID.
    pub revision: Option<String>,
    /// Package name.
    pub name: String,
    /// Published version.
    pub version: String,
}

/// Response from a package search query.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResponse {
    /// Total number of results.
    pub total: u64,
    /// Search results on this page.
    pub objects: Vec<SearchResult>,
    /// Search time in milliseconds.
    pub time: f64,
}

/// A single search result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    /// Package metadata.
    pub package: RegistryPackage,
    /// Search score.
    pub score: Option<f64>,
    /// Search relevance.
    pub search_score: Option<f64>,
    /// Whether this is a high-quality package.
    pub flagged: Option<bool>,
}

/// Package metadata retrieved from the registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryPackage {
    /// Package name.
    pub name: String,
    /// Package scope (if scoped).
    pub scope: Option<String>,
    /// Latest version.
    pub version: String,
    /// Package description.
    pub description: Option<String>,
    /// All available versions.
    pub versions: std::collections::BTreeMap<String, RegistryPackageVersion>,
    /// Distribution tags (e.g., "latest", "next").
    pub dist_tags: std::collections::BTreeMap<String, String>,
    /// Package author.
    pub author: Option<serde_json::Value>,
    /// Package license.
    pub license: Option<String>,
    /// Package homepage.
    pub homepage: Option<String>,
    /// Package repository.
    pub repository: Option<String>,
    /// Time of last modification.
    pub time: Option<std::collections::BTreeMap<String, chrono::DateTime<chrono::Utc>>>,
}

/// A specific version of a package from the registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryPackageVersion {
    /// Version string.
    pub version: String,
    /// Package name at this version.
    pub name: String,
    /// Dependencies at this version.
    pub dependencies: std::collections::BTreeMap<String, String>,
    /// Dev dependencies at this version.
    pub dev_dependencies: std::collections::BTreeMap<String, String>,
    /// Distribution metadata.
    pub dist: RegistryDist,
    /// Package manifest (raw JSON).
    pub manifest: Option<serde_json::Value>,
}

/// Distribution metadata from the registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryDist {
    /// Tarball URL.
    pub tarball: String,
    /// Integrity hash.
    pub integrity: Option<String>,
    /// SHA-1 checksum.
    pub shasum: Option<String>,
    /// Unpacked file count.
    pub file_count: Option<u32>,
    /// Unpacked size.
    pub unpacked_size: Option<u64>,
    /// Tarball content type.
    pub content_type: Option<String>,
}

/// Configuration for the registry client.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryClientConfig {
    /// Base URL of the registry.
    pub base_url: String,
    /// Authentication token.
    pub token: Option<String>,
    /// Request timeout.
    pub timeout_secs: u64,
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Whether to verify SSL certificates.
    pub verify_ssl: bool,
}

impl Default for RegistryClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://registry.dash.sh".to_string(),
            token: None,
            timeout_secs: 30,
            max_retries: 3,
            verify_ssl: true,
        }
    }
}

/// HTTP client for interacting with a Dash package registry.
pub struct RegistryClient {
    client: reqwest::Client,
    base_url: String,
    auth: Option<Arc<dash_auth::Authenticator>>,
    config: RegistryClientConfig,
}

impl RegistryClient {
    /// Create a new registry client.
    pub fn new(config: RegistryClientConfig) -> Result<Self, RegistryError> {
        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(dash_network::user_agent())
            .default_headers(dash_network::default_headers());

        if !config.verify_ssl {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder
            .build()
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        let auth = config.token.as_ref().map(|_| {
            let store = Arc::new(dash_auth::MemoryTokenStore::new());
            Arc::new(
                dash_auth::Authenticator::new(
                    dash_auth::AuthProvider::Jwt,
                    None, // No JWT secret for client-side
                    store,
                )
            )
        });

        Ok(Self {
            client,
            base_url: config.base_url.clone(),
            auth,
            config,
        })
    }

    /// Create a new registry client from a dash config.
    pub fn from_config(config: &dash_config::Config) -> Result<Self, RegistryError> {
        let client_config = RegistryClientConfig {
            base_url: config.get_or("registry.url", "https://registry.dash.sh".to_string()),
            token: config.get("registry.token").ok(),
            timeout_secs: config.get_or("registry.timeout_secs", 30u64),
            max_retries: config.get_or("registry.max_retries", 3u32),
            verify_ssl: config.get_or("registry.verify_ssl", true),
        };
        Self::new(client_config)
    }

    /// Build a request with proper headers and auth.
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let req = self.client.request(method, &url);

        let req = if let Some(ref token) = self.config.token {
            req.bearer_auth(token)
        } else {
            req
        };

        req
    }

    /// Fetch package metadata from the registry.
    pub async fn get_package(&self, name: &str) -> Result<RegistryPackage, RegistryError> {
        let path = format!("/packages/{}", name);
        let response = self
            .request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        if !status.is_success() {
            return match status.as_u16() {
                404 => Err(RegistryError::NotFound(name.to_string())),
                429 => Err(RegistryError::RateLimited { retry_after: None }),
                code => Err(RegistryError::Http {
                    status: code,
                    message: body,
                }),
            };
        }

        serde_json::from_str(&body).map_err(RegistryError::Serialization)
    }

    /// Get package metadata by version.
    pub async fn get_package_version(
        &self,
        name: &str,
        version: &str,
    ) -> Result<RegistryPackageVersion, RegistryError> {
        let pkg = self.get_package(name).await?;
        pkg.versions
            .get(version)
            .cloned()
            .ok_or_else(|| RegistryError::NotFound(format!("{name}@{version}")))
    }

    /// Publish a package to the registry.
    pub async fn publish(
        &self,
        name: &str,
        version: &str,
        tarball: Vec<u8>,
        manifest: serde_json::Value,
    ) -> Result<PublishResponse, RegistryError> {
        let path = format!("/packages/{}/versions", name);

        let body = serde_json::json!({
            "name": name,
            "version": version,
            "tarball": base64_encode(&tarball),
            "manifest": manifest,
        });

        let response = self
            .request(reqwest::Method::PUT, &path)
            .json(&body)
            .send()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        if status.is_success() {
            Ok(PublishResponse {
                ok: true,
                revision: body.get("revision").and_then(|v| v.as_str()).map(String::from),
                name: name.to_string(),
                version: version.to_string(),
            })
        } else {
            Err(RegistryError::Http {
                status: status.as_u16(),
                message: body.to_string(),
            })
        }
    }

    /// Search for packages.
    pub async fn search(
        &self,
        query: &str,
        size: Option<u32>,
        from: Option<u32>,
    ) -> Result<SearchResponse, RegistryError> {
        let mut path = format!("/search?text={}", urlencoding(query));
        if let Some(s) = size {
            path.push_str(&format!("&size={s}"));
        }
        if let Some(f) = from {
            path.push_str(&format!("&from={f}"));
        }

        let response = self
            .request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(RegistryError::Http {
                status: status.as_u16(),
                message: body,
            });
        }

        serde_json::from_str(&body).map_err(RegistryError::Serialization)
    }

    /// Deprecate a package version.
    pub async fn deprecate(
        &self,
        name: &str,
        version: &str,
        message: &str,
    ) -> Result<(), RegistryError> {
        let path = format!("/packages/{}/versions/{}/deprecate", name, version);

        let body = serde_json::json!({
            "message": message,
        });

        let response = self
            .request(reqwest::Method::PUT, &path)
            .json(&body)
            .send()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_default();
            return Err(RegistryError::Http {
                status: status.as_u16(),
                message: text,
            });
        }

        Ok(())
    }

    /// Ping the registry to check availability.
    pub async fn ping(&self) -> Result<bool, RegistryError> {
        let response = self
            .request(reqwest::Method::GET, "/-/ping")
            .send()
            .await
            .map_err(|e| RegistryError::Request(e.to_string()))?;

        Ok(response.status().is_success())
    }

    /// Get the registry base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Set the auth token.
    pub fn set_token(&mut self, token: Option<String>) {
        self.config.token = token;
    }
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_client_config_default() {
        let config = RegistryClientConfig::default();
        assert_eq!(config.base_url, "https://registry.dash.sh");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.verify_ssl);
    }

    #[tokio::test]
    async fn test_ping_no_registry() {
        let config = RegistryClientConfig {
            base_url: "http://localhost:1".to_string(),
            timeout_secs: 1,
            ..Default::default()
        };

        let client = RegistryClient::new(config).unwrap();
        let result = client.ping().await;
        // Should fail as there's no registry running
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_response() {
        let resp = PublishResponse {
            ok: true,
            revision: Some("abc123".to_string()),
            name: "test-pkg".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(resp.ok);
    }

    #[test]
    fn test_search_result() {
        let result = SearchResult {
            package: RegistryPackage {
                name: "test".to_string(),
                scope: None,
                version: "1.0.0".to_string(),
                description: Some("A test package".to_string()),
                versions: std::collections::BTreeMap::new(),
                dist_tags: std::collections::BTreeMap::new(),
                author: None,
                license: None,
                homepage: None,
                repository: None,
                time: None,
            },
            score: Some(0.95),
            search_score: Some(0.85),
            flagged: Some(false),
        };
        assert_eq!(result.package.name, "test");
    }

    #[test]
    fn test_registry_dist() {
        let dist = RegistryDist {
            tarball: "https://example.com/pkg.tar.gz".to_string(),
            integrity: Some("sha256-abc".to_string()),
            shasum: None,
            file_count: Some(10),
            unpacked_size: Some(1024),
            content_type: None,
        };
        assert!(!dist.tarball.is_empty());
    }

    #[test]
    fn test_urlencoding() {
        let encoded = urlencoding("hello world");
        assert_eq!(encoded, "hello+world");
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode(b"hello");
        assert!(!encoded.is_empty());
    }

    #[tokio::test]
    async fn test_get_package_no_registry() {
        let config = RegistryClientConfig {
            base_url: "http://localhost:1".to_string(),
            timeout_secs: 1,
            ..Default::default()
        };

        let client = RegistryClient::new(config).unwrap();
        let result = client.get_package("test-pkg").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_set_token() {
        let config = RegistryClientConfig::default();
        let mut client = RegistryClient::new(config).unwrap();
        assert!(client.config.token.is_none());
        client.set_token(Some("my-token".to_string()));
        assert_eq!(client.config.token.as_deref(), Some("my-token"));
    }
}
