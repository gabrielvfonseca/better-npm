//! Cryptographic signature generation and verification (Ed25519 + RSA).
//!
//! Provides high-level APIs for signing package payloads and
//! verifying signatures against trusted public keys.

use std::path::Path;

/// Error type for signature operations.
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("signing failed: {0}")]
    Signing(String),
    #[error("verification failed: {0}")]
    Verification(String),
    #[error("invalid key: {0}")]
    InvalidKey(String),
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("key not found: {0}")]
    KeyNotFound(String),
}

/// Supported signature algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SignatureAlgorithm {
    /// Ed25519 (EdDSA using Curve25519).
    Ed25519,
    /// RSA with SHA-256.
    Rsa,
}

impl SignatureAlgorithm {
    /// Return the algorithm as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureAlgorithm::Ed25519 => "ed25519",
            SignatureAlgorithm::Rsa => "rsa",
        }
    }

    /// Parse an algorithm from a string.
    pub fn from_str(s: &str) -> Result<Self, SignatureError> {
        match s.to_lowercase().as_str() {
            "ed25519" => Ok(SignatureAlgorithm::Ed25519),
            "rsa" | "rsa-sha256" => Ok(SignatureAlgorithm::Rsa),
            other => Err(SignatureError::UnsupportedAlgorithm(other.to_string())),
        }
    }
}

/// A cryptographic signature with metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Signature {
    /// The algorithm used to create this signature.
    pub algorithm: SignatureAlgorithm,
    /// The raw signature bytes.
    pub value: Vec<u8>,
    /// Identifier for the key used to create this signature.
    pub key_id: String,
    /// Timestamp when the signature was created (milliseconds since Unix epoch).
    pub timestamp: u64,
}

impl Signature {
    /// Create a new signature.
    pub fn new(algorithm: SignatureAlgorithm, value: Vec<u8>, key_id: impl Into<String>) -> Self {
        Self {
            algorithm,
            value,
            key_id: key_id.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Serialize the signature to a hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.value)
    }

    /// Parse a signature from hex.
    pub fn from_hex(hex_str: &str) -> Result<Vec<u8>, SignatureError> {
        hex::decode(hex_str).map_err(|e| SignatureError::Verification(e.to_string()))
    }
}

/// Sign a payload using the specified private key.
///
/// Supports Ed25519 and RSA signatures.
pub fn sign(payload: &[u8], private_key: &[u8], algorithm: SignatureAlgorithm) -> Result<Signature, SignatureError> {
    match algorithm {
        SignatureAlgorithm::Ed25519 => sign_ed25519(payload, private_key),
        SignatureAlgorithm::Rsa => sign_rsa(payload, private_key),
    }
}

fn sign_ed25519(payload: &[u8], private_key: &[u8]) -> Result<Signature, SignatureError> {
    let keypair = ed25519_dalek::SigningKey::from_keypair_bytes(
        private_key
            .try_into()
            .map_err(|_| SignatureError::InvalidKey("invalid Ed25519 key length (expected 64 bytes)".to_string()))?,
    )
    .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;

    use ed25519_dalek::Signer;
    let signature = keypair.sign(payload);

    Ok(Signature::new(
        SignatureAlgorithm::Ed25519,
        signature.to_bytes().to_vec(),
        hex::encode(keypair.verifying_key().to_bytes()),
    ))
}

fn sign_rsa(payload: &[u8], private_key: &[u8]) -> Result<Signature, SignatureError> {
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::signature::RandomizedSigner;
    use rsa::signature::SignatureEncoding;
    use sha2::Sha256;

    let rsa_private = rsa::RsaPrivateKey::from_pkcs8_der(private_key)
        .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;

    let signing_key = rsa::pss::SigningKey::<Sha256>::new(rsa_private);
    let mut rng = rand::thread_rng();
    let signature = signing_key
        .sign_with_rng(&mut rng, payload);

    Ok(Signature::new(
        SignatureAlgorithm::Rsa,
        signature.to_vec(),
        "rsa-key".to_string(),
    ))
}

/// Verify a signature against the given payload and public key.
pub fn verify(
    payload: &[u8],
    signature: &Signature,
    public_key: &[u8],
) -> Result<bool, SignatureError> {
    match signature.algorithm {
        SignatureAlgorithm::Ed25519 => verify_ed25519(payload, signature, public_key),
        SignatureAlgorithm::Rsa => verify_rsa(payload, signature, public_key),
    }
}

fn verify_ed25519(payload: &[u8], signature: &Signature, public_key: &[u8]) -> Result<bool, SignatureError> {
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
        public_key
            .try_into()
            .map_err(|_| SignatureError::InvalidKey("invalid Ed25519 public key length".to_string()))?,
    )
    .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;

    use ed25519_dalek::Verifier;
    let sig_bytes: [u8; 64] = signature
        .value
        .as_slice()
        .try_into()
        .map_err(|_| SignatureError::Verification("invalid signature length".to_string()))?;

    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    Ok(verifying_key.verify(payload, &sig).is_ok())
}

