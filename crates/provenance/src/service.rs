use serde::{Deserialize, Serialize};

use crate::attestation::{Attestation, build_attestation, verify_attestation};
use crate::error::ProvenanceError;
use crate::identity::{Identity, KeylessSigner};
use crate::log::{InMemoryLogService, LogEntry, LogService, SignedCheckpoint};
use crate::merkle::{InclusionProof, verify_inclusion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub log_index: u64,
    pub inclusion_proof: InclusionProof,
    pub attestation: Attestation,
}

pub struct ProvenanceService<S: KeylessSigner> {
    log: InMemoryLogService,
    signer: S,
}

impl<S: KeylessSigner> ProvenanceService<S> {
    pub fn new(signer: S) -> Self {
        Self { log: InMemoryLogService::new(), signer }
    }

    pub fn publish(
        &mut self,
        cid: String,
        identity: Identity,
    ) -> Result<PublishReceipt, ProvenanceError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let data_hash = blake3::hash(cid.as_bytes()).into();

        let entry = LogEntry {
            index: 0,
            cid: cid.clone(),
            identity: identity.oidc_subject.clone(),
            timestamp,
            data_hash,
        };

        let _ = self.signer.sign(&identity, &data_hash)?;
        let (log_index, inclusion_proof) = self.log.append(entry)?;

        let attestation = build_attestation(&identity.oidc_subject, "1.0.0", &cid, "bnpm:registry");

        Ok(PublishReceipt { log_index, inclusion_proof, attestation })
    }

    pub fn verify(&self, entry_cid: &str, receipt: &PublishReceipt) -> bool {
        let entry_hash: [u8; 32] = blake3::hash(entry_cid.as_bytes()).into();
        let checkpoint = self.log.latest_checkpoint();
        let proof_valid =
            verify_inclusion(&receipt.inclusion_proof, &entry_hash, &checkpoint.root_hash);
        if !proof_valid {
            return false;
        }
        let entry = self.log.lookup(receipt.log_index);
        match entry {
            Some(log_entry) if log_entry.cid == entry_cid => {},
            _ => return false,
        }
        verify_attestation(&receipt.attestation)
    }

    pub fn latest_checkpoint(&self) -> SignedCheckpoint {
        self.log.latest_checkpoint()
    }

    pub fn lookup(&self, index: u64) -> Option<LogEntry> {
        self.log.lookup(index)
    }
}
