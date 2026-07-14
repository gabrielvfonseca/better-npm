use ed25519_dalek::{Signer, Verifier};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::ProvenanceError;
use crate::log::{Signature, SignedCheckpoint};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub root_hash: [u8; 32],
    pub tree_size: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct PublicKey {
    pub key_id: String,
    pub bytes: Vec<u8>,
}

pub trait WitnessSigner: Send + Sync {
    fn sign(&self, data: &[u8]) -> Result<Signature, ProvenanceError>;
    fn key_id(&self) -> String;
}

pub struct Ed25519WitnessSigner {
    signing_key: ed25519_dalek::SigningKey,
}

impl Default for Ed25519WitnessSigner {
    fn default() -> Self {
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret);
        Self { signing_key }
    }
}

impl Ed25519WitnessSigner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_signing_key(signing_key: ed25519_dalek::SigningKey) -> Self {
        Self { signing_key }
    }

    pub fn verifying_key(&self) -> ed25519_dalek::VerifyingKey {
        self.signing_key.verifying_key()
    }
}

impl WitnessSigner for Ed25519WitnessSigner {
    fn sign(&self, data: &[u8]) -> Result<Signature, ProvenanceError> {
        let sig = self
            .signing_key
            .try_sign(data)
            .map_err(|e| ProvenanceError::Signing(e.to_string()))?;
        Ok(Signature { key_id: self.key_id(), bytes: sig.to_bytes().to_vec() })
    }

    fn key_id(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }
}

fn serialize_checkpoint(checkpoint: &Checkpoint) -> Vec<u8> {
    let s = format!(
        "dash checkpoint\nroot: {}\nsize: {}\ntime: {}\n",
        hex::encode(checkpoint.root_hash),
        checkpoint.tree_size,
        checkpoint.timestamp,
    );
    s.into_bytes()
}

pub fn cosign_checkpoint(
    checkpoint: &Checkpoint,
    signer: &dyn WitnessSigner,
) -> Result<Signature, ProvenanceError> {
    let data = serialize_checkpoint(checkpoint);
    signer.sign(&data)
}

pub fn verify_checkpoint(
    checkpoint: &Checkpoint,
    signatures: &[Signature],
    trusted_keys: &[PublicKey],
) -> bool {
    if signatures.is_empty() {
        return false;
    }
    let data = serialize_checkpoint(checkpoint);
    for sig in signatures {
        let pk = match trusted_keys.iter().find(|k| k.key_id == sig.key_id) {
            Some(pk) => pk,
            None => return false,
        };
        let pub_key_arr: [u8; 32] = match pk.bytes[..].try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&pub_key_arr) {
            Ok(vk) => vk,
            Err(_) => return false,
        };
        let sig_bytes: [u8; 64] = match sig.bytes[..].try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        if verifying_key.verify(&data, &sig).is_err() {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone)]
pub struct SplitViewDetected {
    pub message: String,
}

