pub mod attestation;
pub mod error;
pub mod identity;
pub mod log;
pub mod merkle;
pub mod service;
pub mod witness;

pub use attestation::{
    Attestation, SlsaMetadata, SlsaPredicate, Subject, build_attestation, verify_attestation,
};
pub use error::ProvenanceError;
pub use identity::{DefaultKeylessSigner, Identity, KeylessSigner};
pub use log::{InMemoryLogService, LogEntry, LogService, LogState, SignedCheckpoint};
pub use merkle::{InclusionProof, MerkleTree, verify_inclusion};
pub use service::{ProvenanceService, PublishReceipt};
pub use witness::{
    Checkpoint, SplitViewDetected, WitnessSigner, cosign_checkpoint, detect_split_view,
    verify_checkpoint,
};
