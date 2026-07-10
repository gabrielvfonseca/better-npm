//! Unified hashing interface supporting SHA-2, SHA-3, and BLAKE3.
//!
//! Provides a consistent API for computing hashes across multiple
//! algorithms, with helpers for files, streams, and hex encoding.

use sha2::{Digest, Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};
use std::io::Read;
use std::path::Path;

/// Error type for hashing operations.
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
}

/// Compute a SHA-256 hash over the given bytes.
pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Compute a SHA-512 hash over the given bytes.
pub fn hash_sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 64];
    out.copy_from_slice(&result);
    out
}

/// Compute a SHA3-256 hash over the given bytes.
pub fn hash_sha3_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Compute a SHA3-512 hash over the given bytes.
pub fn hash_sha3_512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha3_512::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 64];
    out.copy_from_slice(&result);
    out
}

/// Compute a BLAKE3 hash over the given bytes.
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    let hash = blake3::hash(data);
    *hash.as_bytes()
}

/// Compute the SHA-256 hash of a file's contents.
pub fn hash_file(path: &Path) -> Result<[u8; 32], HashError> {
    use std::fs::File;
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    Ok(out)
}

/// Compute the SHA-256 hash of a byte stream.
pub fn hash_stream(mut reader: impl Read) -> Result<[u8; 32], HashError> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    Ok(out)
}

/// Encode bytes as a lowercase hex string.
pub fn to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Decode a hex string into bytes.
pub fn from_hex(s: &str) -> Result<Vec<u8>, HashError> {
    Ok(hex::decode(s)?)
}

/// Encode bytes as a base64 string (standard, no padding).
pub fn to_base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}

/// Decode a base64 string into bytes.
pub fn from_base64(s: &str) -> Result<Vec<u8>, HashError> {
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.decode(s)?)
}

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
    Blake3,
}

/// Hash the given data with the specified algorithm.
pub fn hash(data: &[u8], algo: HashAlgorithm) -> Vec<u8> {
    match algo {
        HashAlgorithm::Sha256 => hash_sha256(data).to_vec(),
        HashAlgorithm::Sha512 => hash_sha512(data).to_vec(),
        HashAlgorithm::Sha3_256 => hash_sha3_256(data).to_vec(),
        HashAlgorithm::Sha3_512 => hash_sha3_512(data).to_vec(),
        HashAlgorithm::Blake3 => hash_blake3(data).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let result = hash_sha256(b"hello world");
        assert_eq!(result.len(), 32);
        let hex = to_hex(&result);
        assert_eq!(hex.len(), 64);
    }

    #[test]
    fn test_hash_sha512() {
        let result = hash_sha512(b"hello world");
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_hash_blake3() {
        let result = hash_blake3(b"hello world");
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_hash_sha3_256() {
        let result = hash_sha3_256(b"hello world");
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = b"hello";
        let hex = to_hex(data);
        let decoded = from_hex(&hex).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hash_file() {
        let path = std::path::Path::new("Cargo.toml");
        let result = hash_file(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_hash_stream() {
        let data = b"streaming test data";
        let result = hash_stream(&data[..]).unwrap();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"base64 test";
        let encoded = to_base64(data);
        let decoded = from_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hash_algorithm() {
        let data = b"test";
        let a = hash(data, HashAlgorithm::Sha256);
        let b = hash(data, HashAlgorithm::Blake3);
        assert_eq!(a.len(), 32);
        assert_eq!(b.len(), 32);
        assert_ne!(a, b);
    }
}
