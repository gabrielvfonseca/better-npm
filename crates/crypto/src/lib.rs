//! Cryptographic primitives and key management for Dash.
//!
//! Provides key generation, symmetric encryption/decryption,
//! and re-exports of common hash and signature algorithms.

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Error type for cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
    #[error("key generation failed: {0}")]
    KeyGeneration(String),
    #[error("invalid key length: {0}")]
    InvalidKeyLength(String),
    #[error("invalid nonce: {0}")]
    InvalidNonce(String),
}

/// An asymmetric key pair (Ed25519).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keypair {
    /// The public key bytes.
    pub public: Vec<u8>,
    /// The private (secret) key bytes.
    pub private: Vec<u8>,
}

impl Keypair {
    /// Create a keypair from raw public and private bytes.
    pub fn from_bytes(public: Vec<u8>, private: Vec<u8>) -> Self {
        Self { public, private }
    }

    /// Get the public key as a hex string.
    pub fn public_hex(&self) -> String {
        hex::encode(&self.public)
    }

    /// Get the private key as a hex string.
    pub fn private_hex(&self) -> String {
        hex::encode(&self.private)
    }
}

impl Zeroize for Keypair {
    fn zeroize(&mut self) {
        self.private.zeroize();
        self.public.zeroize();
    }
}

impl Drop for Keypair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Generate a new Ed25519 key pair.
pub fn generate_keypair() -> Result<Keypair, CryptoError> {
    use ed25519_dalek::SigningKey;
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();

    Ok(Keypair {
        public: verifying_key.to_bytes().to_vec(),
        private: signing_key.to_keypair_bytes().to_vec(),
    })
}

/// Encrypt data using AES-256-GCM.
///
/// Returns the ciphertext prefixed with a 12-byte nonce.
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKeyLength(format!(
            "expected 32 bytes, got {}",
            key.len()
        )));
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data that was encrypted with [`encrypt`].
///
/// Expects the first 12 bytes to be the nonce.
pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKeyLength(format!(
            "expected 32 bytes, got {}",
            key.len()
        )));
    }

    if data.len() < 12 {
        return Err(CryptoError::InvalidNonce("data too short".to_string()));
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Decryption(e.to_string()))?;

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::Decryption(e.to_string()))
}

/// Generate `len` cryptographically secure random bytes.
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    OsRng.fill_bytes(&mut buf);
    buf
}

/// Re-export common hash algorithms.
pub mod digest {
    pub use sha2::{Digest, Sha256, Sha512};
    pub use sha3::{Digest as Sha3Digest, Sha3_256, Sha3_512};
}

/// Supported symmetric encryption algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

/// Supported asymmetric algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsymmetricAlgorithm {
    Ed25519,
    Rsa4096,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let kp = generate_keypair().unwrap();
        assert_eq!(kp.public.len(), 32);
        assert_eq!(kp.private.len(), 64);
        assert!(!kp.public_hex().is_empty());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = random_bytes(32);
        let data = b"secret message";
        let encrypted = encrypt(data, &key).unwrap();
        assert_ne!(encrypted, data);
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_encrypt_wrong_key() {
        let key1 = random_bytes(32);
        let key2 = random_bytes(32);
        let data = b"test";
        let encrypted = encrypt(data, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_random_bytes() {
        let a = random_bytes(16);
        let b = random_bytes(16);
        assert_eq!(a.len(), 16);
        assert_eq!(b.len(), 16);
        assert_ne!(a, b);
    }

    #[test]
    fn test_invalid_key_length() {
        let key = random_bytes(16);
        let result = encrypt(b"data", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_keypair_zeroize() {
        let mut kp = generate_keypair().unwrap();
        kp.zeroize();
        assert!(kp.public.iter().all(|&b| b == 0));
        assert!(kp.private.iter().all(|&b| b == 0));
    }
}
