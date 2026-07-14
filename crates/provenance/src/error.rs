use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProvenanceError {
    #[error("OIDC token verification failed: {0}")]
    OidcVerification(String),
    #[error("signing failed: {0}")]
    Signing(String),
    #[error("verification failed: {0}")]
    Verification(String),
    #[error("invalid inclusion proof")]
    InvalidInclusionProof,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("split-view detected: {0}")]
    SplitView(String),
    #[error("attestation error: {0}")]
    Attestation(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for ProvenanceError {
    fn from(e: serde_json::Error) -> Self {
        ProvenanceError::Serialization(e.to_string())
    }
}
