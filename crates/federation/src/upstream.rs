use std::time::Duration;

use dash_proto::dpr_v1::{Binding, FetchRequest, FetchResponse, ResolveRequest, ResolveResponse, VersionRange};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub url: String,
    pub priority: u32,
    pub upstream_type: UpstreamType,
    pub namespace_filter: Option<String>,
    pub auth: Option<UpstreamAuth>,
    pub timeout: Option<Duration>,
}

impl UpstreamConfig {
    pub fn new_dash(name: impl Into<String>, url: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            priority,
            upstream_type: UpstreamType::Dash,
            namespace_filter: None,
            auth: None,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    pub fn new_npm_bridge(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            url: "https://registry.npmjs.org".to_string(),
            priority,
            upstream_type: UpstreamType::NpmBridge,
            namespace_filter: None,
            auth: None,
            timeout: Some(Duration::from_secs(60)),
        }
    }

    pub fn with_namespace_filter(mut self, filter: impl Into<String>) -> Self {
        self.namespace_filter = Some(filter.into());
        self
    }

    pub fn with_auth(mut self, auth: UpstreamAuth) -> Self {
        self.auth = Some(auth);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UpstreamType {
    Dash,
    NpmBridge,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum UpstreamAuth {
    Bearer { token: String },
    Oidc { issuer: String, audience: String },
    ApiKey { key: String },
    None,
}

impl Default for UpstreamAuth {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpstreamError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("timeout")]
    Timeout,
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("upstream returned error: {status} - {body}")]
    UpstreamError { status: u16, body: String },
    #[error("namespace not allowed: {0}")]
    NamespaceNotAllowed(String),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct UpstreamClient {
    pub config: UpstreamConfig,
    http_client: Client,
}

impl UpstreamClient {
    pub fn new(config: UpstreamConfig) -> Result<Self, UpstreamError> {
        let timeout = config.timeout.unwrap_or(Duration::from_secs(30));
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(UpstreamError::Http)?;

        Ok(Self { config, http_client })
    }

    pub fn config(&self) -> &UpstreamConfig {
        &self.config
    }

    pub async fn resolve(&self, name: &str, ranges: &[VersionRange]) -> Result<Vec<Binding>, UpstreamError> {
        if let Some(filter) = &self.config.namespace_filter {
            let namespace = name.split('/').next().unwrap_or(name);
            if namespace != filter && !name.starts_with(&format!("{}/", filter)) {
                return Err(UpstreamError::NamespaceNotAllowed(
                    format!("namespace {} not allowed for upstream {}", namespace, self.config.name)
                ));
            }
        }

        match self.config.upstream_type {
            UpstreamType::Dash => self.resolve_dash(name, ranges).await,
            UpstreamType::NpmBridge => self.resolve_npm_bridge(name, ranges).await,
            UpstreamType::Custom => self.resolve_custom(name, ranges).await,
        }
    }

    async fn resolve_dash(&self, name: &str, ranges: &[VersionRange]) -> Result<Vec<Binding>, UpstreamError> {
        let url = format!("{}/api/v1/resolve", self.config.url.trim_end_matches('/'));
        let req = ResolveRequest {
            name: name.to_string(),
            version_ranges: ranges.to_vec(),
            registry: String::new(),
        };

        let response = timeout(self.config.timeout.unwrap_or(Duration::from_secs(30)), async {
            self.http_client
                .post(&url)
                .json(&req)
                .send()
                .await
        })
        .await
        .map_err(|_| UpstreamError::Timeout)??
        .error_for_status()
        .map_err(|e| {
            let status = e.status().map(|s| s.as_u16()).unwrap_or(0);
            UpstreamError::UpstreamError { status, body: e.to_string() }
        })?;

        let resp: ResolveResponse = response.json().await.map_err(UpstreamError::Http)?;
        Ok(resp.bindings)
    }

    async fn resolve_npm_bridge(&self, name: &str, ranges: &[VersionRange]) -> Result<Vec<Binding>, UpstreamError> {
        let metadata_url = format!("{}/{}", self.config.url.trim_end_matches('/'), name);
        let response = timeout(self.config.timeout.unwrap_or(Duration::from_secs(60)), async {
            self.http_client.get(&metadata_url).send().await
        })
        .await
        .map_err(|_| UpstreamError::Timeout)??
        .error_for_status()
        .map_err(|e| {
            let status = e.status().map(|s| s.as_u16()).unwrap_or(0);
            UpstreamError::UpstreamError { status, body: e.to_string() }
        })?;

        let metadata: serde_json::Value = response.json().await.map_err(UpstreamError::Http)?;
        let versions = metadata
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| UpstreamError::InvalidResponse("no versions field".into()))?;

        let mut bindings = Vec::new();
        for (ver_str, _ver_data) in versions {
            if let Ok(version) = semver::Version::parse(ver_str) {
                let matches = if ranges.is_empty() {
                    true
                } else {
                    ranges.iter().any(|r| version_matches_range(&version, r))
                };
                if matches {
                    bindings.push(Binding {
                        version: ver_str.clone(),
                        cid: String::new(),
                        log_index: 0,
                        inclusion_proof: None,
                        provenance: None,
                    });
                }
            }
        }

        Ok(bindings)
    }

    async fn resolve_custom(&self, _name: &str, _ranges: &[VersionRange]) -> Result<Vec<Binding>, UpstreamError> {
        Err(UpstreamError::Config("custom upstream not implemented".into()))
    }

    pub async fn fetch(&self, cid: &str, ranges: &[dash_proto::dpr_v1::Range]) -> Result<FetchResponse, UpstreamError> {
        let url = format!("{}/api/v1/fetch", self.config.url.trim_end_matches('/'));
        let req = FetchRequest {
            cid: cid.to_string(),
            ranges: ranges.to_vec(),
        };

        let response = timeout(self.config.timeout.unwrap_or(Duration::from_secs(30)), async {
            self.http_client
                .post(&url)
                .json(&req)
                .send()
                .await
        })
        .await
        .map_err(|_| UpstreamError::Timeout)??
        .error_for_status()
        .map_err(|e| {
            let status = e.status().map(|s| s.as_u16()).unwrap_or(0);
            UpstreamError::UpstreamError { status, body: e.to_string() }
        })?;

        let resp: FetchResponse = response.json().await.map_err(UpstreamError::Http)?;
        Ok(resp)
    }

    pub async fn publish(&self, _capsule_bytes: &[u8], _identity: &str) -> Result<ResolveResponse, UpstreamError> {
        Err(UpstreamError::Config("publish not implemented for upstreams".into()))
    }
}

fn version_matches_range(version: &semver::Version, range: &VersionRange) -> bool {
    use dash_proto::dpr_v1::version_range::Range;
    match &range.range {
        Some(Range::Exact(exact)) => semver::Version::parse(exact).map(|v| v == *version).unwrap_or(false),
        Some(Range::Constraint(c)) => {
            let low_ok = if let Ok(low_ver) = semver::Version::parse(&c.low) {
                if c.low_inclusive { *version >= low_ver } else { *version > low_ver }
            } else { false };
            if !low_ok { return false; }
            if !c.high.is_empty() {
                if let Ok(high_ver) = semver::Version::parse(&c.high) {
                    if c.high_inclusive { *version <= high_ver } else { *version < high_ver }
                } else { false }
            } else { true }
        },
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_config_dash() {
        let config = UpstreamConfig::new_dash("public", "https://reg.dash.dev", 1);
        assert_eq!(config.name, "public");
        assert_eq!(config.upstream_type, UpstreamType::Dash);
        assert_eq!(config.priority, 1);
    }

    #[test]
    fn test_upstream_config_npm_bridge() {
        let config = UpstreamConfig::new_npm_bridge("npm", 2);
        assert_eq!(config.name, "npm");
        assert_eq!(config.upstream_type, UpstreamType::NpmBridge);
        assert_eq!(config.priority, 2);
    }

    #[test]
    fn test_upstream_config_with_namespace() {
        let config = UpstreamConfig::new_dash("internal", "https://internal.dev", 0)
            .with_namespace_filter("@internal");
        assert_eq!(config.namespace_filter, Some("@internal".to_string()));
    }

    #[test]
    fn test_version_matches_range_exact() {
        let v = semver::Version::parse("1.2.3").unwrap();
        let range = VersionRange {
            range: Some(dash_proto::dpr_v1::version_range::Range::Exact("1.2.3".to_string())),
        };
        assert!(version_matches_range(&v, &range));

        let range = VersionRange {
            range: Some(dash_proto::dpr_v1::version_range::Range::Exact("1.2.4".to_string())),
        };
        assert!(!version_matches_range(&v, &range));
    }

    #[test]
    fn test_version_matches_range_constraint() {
        let v = semver::Version::parse("1.5.0").unwrap();
        let range = VersionRange {
            range: Some(dash_proto::dpr_v1::version_range::Range::Constraint(
                dash_proto::dpr_v1::RangeConstraint {
                    low: "1.0.0".to_string(),
                    high: "2.0.0".to_string(),
                    low_inclusive: true,
                    high_inclusive: false,
                }
            )),
        };
        assert!(version_matches_range(&v, &range));

        let v2 = semver::Version::parse("2.0.0").unwrap();
        assert!(!version_matches_range(&v2, &range));
    }
}