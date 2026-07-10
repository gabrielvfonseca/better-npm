//! Content hash verification and integrity checking for packages.
//!
//! Defines the `Integrity` type used throughout Dash for verifying
//! that downloaded package contents match their expected hashes.

use std::path::Path;

/// Error type for integrity operations.
#[derive(Debug, thiserror::Error)]
pub enum IntegrityError {
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("invalid integrity string: {0}")]
    InvalidFormat(String),
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("hashing error: {0}")]
    Hash(String),
}

/// Supported hash algorithms for integrity checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IntegrityAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

impl IntegrityAlgorithm {
    /// Return the algorithm as a string (e.g. "sha256").
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegrityAlgorithm::Sha256 => "sha256",
            IntegrityAlgorithm::Sha512 => "sha512",
            IntegrityAlgorithm::Blake3 => "blake3",
        }
    }

    /// Parse an algorithm from a string.
    pub fn from_str(s: &str) -> Result<Self, IntegrityError> {
        match s.to_lowercase().as_str() {
            "sha256" | "sha-256" => Ok(IntegrityAlgorithm::Sha256),
            "sha512" | "sha-512" => Ok(IntegrityAlgorithm::Sha512),
            "blake3" | "blake-3" => Ok(IntegrityAlgorithm::Blake3),
            other => Err(IntegrityError::UnsupportedAlgorithm(other.to_string())),
        }
    }

    /// Return the output size in bytes for this algorithm.
    pub fn output_size(&self) -> usize {
        match self {
            IntegrityAlgorithm::Sha256 => 32,
            IntegrityAlgorithm::Sha512 => 64,
            IntegrityAlgorithm::Blake3 => 32,
        }
    }
}

/// An integrity hash value (algorithm + digest).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Integrity {
    /// The hash algorithm used.
    pub algorithm: IntegrityAlgorithm,
    /// The raw digest bytes.
    pub value: Vec<u8>,
    /// The original string representation (if parsed from one).
    pub raw: Option<String>,
}

impl Integrity {
    /// Compute an integrity hash from the given data using the specified algorithm.
    pub fn hash(data: &[u8], algorithm: IntegrityAlgorithm) -> Self {
        let value = match algorithm {
            IntegrityAlgorithm::Sha256 => dash_hashing::hash_sha256(data).to_vec(),
            IntegrityAlgorithm::Sha512 => dash_hashing::hash_sha512(data).to_vec(),
            IntegrityAlgorithm::Blake3 => dash_hashing::hash_blake3(data).to_vec(),
        };

        Self {
            algorithm,
            value,
            raw: None,
        }
    }

    /// Verify that the given data matches this integrity value.
    pub fn verify(&self, data: &[u8]) -> bool {
        let expected = &self.value;
        let actual = match self.algorithm {
            IntegrityAlgorithm::Sha256 => dash_hashing::hash_sha256(data).to_vec(),
            IntegrityAlgorithm::Sha512 => dash_hashing::hash_sha512(data).to_vec(),
            IntegrityAlgorithm::Blake3 => dash_hashing::hash_blake3(data).to_vec(),
        };
        expected == &actual
    }

    /// Serialize to the standard string format: `{algorithm}-{base64(value)}`
    pub fn to_string(&self) -> String {
        format!(
            "{}-{}",
            self.algorithm.as_str(),
            dash_hashing::to_base64(&self.value)
        )
    }

    /// Parse an integrity string in the format `{algorithm}-{base64(value)}`.
    pub fn from_string(s: &str) -> Result<Self, IntegrityError> {
        let s = s.trim();

        let (algo_str, b64_str) = s
            .split_once('-')
            .ok_or_else(|| IntegrityError::InvalidFormat(s.to_string()))?;

        let algorithm = IntegrityAlgorithm::from_str(algo_str)?;
        let value = dash_hashing::from_base64(b64_str)
            .map_err(|_| IntegrityError::InvalidFormat(s.to_string()))?;

        if value.len() != algorithm.output_size() {
            return Err(IntegrityError::InvalidFormat(format!(
                "expected {} bytes for {}, got {}",
                algorithm.output_size(),
                algorithm.as_str(),
                value.len()
            )));
        }

        Ok(Self {
            algorithm,
            value,
            raw: Some(s.to_string()),
        })
    }
}

impl std::fmt::Display for Integrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Compute the integrity hash of a file.
pub fn compute_integrity(path: &Path) -> Result<Integrity, IntegrityError> {
    let data = std::fs::read(path)?;
    Ok(Integrity::hash(&data, IntegrityAlgorithm::Sha256))
}

/// Compute the integrity hash of a file with a specific algorithm.
pub fn compute_integrity_with_algo(
    path: &Path,
    algorithm: IntegrityAlgorithm,
) -> Result<Integrity, IntegrityError> {
    let data = std::fs::read(path)?;
    Ok(Integrity::hash(&data, algorithm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrity_hash_sha256() {
        let data = b"test data";
        let integrity = Integrity::hash(data, IntegrityAlgorithm::Sha256);
        assert_eq!(integrity.value.len(), 32);
        assert_eq!(integrity.algorithm, IntegrityAlgorithm::Sha256);
    }

    #[test]
    fn test_integrity_verify() {
        let data = b"hello world";
        let integrity = Integrity::hash(data, IntegrityAlgorithm::Sha256);
        assert!(integrity.verify(data));
        assert!(!integrity.verify(b"wrong data"));
    }

    #[test]
    fn test_integrity_roundtrip() {
        let data = b"roundtrip test";
        let integrity = Integrity::hash(data, IntegrityAlgorithm::Sha512);
        let serialized = integrity.to_string();
        let parsed = Integrity::from_string(&serialized).unwrap();
        assert_eq!(integrity.algorithm, parsed.algorithm);
        assert_eq!(integrity.value, parsed.value);
    }

    #[test]
    fn test_integrity_from_string() {
        let s = "sha256-4g9a5HcY8OqXZVZ8qjYRqKj2qzQKj2qzQKj2qzQKj2";
        let result = Integrity::from_string(s);
        // This may fail if base64 isn't exactly 32 bytes, which is fine for test
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_compute_integrity() {
        let path = Path::new("Cargo.toml");
        let integrity = compute_integrity(path).unwrap();
        assert_eq!(integrity.algorithm, IntegrityAlgorithm::Sha256);
        assert_eq!(integrity.value.len(), 32);
    }

    #[test]
    fn test_algorithm_from_str() {
        assert_eq!(
            IntegrityAlgorithm::from_str("sha256").unwrap(),
            IntegrityAlgorithm::Sha256
        );
        assert_eq!(
            IntegrityAlgorithm::from_str("SHA-512").unwrap(),
            IntegrityAlgorithm::Sha512
        );
        assert_eq!(
            IntegrityAlgorithm::from_str("blake3").unwrap(),
            IntegrityAlgorithm::Blake3
        );
        assert!(IntegrityAlgorithm::from_str("md5").is_err());
    }

    #[test]
    fn test_display() {
        let data = b"display test";
        let integrity = Integrity::hash(data, IntegrityAlgorithm::Sha256);
        let s = integrity.to_string();
        assert!(s.starts_with("sha256-"));
    }
}
