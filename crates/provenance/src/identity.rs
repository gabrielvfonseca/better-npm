use ed25519_dalek::{Signer, Verifier};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::ProvenanceError;
use crate::log::Signature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub oidc_issuer: String,
    pub oidc_subject: String,
    pub key_id: String,
}

pub trait KeylessSigner: Send + Sync {
    fn sign(&self, identity: &Identity, payload: &[u8]) -> Result<Signature, ProvenanceError>;
    fn verify(
        &self,
        identity: &Identity,
        payload: &[u8],
        signature: &Signature,
    ) -> Result<bool, ProvenanceError>;
}

pub struct DefaultKeylessSigner {
    signing_key: ed25519_dalek::SigningKey,
}

impl Default for DefaultKeylessSigner {
    fn default() -> Self {
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret);
        Self { signing_key }
    }
}

impl DefaultKeylessSigner {
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

impl KeylessSigner for DefaultKeylessSigner {
    fn sign(&self, identity: &Identity, payload: &[u8]) -> Result<Signature, ProvenanceError> {
        let sig = self
            .signing_key
            .try_sign(payload)
            .map_err(|e| ProvenanceError::Signing(e.to_string()))?;
        Ok(Signature { key_id: identity.key_id.clone(), bytes: sig.to_bytes().to_vec() })
    }

    fn verify(
        &self,
        identity: &Identity,
        payload: &[u8],
        signature: &Signature,
    ) -> Result<bool, ProvenanceError> {
        let pub_key_bytes = hex::decode(&identity.key_id)
            .map_err(|e| ProvenanceError::Verification(e.to_string()))?;
        let pub_key_arr: [u8; 32] = pub_key_bytes
            .try_into()
            .map_err(|_| ProvenanceError::Verification("invalid public key length".into()))?;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pub_key_arr)
            .map_err(|e| ProvenanceError::Verification(e.to_string()))?;
        let sig_bytes: [u8; 64] = signature.bytes[..]
            .try_into()
            .map_err(|_| ProvenanceError::Verification("invalid signature length".into()))?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        Ok(verifying_key.verify(payload, &sig).is_ok())
    }
}

pub fn verify_oidc_token(token: &str) -> Result<Identity, ProvenanceError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(ProvenanceError::OidcVerification(
            "invalid JWT format: expected 3 dot-separated parts".into(),
        ));
    }
    let claims_part = parts
        .get(1)
        .ok_or_else(|| ProvenanceError::OidcVerification("missing claims part".into()))?;
    let decoded = base64_decode(claims_part)?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded)?;
    let issuer = claims
        .get("iss")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProvenanceError::OidcVerification("missing 'iss' claim".into()))?;
    let subject = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProvenanceError::OidcVerification("missing 'sub' claim".into()))?;
    let key_id = claims.get("azp").and_then(|v| v.as_str()).unwrap_or(subject).to_string();
    Ok(Identity { oidc_issuer: issuer.to_string(), oidc_subject: subject.to_string(), key_id })
}

fn base64_decode(input: &str) -> Result<Vec<u8>, ProvenanceError> {
    use base64::Engine;
    let padded = match input.len() % 4 {
        2 => format!("{input}=="),
        3 => format!("{input}="),
        _ => input.to_string(),
    };
    base64::engine::general_purpose::URL_SAFE
        .decode(&padded)
        .map_err(|e| ProvenanceError::OidcVerification(e.to_string()))
}
