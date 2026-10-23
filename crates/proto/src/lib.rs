// bnpm-proto: DPR wire protocol client and server

use bytes::Bytes;
use thiserror::Error;

// Create the expected module hierarchy matching protobuf package structure:
// bnpm.common.v1  -> common types
// bnpm.dpr.v1     -> DPR protocol types
pub mod bnpm {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bnpm.common.v1.rs"));
        }
    }
    pub mod dpr {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bnpm.dpr.v1.rs"));
        }
    }
}

// Re-export at crate root for convenience
pub use bnpm::common::v1 as common_v1;
pub use bnpm::dpr::v1 as dpr_v1;

#[allow(unused_imports)]
use bnpm::dpr::v1::{
    Attestation, Binding, FetchRequest, FetchResponse, Identity, InclusionProof, Provenance,
    PublishRequest, PublishResponse, ResolveRequest, ResolveResponse, VersionRange,
};
#[allow(unused_imports)]
use tonic::{Request, Response, Status};

#[derive(Error, Debug)]
#[allow(clippy::result_large_err)]
pub enum DprError {
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("gRPC status: {0}")]
    Grpc(#[from] Status),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("invalid CID: {0}")]
    InvalidCid(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("merkle proof verification failed: {0}")]
    MerkleProofFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("rate limited: {0}")]
    RateLimited(String),
    #[error("publish rejected: {0}")]
    PublishRejected(String),
    #[error("unimplemented: {0}")]
    Unimplemented(String),
}

impl From<DprError> for Status {
    fn from(e: DprError) -> Self {
        match e {
            DprError::Transport(e) => Status::unavailable(e.to_string()),
            DprError::Grpc(s) => s,
            DprError::Serialization(s) => Status::internal(s),
            DprError::InvalidArgument(s) => Status::invalid_argument(s),
            DprError::InvalidCid(s) => Status::invalid_argument(s),
            DprError::InvalidResponse(s) => Status::internal(s),
            DprError::MerkleProofFailed(s) => Status::internal(s),
            DprError::NotFound(s) => Status::not_found(s),
            DprError::Unauthorized(s) => Status::unauthenticated(s),
            DprError::RateLimited(s) => Status::resource_exhausted(s),
            DprError::PublishRejected(s) => Status::permission_denied(s),
            DprError::Unimplemented(s) => Status::unimplemented(s),
        }
    }
}

/// DPR Client for communicating with the registry
#[derive(Clone)]
pub struct DprClient {
    inner: bnpm::dpr::v1::dpr_service_client::DprServiceClient<tonic::transport::Channel>,
}

impl DprClient {
    /// Connect to a DPR endpoint
    pub async fn connect(uri: &str) -> Result<Self, DprError> {
        let channel = tonic::transport::Channel::from_shared(uri.to_string())
            .map_err(|e| DprError::InvalidArgument(e.to_string()))?
            .connect()
            .await
            .map_err(DprError::Transport)?;
        Ok(Self { inner: bnpm::dpr::v1::dpr_service_client::DprServiceClient::new(channel) })
    }

    /// Resolve a package name with version ranges
    pub async fn resolve(
        &mut self,
        name: String,
        version_ranges: Vec<VersionRange>,
        registry: String,
    ) -> Result<Vec<Binding>, DprError> {
        let req = Request::new(ResolveRequest { name, version_ranges, registry });
        let resp = self.inner.resolve(req).await?;
        Ok(resp.into_inner().bindings)
    }

    /// Fetch content by CID with optional range requests
    pub async fn fetch(
        &mut self,
        cid: String,
        ranges: Vec<bnpm::dpr::v1::Range>,
    ) -> Result<FetchResponse, DprError> {
        let req = Request::new(FetchRequest { cid, ranges });
        let resp = self.inner.fetch(req).await?;
        Ok(resp.into_inner())
    }

    /// Publish a capsule with attestations
    pub async fn publish(
        &mut self,
        capsule: Bytes,
        attestations: Vec<Attestation>,
        signature: String,
        identity: Identity,
    ) -> Result<PublishResponse, DprError> {
        let req = Request::new(PublishRequest {
            capsule: capsule.to_vec(),
            attestations,
            signature,
            identity: Some(identity),
        });
        let resp = self.inner.publish(req).await?;
        Ok(resp.into_inner())
    }
}

/// Run the DPR gRPC server with the given implementation
#[allow(clippy::result_large_err)]
pub async fn run_server<D>(addr: std::net::SocketAddr, service: D) -> Result<(), DprError>
where
    D: bnpm::dpr::v1::dpr_service_server::DprService + Send + Sync + 'static,
{
    let svc = bnpm::dpr::v1::dpr_service_server::DprServiceServer::new(service);
    tonic::transport::Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

/// Default server implementation (to be replaced by control-plane services)
#[derive(Default, Clone)]
pub struct DefaultDprServer;

#[tonic::async_trait]
impl bnpm::dpr::v1::dpr_service_server::DprService for DefaultDprServer {
    async fn resolve(
        &self,
        _request: Request<ResolveRequest>,
    ) -> Result<Response<ResolveResponse>, Status> {
        Err(Status::unimplemented("resolve not implemented in this crate"))
    }

    async fn fetch(
        &self,
        _request: Request<FetchRequest>,
    ) -> Result<Response<FetchResponse>, Status> {
        Err(Status::unimplemented("fetch not implemented in this crate"))
    }

    async fn publish(
        &self,
        _request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        Err(Status::unimplemented("publish not implemented in this crate"))
    }
}

/// Run the DPR gRPC server with the default implementation
#[allow(clippy::result_large_err)]
pub async fn run_default_server(addr: std::net::SocketAddr) -> Result<(), DprError> {
    let svc = bnpm::dpr::v1::dpr_service_server::DprServiceServer::new(DefaultDprServer);
    tonic::transport::Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn test_dpr_error_display() {
        let err = DprError::InvalidArgument("test".into());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_protobuf_roundtrip() {
        let req = ResolveRequest {
            name: "test-package".to_string(),
            version_ranges: vec![],
            registry: "https://registry.example.com".to_string(),
        };
        let mut buf = Vec::new();
        req.encode(&mut buf).unwrap();
        let decoded = ResolveRequest::decode(&buf[..]).unwrap();
        assert_eq!(decoded.name, req.name);
    }

    #[test]
    fn test_binding_serialization() {
        let binding = Binding {
            version: "1.0.0".to_string(),
            cid: "b3:test".to_string(),
            log_index: 123,
            inclusion_proof: Some(InclusionProof {
                lemma: vec!["hash1".to_string(), "hash2".to_string()],
                root_hash: "root".to_string(),
                tree_size: 10,
            }),
            provenance: Some(Provenance {
                identity: "did:bnpm:test".to_string(),
                timestamp: 1234567890,
                builder_id: "builder".to_string(),
                source_repository: "https://github.com/test/repo".to_string(),
                source_commit: "abc123".to_string(),
                slsa_level: "3".to_string(),
                metadata: None,
            }),
        };
        let mut buf = Vec::new();
        binding.encode(&mut buf).unwrap();
        let decoded = Binding::decode(&buf[..]).unwrap();
        assert_eq!(decoded.version, binding.version);
        assert_eq!(decoded.cid, binding.cid);
    }
}
