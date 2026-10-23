// bnpm-federation: Federated registry implementation

use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use bnpm_capsule::Capsule;
use bnpm_cid::Cid;
use bnpm_governance::{CapabilityAcl, Scope};
use bnpm_manifest::Manifest as SchemaManifest;
use bnpm_proto::dpr_v1::{Binding, FetchResponse, MerkleProof, Range, VersionRange as ProtoVersionRange, version_range::Range as ProtoRange};
use bnpm_registry::{ContentService, NameService, VersionRange as LocalVersionRange};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::namespace_policy::NamespacePolicy;
use crate::transparency_log::{FederatedLogEntry, LogSource, TransparencyLog, TransparencyLogConfig, TransparencyLogError};
use crate::upstream::{UpstreamClient, UpstreamConfig, UpstreamError};

use blake3;
use hex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedRegistryConfig {
    pub upstreams: Vec<UpstreamConfig>,
    pub namespaces: Vec<NamespacePolicy>,
    pub local_acl: Option<CapabilityAcl>,
    pub log_config: TransparencyLogConfig,
    pub fetch_timeout: Option<StdDuration>,
    pub resolve_timeout: Option<StdDuration>,
}

impl Default for FederatedRegistryConfig {
    fn default() -> Self {
        Self {
            upstreams: Vec::new(),
            namespaces: Vec::new(),
            local_acl: None,
            log_config: TransparencyLogConfig::default(),
            fetch_timeout: Some(StdDuration::from_secs(30)),
            resolve_timeout: Some(StdDuration::from_secs(10)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveResult {
    pub bindings: Vec<Binding>,
    pub source: LogSource,
    pub upstream_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum FederatedError {
    #[error("no upstream available for namespace: {0}")]
    NoUpstream(String),
    #[error("all upstreams failed: {0}")]
    AllUpstreamsFailed(String),
    #[error("namespace not allowed: {0}")]
    NamespaceNotAllowed(String),
    #[error("upstream error: {0}")]
    UpstreamError(String),
    #[error("fetch error: {0}")]
    FetchError(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("invalid CID: {0}")]
    InvalidCid(String),
    #[error("transparency log error: {0}")]
    TransparencyLogError(#[from] TransparencyLogError),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<UpstreamError> for FederatedError {
    fn from(e: UpstreamError) -> Self {
        FederatedError::UpstreamError(e.to_string())
    }
}

pub struct FederatedRegistry {
    config: FederatedRegistryConfig,
    upstreams: Vec<UpstreamClient>,
    namespaces: Vec<NamespacePolicy>,
    local_acl: Option<CapabilityAcl>,
    local_name_service: Arc<NameService>,
    local_content_service: Arc<ContentService>,
    transparency_log: Arc<RwLock<TransparencyLog>>,
    _http_client: Client,
    _fetch_timeout: StdDuration,
    resolve_timeout: StdDuration,
}

impl FederatedRegistry {
    pub fn new(
        config: FederatedRegistryConfig,
        local_name_service: Arc<NameService>,
        local_content_service: Arc<ContentService>,
    ) -> Result<Self, FederatedError> {
        let mut upstreams = Vec::new();
        for upstream_config in &config.upstreams {
            match UpstreamClient::new(upstream_config.clone()) {
                Ok(client) => upstreams.push(client),
                Err(e) => {
                    warn!("Failed to create upstream client for {}: {}", upstream_config.name, e);
                }
            }
        }

        let transparency_log = Arc::new(RwLock::new(TransparencyLog::new(config.log_config.clone())));
        let _http_client = Client::new();
        let _fetch_timeout = config.fetch_timeout.unwrap_or(StdDuration::from_secs(30));
        let resolve_timeout = config.resolve_timeout.unwrap_or(StdDuration::from_secs(10));

        let local_acl = config.local_acl.clone();
        let namespaces = config.namespaces.clone();

        Ok(Self {
            config,
            upstreams,
            namespaces,
            local_acl,
            local_name_service,
            local_content_service,
            transparency_log,
            _http_client,
            _fetch_timeout,
            resolve_timeout,
        })
    }

    fn convert_ranges(ranges: &[ProtoVersionRange]) -> Vec<LocalVersionRange> {
        ranges.iter().map(|r| {
            match &r.range {
                Some(ProtoRange::Exact(exact)) => LocalVersionRange::Exact(exact.clone()),
                Some(ProtoRange::Constraint(c)) => LocalVersionRange::Constraint(
                    c.low.clone(),
                    c.high.clone(),
                    c.low_inclusive,
                    c.high_inclusive,
                ),
                None => LocalVersionRange::Exact(String::new()),
            }
        }).collect()
    }

    pub async fn resolve(
        &self,
        name: &str,
        ranges: &[ProtoVersionRange],
    ) -> Result<ResolveResult, FederatedError> {
        let start = Instant::now();
        let timeout = self.resolve_timeout;

        let namespace = self.extract_namespace(name);
        let namespace_policy = self.find_namespace_policy(&namespace);

        if let Some(policy) = &namespace_policy {
            if policy.local_only {
                debug!("Namespace {} is local_only, resolving locally", namespace);
                return self.resolve_local(name, &Self::convert_ranges(ranges)).await;
            }
        }

        let mut last_error = None;
        for upstream in &self.upstreams {
            if start.elapsed() >= timeout {
                return Err(FederatedError::Timeout("resolve timeout".into()));
            }

            if let Some(policy) = &namespace_policy {
                if policy.delegate_to_upstream && !policy.matches_upstream(&upstream.config.name) {
                    continue;
                }
            }

            debug!("Trying upstream {} for {}", upstream.config.name, name);
            match upstream.resolve(name, ranges).await {
                Ok(bindings) => {
                    if bindings.is_empty() {
                        continue;
                    }


                    let log_source = LogSource::Upstream {
                        upstream_name: upstream.config.name.clone(),
                        log_index: 0,
                    };

                    let entry = FederatedLogEntry {
                        name: name.to_string(),
                        bindings: bindings.clone(),
                        source: log_source.clone(),
                        timestamp: chrono::Utc::now(),
                        log_index: self.transparency_log.read().await.next_index(),
                    };
                    self.transparency_log.write().await.append(entry).await?;

                    return Ok(ResolveResult {
                        bindings,
                        source: log_source,
                        upstream_name: Some(upstream.config.name.clone()),
                    });
                }
                Err(e) => {
                    warn!("Upstream {} failed for {}: {}", upstream.config.name, name, e);
                    last_error = Some(e);
                }
            }
        }

        if let Some(policy) = &namespace_policy {
            if !policy.delegate_to_upstream {
                return self.resolve_local(name, &Self::convert_ranges(ranges)).await;
            }
        }

        if self.config.upstreams.is_empty() {
            return self.resolve_local(name, &Self::convert_ranges(ranges)).await;
        }

        Err(last_error.map(|e| FederatedError::UpstreamError(e.to_string())).unwrap_or_else(|| FederatedError::AllUpstreamsFailed(
            format!("all upstreams failed for {}", name)
        )))
    }

    async fn resolve_local(
        &self,
        name: &str,
        ranges: &[LocalVersionRange],
    ) -> Result<ResolveResult, FederatedError> {
        let bindings = self.local_name_service.resolve(name, ranges);
        let proto_bindings: Vec<Binding> = bindings.into_iter().map(|b| Binding {
            version: b.version,
            cid: b.cid,
            log_index: b.log_index,
            inclusion_proof: None,
            provenance: None,
        }).collect();

        Ok(ResolveResult {
            bindings: proto_bindings,
            source: LogSource::Local { log_index: 0 },
            upstream_name: None,
        })
    }

    pub async fn fetch(
        &self,
        cid: &str,
        ranges: &[Range],
    ) -> Result<FetchResponse, FederatedError> {
        let cid_parsed = cid.parse::<Cid>()
            .map_err(|e| FederatedError::InvalidCid(e.to_string()))?;

        if self.local_content_service.has(&cid_parsed.to_string()).await.unwrap_or(false) {
            let content = self.local_content_service.fetch(&cid_parsed.to_string()).await
                .map_err(|e| FederatedError::FetchError(e.to_string()))?;
            let content_hash = blake3::hash(&content);
            let merkle_proof = MerkleProof {
                root_hash: hex::encode(content_hash.as_bytes()),
                leaf_hash: hex::encode(content_hash.as_bytes()),
                siblings: vec![],
                leaf_index: 0,
                tree_size: 1,
            };
            return Ok(FetchResponse { content, merkle_proof: Some(merkle_proof) });
        }

        let mut last_error = None;
        for upstream in &self.upstreams {
            match upstream.fetch(cid, ranges).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("Upstream {} fetch failed for {}: {}", upstream.config.name, cid, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.map(|e| FederatedError::UpstreamError(e.to_string())).unwrap_or_else(|| FederatedError::FetchError(
            format!("no upstream available for fetch: {}", cid)
        )))
    }

    pub async fn publish_local(
        &self,
        name: &str,
        version: &str,
        capsule_bytes: &[u8],
        _identity: &str,
    ) -> Result<bnpm_proto::dpr_v1::PublishResponse, FederatedError> {
        let capsule = Capsule::from_bytes(capsule_bytes)
            .map_err(|e| FederatedError::Internal(format!("invalid capsule: {e}")))?;
        let cid = capsule.cid().to_string();

        self.local_content_service.store(capsule_bytes).await
            .map_err(|e| FederatedError::Internal(format!("store failed: {e}")))?;

        let log_index = self.transparency_log.read().await.next_index();
        self.local_name_service.register(
            name,
            version,
            &cid,
            log_index,
            None,
            None,
        ).map_err(|e| FederatedError::Internal(format!("register failed: {e}")))?;

        let entry = FederatedLogEntry {
            name: name.to_string(),
            bindings: vec![Binding {
                version: version.to_string(),
                cid: cid.clone(),
                log_index,
                inclusion_proof: None,
                provenance: None,
            }],
            source: LogSource::Local { log_index },
            timestamp: chrono::Utc::now(),
            log_index,
        };
        self.transparency_log.write().await.append(entry).await?;

        Ok(bnpm_proto::dpr_v1::PublishResponse {
            cid,
            version: version.to_string(),
            log_index,
            inclusion_proof: None,
        })
    }

    pub async fn get_transparency_log(&self) -> Vec<FederatedLogEntry> {
        self.transparency_log.read().await.entries().iter().cloned().collect()
    }

    fn extract_namespace(&self, name: &str) -> String {
        if let Some(idx) = name.find('/') {
            name[..idx].to_string()
        } else {
            name.to_string()
        }
    }

    fn find_namespace_policy(&self, namespace: &str) -> Option<&NamespacePolicy> {
        self.namespaces.iter().find(|p| p.matches(namespace))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use bnpm_capsule::Capsule;
    use bnpm_store::cas::CasStore;
    use tempfile::TempDir;

    async fn create_test_registry() -> (FederatedRegistry, TempDir) {
        let dir = TempDir::new().unwrap();
        let cas_store = CasStore::new(dir.path().join("cas")).await.unwrap();
        let content_service = Arc::new(ContentService::new(cas_store));
        let name_service = Arc::new(NameService::new());

        let config = FederatedRegistryConfig {
            upstreams: vec![],
            namespaces: vec![],
            local_acl: None,
            log_config: Default::default(),
            fetch_timeout: Some(StdDuration::from_secs(30)),
            resolve_timeout: Some(StdDuration::from_secs(10)),
        };

        let registry = FederatedRegistry::new(config, name_service, content_service).unwrap();
        (registry, dir)
    }

    #[tokio::test]
    async fn test_local_resolve_empty() {
        let (registry, _dir) = create_test_registry().await;
        let result = registry.resolve("nonexistent", &[]).await.unwrap();
        assert!(result.bindings.is_empty());
        assert!(matches!(result.source, LogSource::Local { .. }));
    }

    #[tokio::test]
    async fn test_local_publish_and_resolve() {
        let (registry, _dir) = create_test_registry().await;

        let capsule = Capsule::pack(vec![("test.txt".to_string(), b"hello".to_vec())]).unwrap();
        let capsule_bytes = capsule.to_bytes().unwrap();

        let publish_resp = registry.publish_local("test-pkg", "1.0.0", &capsule_bytes, "did:bnpm:test").await.unwrap();
        assert!(!publish_resp.cid.is_empty());

        let result = registry.resolve("test-pkg", &[]).await.unwrap();
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].version, "1.0.0");
        assert_eq!(result.bindings[0].cid, publish_resp.cid);
    }

    #[tokio::test]
    async fn test_namespace_local_only() {
        let dir = TempDir::new().unwrap();
        let cas_store = CasStore::new(dir.path().join("cas")).await.unwrap();
        let content_service = Arc::new(ContentService::new(cas_store));
        let name_service = Arc::new(NameService::new());

        let mut config = FederatedRegistryConfig::default();
        config.namespaces.push(NamespacePolicy {
            prefix: "@private".to_string(),
            delegate_to_upstream: false,
            local_only: true,
            allowed_upstreams: Vec::new(),
        });

        let registry = FederatedRegistry::new(config, name_service, content_service).unwrap();

        let result = registry.resolve("@private/pkg", &[]).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(matches!(res.source, LogSource::Local { .. }));
    }
}