fn verify_rsa(payload: &[u8], signature: &Signature, public_key: &[u8]) -> Result<bool, SignatureError> {
    use rsa::pkcs1::DecodeRsaPublicKey;
    use rsa::signature::Verifier;
    use sha2::Sha256;

    let rsa_public = rsa::RsaPublicKey::from_pkcs1_der(public_key)
        .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;

    let sig = rsa::pss::Signature::try_from(signature.value.as_slice())
        .map_err(|e| SignatureError::Verification(e.to_string()))?;

    let verifying_key = rsa::pss::VerifyingKey::<Sha256>::new(rsa_public);
    Ok(verifying_key.verify(payload, &sig).is_ok())
}

/// Key pair for signing operations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyPair {
    /// The public key bytes (PKCS8 DER for RSA, raw for Ed25519).
    pub public: Vec<u8>,
    /// The private key bytes (PKCS8 DER for RSA, raw for Ed25519).
    pub private: Vec<u8>,
    /// The key algorithm.
    pub algorithm: SignatureAlgorithm,
    /// Optional key identifier.
    pub key_id: Option<String>,
}

impl KeyPair {
    /// Create a new key pair.
    pub fn new(public: Vec<u8>, private: Vec<u8>, algorithm: SignatureAlgorithm) -> Self {
        Self {
            public,
            private,
            algorithm,
            key_id: None,
        }
    }

    /// Sign a payload with this key pair.
    pub fn sign(&self, payload: &[u8]) -> Result<Signature, SignatureError> {
        let mut sig = sign(payload, &self.private, self.algorithm)?;
        if let Some(key_id) = &self.key_id {
            sig.key_id = key_id.clone();
        }
        Ok(sig)
    }

    /// Verify a signature with this key pair's public key.
    pub fn verify(&self, payload: &[u8], signature: &Signature) -> Result<bool, SignatureError> {
        verify(payload, signature, &self.public)
    }
}

/// Load a public key from a file.
pub fn load_public_key(path: &Path) -> Result<Vec<u8>, SignatureError> {
    Ok(std::fs::read(path)?)
}

/// Load a private key from a file.
pub fn load_private_key(path: &Path) -> Result<Vec<u8>, SignatureError> {
    Ok(std::fs::read(path)?)
}

/// Verify a signature on a file.
pub fn verify_file(path: &Path, signature: &Signature, public_key: &[u8]) -> Result<bool, SignatureError> {
    let data = std::fs::read(path)?;
    verify(&data, signature, public_key)
}

/// Sign a file.
pub fn sign_file(path: &Path, private_key: &[u8], algorithm: SignatureAlgorithm) -> Result<Signature, SignatureError> {
    let data = std::fs::read(path)?;
    sign(&data, private_key, algorithm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_verify() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let payload = b"hello, signatures!";
        let signature = sign(payload, &signing_key.to_keypair_bytes(), SignatureAlgorithm::Ed25519).unwrap();
        let result = verify(payload, &signature, &verifying_key.to_bytes()).unwrap();
        assert!(result);
    }

    #[test]
    fn test_ed25519_verify_failure() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let payload = b"original message";
        let signature = sign(payload, &signing_key.to_keypair_bytes(), SignatureAlgorithm::Ed25519).unwrap();

        // Wrong payload
        let result = verify(b"tampered message", &signature, &verifying_key.to_bytes()).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_ed25519_wrong_key() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let wrong_key = SigningKey::generate(&mut csprng);

        let payload = b"test";
        let signature = sign(payload, &signing_key.to_keypair_bytes(), SignatureAlgorithm::Ed25519).unwrap();
        let result = verify(payload, &signature, &wrong_key.verifying_key().to_bytes()).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_algorithm_from_str() {
        assert_eq!(
            SignatureAlgorithm::from_str("ed25519").unwrap(),
            SignatureAlgorithm::Ed25519
        );
        assert_eq!(
            SignatureAlgorithm::from_str("rsa").unwrap(),
            SignatureAlgorithm::Rsa
        );
        assert!(SignatureAlgorithm::from_str("unknown").is_err());
    }

    #[test]
    fn test_algorithm_display() {
        assert_eq!(SignatureAlgorithm::Ed25519.as_str(), "ed25519");
        assert_eq!(SignatureAlgorithm::Rsa.as_str(), "rsa");
    }

    #[test]
    fn test_signature_hex() {
        let sig = Signature::new(SignatureAlgorithm::Ed25519, vec![1, 2, 3, 4], "key-1");
        let hex = sig.to_hex();
        assert_eq!(hex, "01020304");
    }

    #[test]
    fn test_keypair_sign_verify() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let vk = sk.verifying_key();

        let kp = KeyPair::new(
            vk.to_bytes().to_vec(),
            sk.to_keypair_bytes().to_vec(),
            SignatureAlgorithm::Ed25519,
        );

        let payload = b"keypair test";
        let sig = kp.sign(payload).unwrap();
        let valid = kp.verify(payload, &sig).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_sign_file() {
        // Create a temp file, sign it, verify
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"file content").unwrap();

        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let vk = sk.verifying_key();

        let signature = sign_file(&file_path, &sk.to_keypair_bytes(), SignatureAlgorithm::Ed25519).unwrap();
        let valid = verify_file(&file_path, &signature, &vk.to_bytes()).unwrap();
        assert!(valid);
    }
}