pub fn detect_split_view(
    checkpoints_from_multiple_witnesses: Vec<(String, SignedCheckpoint)>,
) -> Result<(), SplitViewDetected> {
    for i in 0..checkpoints_from_multiple_witnesses.len() {
        for j in (i + 1)..checkpoints_from_multiple_witnesses.len() {
            let (witness_a, cp_a) = &checkpoints_from_multiple_witnesses[i];
            let (witness_b, cp_b) = &checkpoints_from_multiple_witnesses[j];
            let a_min = cp_a.tree_size.saturating_sub(1);
            let b_min = cp_b.tree_size.saturating_sub(1);
            let overlap_start = a_min.max(b_min);
            let overlap_end = cp_a.tree_size.min(cp_b.tree_size);
            if overlap_start < overlap_end && cp_a.root_hash != cp_b.root_hash {
                return Err(SplitViewDetected {
                    message: format!(
                        "split-view between witnesses '{}' and '{}': \
                         root hashes differ for overlapping tree size range [{}, {})",
                        witness_a, witness_b, overlap_start, overlap_end,
                    ),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::SignedCheckpoint;

    fn make_test_signer() -> Ed25519WitnessSigner {
        Ed25519WitnessSigner::new()
    }

    fn make_checkpoint(root: &[u8; 32], size: u64, time: i64) -> Checkpoint {
        Checkpoint { root_hash: *root, tree_size: size, timestamp: time }
    }

    #[test]
    fn test_cosign_and_verify() {
        let signer = make_test_signer();
        let root = blake3::hash(b"test-root").into();
        let cp = make_checkpoint(&root, 10, 1000);
        let sig = cosign_checkpoint(&cp, &signer).unwrap();
        assert_eq!(sig.key_id, signer.key_id());
        assert_eq!(sig.bytes.len(), 64);

        let pub_key = PublicKey {
            key_id: signer.key_id(),
            bytes: signer.verifying_key().to_bytes().to_vec(),
        };
        assert!(verify_checkpoint(&cp, &[sig], &[pub_key]));
    }

    #[test]
    fn test_verify_wrong_key() {
        let signer = make_test_signer();
        let wrong = make_test_signer();
        let root = blake3::hash(b"other-root").into();
        let cp = make_checkpoint(&root, 5, 2000);
        let sig = cosign_checkpoint(&cp, &signer).unwrap();

        let wrong_key =
            PublicKey { key_id: wrong.key_id(), bytes: wrong.verifying_key().to_bytes().to_vec() };
        assert!(!verify_checkpoint(&cp, &[sig], &[wrong_key]));
    }

    #[test]
    fn test_verify_no_signatures() {
        let signer = make_test_signer();
        let root = blake3::hash(b"root").into();
        let cp = make_checkpoint(&root, 1, 500);
        let pk = PublicKey {
            key_id: signer.key_id(),
            bytes: signer.verifying_key().to_bytes().to_vec(),
        };
        assert!(!verify_checkpoint(&cp, &[], &[pk]));
    }

    #[test]
    fn test_split_view_agreeing_passes() {
        let root = blake3::hash(b"same-root").into();
        let cp1 =
            SignedCheckpoint { root_hash: root, tree_size: 10, timestamp: 100, signatures: vec![] };
        let cp2 =
            SignedCheckpoint { root_hash: root, tree_size: 10, timestamp: 101, signatures: vec![] };
        let witnesses = vec![("witness-a".to_string(), cp1), ("witness-b".to_string(), cp2)];
        assert!(detect_split_view(witnesses).is_ok());
    }

    #[test]
    fn test_split_view_disagreeing_fails() {
        let root_a = blake3::hash(b"root-a").into();
        let root_b = blake3::hash(b"root-b").into();
        let cp1 = SignedCheckpoint {
            root_hash: root_a,
            tree_size: 10,
            timestamp: 100,
            signatures: vec![],
        };
        let cp2 = SignedCheckpoint {
            root_hash: root_b,
            tree_size: 10,
            timestamp: 101,
            signatures: vec![],
        };
        let witnesses = vec![("witness-a".to_string(), cp1), ("witness-b".to_string(), cp2)];
        let result = detect_split_view(witnesses);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("split-view"));
    }

    #[test]
    fn test_split_view_non_overlapping_ok() {
        let root_a = blake3::hash(b"root-a").into();
        let root_b = blake3::hash(b"root-b").into();
        let cp1 = SignedCheckpoint {
            root_hash: root_a,
            tree_size: 10,
            timestamp: 100,
            signatures: vec![],
        };
        let cp2 = SignedCheckpoint {
            root_hash: root_b,
            tree_size: 20,
            timestamp: 101,
            signatures: vec![],
        };
        let witnesses = vec![("witness-a".to_string(), cp1), ("witness-b".to_string(), cp2)];
        assert!(detect_split_view(witnesses).is_ok());
    }
}
