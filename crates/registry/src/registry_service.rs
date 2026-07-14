use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use dash_proto::dpr_v1::{
    self, FetchResponse, InclusionProof, PublishResponse, ResolveResponse,
    dpr_service_server::DprService,
};
use dash_provenance::{Identity, KeylessSigner, ProvenanceService};

use crate::content_service::ContentService;
use crate::name_service::{NameService, VersionRange};

pub struct RegistryService<S: KeylessSigner> {
    name_service: Arc<NameService>,
    content_service: Arc<ContentService>,
    provenance_service: Arc<Mutex<ProvenanceService<S>>>,
}

impl<S: KeylessSigner> RegistryService<S> {
    pub fn new(
        name_service: Arc<NameService>,
        content_service: Arc<ContentService>,
        provenance_service: Arc<Mutex<ProvenanceService<S>>>,
    ) -> Self {
        Self { name_service, content_service, provenance_service }
    }
}

#[tonic::async_trait]
impl<S> DprService for RegistryService<S>
where
    S: KeylessSigner + Send + Sync + 'static,
{
    async fn resolve(
        &self,
        request: Request<dpr_v1::ResolveRequest>,
    ) -> Result<Response<ResolveResponse>, Status> {
        let req = request.into_inner();
        let ranges: Vec<VersionRange> =
            req.version_ranges.into_iter().map(VersionRange::from).collect();

        let bindings = self.name_service.resolve(&req.name, &ranges);
        let provenance_service = self.provenance_service.lock().await;

        let proto_bindings: Vec<dpr_v1::Binding> = bindings
            .into_iter()
            .map(|b| {
                let (inclusion_proof, provenance) = if let Some(ref proof) = b.inclusion_proof {
                    let checkpoint = provenance_service.latest_checkpoint();
                    let ip = InclusionProof {
                        lemma: proof.siblings.iter().map(hex::encode).collect(),
                        root_hash: hex::encode(checkpoint.root_hash),
                        tree_size: proof.tree_size,
                    };
                    let prov = b.provenance.as_ref().map(|att| dpr_v1::Provenance {
                        identity: att.subject.first().map(|s| s.name.clone()).unwrap_or_default(),
                        timestamp: 0,
                        builder_id: att
                            .predicate
                            .get("builder")
                            .and_then(|b| b.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        source_repository: String::new(),
                        source_commit: String::new(),
                        slsa_level: "3".to_string(),
                        metadata: None,
                    });
                    (Some(ip), prov)
                } else {
                    (None, None)
                };

                dpr_v1::Binding {
                    version: b.version,
                    cid: b.cid,
                    log_index: b.log_index,
                    inclusion_proof,
                    provenance,
                }
            })
            .collect();

        Ok(Response::new(ResolveResponse { bindings: proto_bindings }))
    }

    async fn fetch(
        &self,
        request: Request<dpr_v1::FetchRequest>,
    ) -> Result<Response<FetchResponse>, Status> {
        let req = request.into_inner();
        let cid_str = &req.cid;

        let content = self.content_service.fetch(cid_str).await.map_err(|e| {
            if matches!(e, crate::content_service::ContentServiceError::InvalidCid(_)) {
                Status::invalid_argument(format!("invalid CID: {e}"))
            } else {
                Status::not_found(format!("content not found: {e}"))
            }
        })?;

        let content_hash = blake3::hash(&content);
        let merkle_proof = dpr_v1::MerkleProof {
            root_hash: hex::encode(content_hash.as_bytes()),
            leaf_hash: hex::encode(content_hash.as_bytes()),
            siblings: vec![],
            leaf_index: 0,
            tree_size: 1,
        };

        Ok(Response::new(FetchResponse { content, merkle_proof: Some(merkle_proof) }))
    }

    async fn publish(
        &self,
        request: Request<dpr_v1::PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let req = request.into_inner();
        let capsule_bytes = Bytes::from(req.capsule);
        let proto_identity = req
            .identity
            .ok_or_else(|| Status::invalid_argument("identity is required for publish"))?;

        let name_for_registration = proto_identity.oidc_subject.clone();

        let identity = Identity {
            oidc_issuer: proto_identity.oidc_issuer,
            oidc_subject: proto_identity.oidc_subject.clone(),
            key_id: proto_identity.key_id,
        };

        let capsule = dash_capsule::Capsule::from_bytes(&capsule_bytes)
            .map_err(|e| Status::invalid_argument(format!("invalid capsule: {e}")))?;
        let cid = capsule.cid().to_string();

        self.content_service
            .store(&capsule_bytes)
            .await
            .map_err(|e| Status::internal(format!("failed to store capsule: {e}")))?;

        let mut prov_service = self.provenance_service.lock().await;
        let receipt = prov_service
            .publish(cid.clone(), identity)
            .map_err(|e| Status::internal(format!("failed to log publication: {e}")))?;

        let version = "0.1.0";

        self.name_service
            .register(
                &name_for_registration,
                version,
                &cid,
                receipt.log_index,
                Some(receipt.inclusion_proof.clone()),
                Some(receipt.attestation.clone()),
            )
            .map_err(|e| Status::already_exists(format!("failed to register name: {e}")))?;

        let checkpoint = prov_service.latest_checkpoint();
        let proto_proof = InclusionProof {
            lemma: receipt.inclusion_proof.siblings.iter().map(hex::encode).collect(),
            root_hash: hex::encode(checkpoint.root_hash),
            tree_size: receipt.inclusion_proof.tree_size,
        };

        Ok(Response::new(PublishResponse {
            cid,
            version: version.to_string(),
            log_index: receipt.log_index,
            inclusion_proof: Some(proto_proof),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_capsule::Capsule;
    use dash_provenance::DefaultKeylessSigner;
    use dash_store::cas::CasStore;

    struct TestServices {
        registry: RegistryService<DefaultKeylessSigner>,
        name_service: Arc<NameService>,
        _dir: tempfile::TempDir,
    }

    async fn test_services() -> TestServices {
        let dir = tempfile::tempdir().unwrap();
        let store_path = dir.path().to_path_buf();
        let cas_store = CasStore::new(store_path).await.unwrap();
        let content_service = Arc::new(ContentService::new(cas_store));
        let name_service = Arc::new(NameService::new());
        let signer = DefaultKeylessSigner::new();
        let provenance_service = Arc::new(Mutex::new(ProvenanceService::new(signer)));
        let registry =
            RegistryService::new(name_service.clone(), content_service, provenance_service);
        TestServices { registry, name_service, _dir: dir }
    }

    #[tokio::test]
    async fn test_resolve_no_bindings() {
        let svc = test_services().await;
        let req = dpr_v1::ResolveRequest {
            name: "nonexistent".to_string(),
            version_ranges: vec![],
            registry: String::new(),
        };
        let resp = svc.registry.resolve(Request::new(req)).await.unwrap();
        assert!(resp.into_inner().bindings.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_nonexistent() {
        let svc = test_services().await;
        let req = dpr_v1::FetchRequest { cid: "b3:nonexistent".to_string(), ranges: vec![] };
        let resp = svc.registry.fetch(Request::new(req)).await;
        assert!(resp.is_err());
    }

    #[tokio::test]
    async fn test_publish_and_resolve() {
        let svc = test_services().await;

        let files = vec![("test.txt".to_string(), b"hello publish".to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        let capsule_bytes = capsule.to_bytes().unwrap();
        let expected_cid = capsule.cid().to_string();

        let identity = dpr_v1::Identity {
            oidc_issuer: "https://github.com".to_string(),
            oidc_subject: "test-user".to_string(),
            key_id: "test-key".to_string(),
            public_key: vec![],
        };

        let req = dpr_v1::PublishRequest {
            capsule: capsule_bytes.to_vec(),
            attestations: vec![],
            signature: String::new(),
            identity: Some(identity),
        };

        let resp = svc.registry.publish(Request::new(req)).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.cid, expected_cid);
        assert!(inner.inclusion_proof.is_some());

        let bindings = svc.name_service.resolve("test-user", &[]);
        assert!(!bindings.is_empty());
        assert_eq!(bindings[0].cid, expected_cid);
    }

    #[tokio::test]
    async fn test_fetch_after_publish() {
        let svc = test_services().await;

        let files = vec![("hello.txt".to_string(), b"fetch me".to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        let capsule_bytes = capsule.to_bytes().unwrap();
        let cid_str = capsule.cid().to_string();

        let identity = dpr_v1::Identity {
            oidc_issuer: "https://example.com".to_string(),
            oidc_subject: "fetcher".to_string(),
            key_id: "key-1".to_string(),
            public_key: vec![],
        };

        let pub_req = dpr_v1::PublishRequest {
            capsule: capsule_bytes.to_vec(),
            attestations: vec![],
            signature: String::new(),
            identity: Some(identity),
        };
        svc.registry.publish(Request::new(pub_req)).await.unwrap();

        let fetch_req = dpr_v1::FetchRequest { cid: cid_str.clone(), ranges: vec![] };
        let resp = svc.registry.fetch(Request::new(fetch_req)).await.unwrap();
        let fetch_inner = resp.into_inner();
        assert!(!fetch_inner.content.is_empty());
    }

    #[tokio::test]
    async fn test_publish_missing_identity_rejected() {
        let svc = test_services().await;
        let capsule = Capsule::pack(vec![]).unwrap();
        let req = dpr_v1::PublishRequest {
            capsule: capsule.to_bytes().unwrap(),
            attestations: vec![],
            signature: String::new(),
            identity: None,
        };
        let resp = svc.registry.publish(Request::new(req)).await;
        assert!(resp.is_err());
    }
}
