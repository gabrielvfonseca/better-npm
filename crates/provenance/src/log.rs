use serde::{Deserialize, Serialize};

use crate::error::ProvenanceError;
use crate::merkle::{InclusionProof, MerkleTree, verify_inclusion as merkle_verify};
use crate::witness::Checkpoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub index: u64,
    pub cid: String,
    pub identity: String,
    pub timestamp: i64,
    pub data_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub key_id: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCheckpoint {
    pub root_hash: [u8; 32],
    pub tree_size: u64,
    pub timestamp: i64,
    pub signatures: Vec<Signature>,
}

impl SignedCheckpoint {
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            root_hash: self.root_hash,
            tree_size: self.tree_size,
            timestamp: self.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogState {
    pub root_hash: [u8; 32],
    pub tree_size: u64,
    pub checkpoint: SignedCheckpoint,
}

pub trait LogService: Send + Sync {
    fn append(&mut self, entry: LogEntry) -> Result<(u64, InclusionProof), ProvenanceError>;
    fn lookup(&self, index: u64) -> Option<LogEntry>;
    fn latest_checkpoint(&self) -> SignedCheckpoint;
    fn verify_inclusion(
        proof: &InclusionProof,
        entry_hash: &[u8; 32],
        checkpoint: &SignedCheckpoint,
    ) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryLogService {
    tree: MerkleTree,
    entries: Vec<LogEntry>,
    signatures: Vec<Signature>,
}

impl InMemoryLogService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_signature(&mut self, signature: Signature) {
        self.signatures.push(signature);
    }

    fn entry_hash(entry: &LogEntry) -> [u8; 32] {
        entry.data_hash
    }

    pub fn merkle_proof(&self, leaf_index: u64) -> InclusionProof {
        self.tree.inclusion_proof(leaf_index)
    }
}

impl LogService for InMemoryLogService {
    fn append(&mut self, entry: LogEntry) -> Result<(u64, InclusionProof), ProvenanceError> {
        let index = self.entries.len() as u64;
        let entry_hash = Self::entry_hash(&entry);
        let tree_index = self.tree.append(entry_hash);
        debug_assert_eq!(index, tree_index);

        let proof = self.tree.inclusion_proof(index);
        self.entries.push(entry);

        Ok((index, proof))
    }

    fn lookup(&self, index: u64) -> Option<LogEntry> {
        self.entries.get(index as usize).cloned()
    }

    fn latest_checkpoint(&self) -> SignedCheckpoint {
        SignedCheckpoint {
            root_hash: self.tree.root_hash(),
            tree_size: self.tree.size(),
            timestamp: chrono_now(),
            signatures: self.signatures.clone(),
        }
    }

    fn verify_inclusion(
        proof: &InclusionProof,
        entry_hash: &[u8; 32],
        checkpoint: &SignedCheckpoint,
    ) -> bool {
        merkle_verify(proof, entry_hash, &checkpoint.root_hash)
    }
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(index: u64, data: &[u8]) -> LogEntry {
        LogEntry {
            index,
            cid: format!("b3:{}", hex::encode(data)),
            identity: "test-identity".to_string(),
            timestamp: 1000,
            data_hash: blake3::hash(data).into(),
        }
    }

    #[test]
    fn test_log_append_and_lookup() {
        let mut svc = InMemoryLogService::new();
        let entry = make_entry(0, b"hello");
        let (idx, proof) = svc.append(entry.clone()).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(proof.leaf_index, 0);

        let looked_up = svc.lookup(0).unwrap();
        assert_eq!(looked_up.cid, entry.cid);
        assert_eq!(looked_up.data_hash, entry.data_hash);
    }

    #[test]
    fn test_log_lookup_missing() {
        let svc = InMemoryLogService::new();
        assert!(svc.lookup(0).is_none());
    }

    #[test]
    fn test_log_checkpoint_after_append() {
        let mut svc = InMemoryLogService::new();
        let cp0 = svc.latest_checkpoint();
        assert_eq!(cp0.tree_size, 0);

        svc.append(make_entry(0, b"entry1")).unwrap();
        let cp1 = svc.latest_checkpoint();
        assert_eq!(cp1.tree_size, 1);

        svc.append(make_entry(1, b"entry2")).unwrap();
        let cp2 = svc.latest_checkpoint();
        assert_eq!(cp2.tree_size, 2);
        assert_ne!(cp1.root_hash, cp2.root_hash);
    }

    #[test]
    fn test_log_inclusion_proof_verify() {
        let mut svc = InMemoryLogService::new();
        let entries: Vec<LogEntry> = (0..10).map(|i| make_entry(i as u64, &[i])).collect();
        let mut proofs_and_cps = Vec::new();
        for entry in &entries {
            let (idx, proof) = svc.append(entry.clone()).unwrap();
            let cp = svc.latest_checkpoint();
            proofs_and_cps.push((idx, proof, cp));
        }
        for (i, entry) in entries.iter().enumerate() {
            let (_, ref proof, ref cp) = proofs_and_cps[i];
            let valid = InMemoryLogService::verify_inclusion(proof, &entry.data_hash, cp);
            assert!(valid, "inclusion proof failed for index {i}");
        }
    }

    #[test]
    fn test_log_inclusion_proof_verify_final_root() {
        let mut svc = InMemoryLogService::new();
        let entries: Vec<LogEntry> = (0..10).map(|i| make_entry(i as u64, &[i])).collect();
        for entry in &entries {
            svc.append(entry.clone()).unwrap();
        }
        let final_cp = svc.latest_checkpoint();
        let proof = svc.merkle_proof(0);
        let valid = InMemoryLogService::verify_inclusion(&proof, &entries[0].data_hash, &final_cp);
        assert!(valid, "final root proof failed");
    }

    #[test]
    fn test_log_inclusion_proof_tampered_checkpoint() {
        let mut svc = InMemoryLogService::new();
        let entry = make_entry(0, b"data");
        let (_idx, proof) = svc.append(entry.clone()).unwrap();
        let mut bad_checkpoint = svc.latest_checkpoint();
        bad_checkpoint.root_hash[0] ^= 0xff;
        assert!(!InMemoryLogService::verify_inclusion(&proof, &entry.data_hash, &bad_checkpoint,));
    }
}
