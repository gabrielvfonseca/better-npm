// bnpm-cid: BLAKE3 CID, multihash, and base32 canonical encoding

use ciborium::Value;
use ciborium::de::from_reader;
use std::str::FromStr;
use thiserror::Error;

/// Error type for CID operations
#[derive(Error, Debug, PartialEq)]
pub enum CidError {
    #[error("invalid CID format: expected 'b3:' prefix")]
    InvalidPrefix,
    #[error("invalid base32 encoding: {0}")]
    InvalidBase32(String),
    #[error("invalid multihash: {0}")]
    InvalidMultihash(String),
    #[error("hex decode error: {0}")]
    HexDecodeError(hex::FromHexError),
    #[error("base32 decode error")]
    Base32DecodeError,
    #[error("BLAKE3 hash length mismatch: expected 32 bytes, got {0}")]
    Blak3LengthMismatch(usize),
    #[error(
        "multihash length mismatch: expected 34 bytes (1 byte code + 1 byte len + 32 bytes hash), got {0}"
    )]
    MultihashLengthMismatch(usize),
    #[error("unsupported multihash code: 0x{0:x} (expected 0x1f for BLAKE3-256)")]
    UnsupportedMultihashCode(u8),
    #[error("multihash digest length mismatch: expected 32 bytes for BLAKE3-256, got {0}")]
    MultihashDigestLengthMismatch(usize),
    #[error("CBOR encoding error: {0}")]
    CborEncodeError(String),
    #[error("CBOR decoding error: {0}")]
    CborDecodeError(String),
    #[error("non-canonical CBOR encoding: {0}")]
    NonCanonicalEncoding(String),
}

impl From<hex::FromHexError> for CidError {
    fn from(e: hex::FromHexError) -> Self {
        CidError::HexDecodeError(e)
    }
}

/// BLAKE3-256 multihash code (from multicodec table)
pub const BLAKE3_256_CODE: u8 = 0x1f;
/// BLAKE3-256 digest length in bytes
pub const BLAKE3_256_DIGEST_LEN: usize = 32;
/// Full multihash length: 1 byte code + 1 byte length + 32 bytes digest
pub const BLAKE3_256_MULTIHASH_LEN: usize = 1 + 1 + BLAKE3_256_DIGEST_LEN;

/// A Content ID (CID) using BLAKE3 multihash with base32 encoding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid {
    /// The full multihash bytes: [code, length, digest...]
    multihash: [u8; BLAKE3_256_MULTIHASH_LEN],
}

impl serde::Serialize for Cid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Cid::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Cid {
    /// Create a new CID from canonical bytes by computing BLAKE3-256
    pub fn new(canonical_bytes: &[u8]) -> Self {
        let hash = blake3::hash(canonical_bytes);
        let mut multihash = [0u8; BLAKE3_256_MULTIHASH_LEN];
        multihash[0] = BLAKE3_256_CODE;
        multihash[1] = BLAKE3_256_DIGEST_LEN as u8;
        multihash[2..].copy_from_slice(hash.as_bytes());
        Self { multihash }
    }

    /// Create a CID from an existing multihash (validates format)
    pub fn from_multihash(multihash: &[u8]) -> Result<Self, CidError> {
        if multihash.len() != BLAKE3_256_MULTIHASH_LEN {
            return Err(CidError::MultihashLengthMismatch(multihash.len()));
        }
        if multihash[0] != BLAKE3_256_CODE {
            return Err(CidError::UnsupportedMultihashCode(multihash[0]));
        }
        if multihash[1] != BLAKE3_256_DIGEST_LEN as u8 {
            return Err(CidError::MultihashDigestLengthMismatch(multihash[1] as usize));
        }
        let mut mh = [0u8; BLAKE3_256_MULTIHASH_LEN];
        mh.copy_from_slice(multihash);
        Ok(Self { multihash: mh })
    }

    fn from_cid_str(s: &str) -> Result<Self, CidError> {
        let prefix = "b3:";
        if !s.starts_with(prefix) {
            return Err(CidError::InvalidPrefix);
        }
        let b32 = &s[prefix.len()..];
        let multihash = decode_base32(b32)?;
        Self::from_multihash(&multihash)
    }

    /// Get the raw multihash bytes
    pub fn multihash(&self) -> &[u8] {
        &self.multihash
    }

    /// Get the BLAKE3 digest (32 bytes)
    pub fn digest(&self) -> &[u8] {
        &self.multihash[2..]
    }

    /// Get the hex-encoded BLAKE3 digest
    pub fn digest_hex(&self) -> String {
        hex::encode(self.digest())
    }

    /// Get the full multihash hex-encoded
    pub fn multihash_hex(&self) -> String {
        hex::encode(self.multihash)
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b32 = encode_base32(&self.multihash);
        write!(f, "b3:{}", b32)
    }
}

impl std::str::FromStr for Cid {
    type Err = CidError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_cid_str(s)
    }
}

/// Encode bytes as RFC 4648 base32 (lowercase, no padding)
fn encode_base32(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut result = String::with_capacity((data.len() * 8).div_ceil(5));
    let mut bits = 0u32;
    let mut bits_len = 0;

    for &byte in data {
        bits = (bits << 8) | byte as u32;
        bits_len += 8;
        while bits_len >= 5 {
            bits_len -= 5;
            let index = (bits >> bits_len) & 0x1f;
            result.push(ALPHABET[index as usize] as char);
        }
    }

    if bits_len > 0 {
        let index = (bits << (5 - bits_len)) & 0x1f;
        result.push(ALPHABET[index as usize] as char);
    }

    result
}

/// Decode RFC 4648 base32 (lowercase, no padding) to bytes
fn decode_base32(s: &str) -> Result<Vec<u8>, CidError> {
    let s = s.to_lowercase();
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut decode_map = [0xffu8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        decode_map[c as usize] = i as u8;
    }

    let mut bits = 0u32;
    let mut bits_len = 0;
    let mut result = Vec::with_capacity(s.len() * 5 / 8);

    for c in s.chars() {
        let byte = c as u8;
        let val = decode_map[byte as usize];
        if val == 0xff {
            return Err(CidError::InvalidBase32(format!("invalid character: {}", c)));
        }
        bits = (bits << 5) | val as u32;
        bits_len += 5;
        if bits_len >= 8 {
            bits_len -= 8;
            result.push((bits >> bits_len) as u8);
        }
    }

    Ok(result)
}

/// Compute BLAKE3-256 hash of canonical bytes
pub fn hash_blake3_256(data: &[u8]) -> [u8; 32] {
    let hash = blake3::hash(data);
    *hash.as_bytes()
}

/// Compute multihash for BLAKE3-256
pub fn multihash_blake3_256(data: &[u8]) -> [u8; BLAKE3_256_MULTIHASH_LEN] {
    let mut mh = [0u8; BLAKE3_256_MULTIHASH_LEN];
    mh[0] = BLAKE3_256_CODE;
    mh[1] = BLAKE3_256_DIGEST_LEN as u8;
    mh[2..].copy_from_slice(&hash_blake3_256(data));
    mh
}

/// Verify that canonical bytes produce the expected CID
pub fn verify_cid(canonical_bytes: &[u8], expected_cid: &str) -> Result<bool, CidError> {
    let computed = Cid::new(canonical_bytes);
    let expected = Cid::from_str(expected_cid)?;
    Ok(computed == expected)
}

/// Canonical CBOR encoding rules (RFC 8949 §4.2.1, deterministic encoding):
/// - Maps: keys sorted lexicographically by bytewise comparison of their encoded form
/// - No indefinite-length items
/// - Float values: prefer integer over float where an exact integer is representable
/// - Byte strings and text strings: definite length
/// - Array items in order
///
/// This function canonicalizes the value (sorts maps, normalizes floats) before encoding.
pub fn canonical_cbor_encode(value: &Value) -> Result<Vec<u8>, CidError> {
    let canonical = canonicalize_value(value);
    let mut buf = Vec::new();
    ciborium::ser::into_writer(&canonical, &mut buf)
        .map_err(|e| CidError::CborEncodeError(e.to_string()))?;
    Ok(buf)
}

/// Recursively canonicalize a CBOR Value:
/// - Sort map entries by bytewise comparison of each key's CBOR encoding
/// - Recurse into nested maps and arrays
/// - Convert floats that represent exact integers to integer type
fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Map(entries) => {
            let mut canonical_entries: Vec<_> = entries
                .iter()
                .map(|(k, v)| (canonicalize_value(k), canonicalize_value(v)))
                .collect();
            canonical_entries.sort_by(|(k1, _), (k2, _)| {
                let mut b1 = Vec::new();
                let mut b2 = Vec::new();
                let _ = ciborium::ser::into_writer(k1, &mut b1);
                let _ = ciborium::ser::into_writer(k2, &mut b2);
                b1.cmp(&b2)
            });
            Value::Map(canonical_entries)
        },
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        Value::Float(f) => {
            if f.is_finite() {
                let i = *f as i64;
                if i as f64 == *f {
                    return Value::Integer(i.into());
                }
            }
            value.clone()
        },
        _ => value.clone(),
    }
}

/// Validate that raw CBOR bytes are in canonical form per RFC 8949 §4.2.1.
///
/// Checks performed:
/// - Decodable as well-formed CBOR (rejects invalid/malformed bytes)
/// - Map keys are sorted by bytewise comparison of encoded keys
/// - No float values that could be represented as integers
/// - No non-minimal integer encodings (ciborium's serializer always emits minimal forms,
///   so re-encoding the decoded value and comparing catches these)
///
/// Returns `Ok(())` if the bytes are canonical, or `Err(CidError::NonCanonicalEncoding(...))`
/// with a description of the violation.
pub fn canonical_cbor_validate(bytes: &[u8]) -> Result<(), CidError> {
    let value: Value = from_reader(bytes).map_err(|e| CidError::CborDecodeError(e.to_string()))?;
    let reencoded = canonical_cbor_encode(&value)?;
    if reencoded != bytes {
        return Err(CidError::NonCanonicalEncoding(format!(
            "re-encoded canonical form differs from input: expected {} bytes, got {} bytes. \
             Common causes: unsorted map keys, float where integer suffices, non-minimal integer encoding.",
            bytes.len(),
            reencoded.len()
        )));
    }
    Ok(())
}

/// Trait for types that can be encoded to canonical CBOR
pub trait CanonicalCbor {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>, CidError>;
}

impl CanonicalCbor for Value {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>, CidError> {
        canonical_cbor_encode(self)
    }
}

impl CanonicalCbor for Vec<(String, Value)> {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>, CidError> {
        let mut sorted = self.clone();
        sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let value = Value::Map(sorted.into_iter().map(|(k, v)| (Value::Text(k), v)).collect());
        canonical_cbor_encode(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_capsule() {
        let cid = Cid::new(b"");
        assert_eq!(
            cid.digest_hex(),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        assert_eq!(cid.to_string(), "b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq");
        assert_eq!(
            cid.multihash_hex(),
            "1f20af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn test_null_byte() {
        let cid = Cid::new(&[0x00]);
        assert_eq!(cid.to_string(), "b3:d4qc2ow637yrwyprjseg4nnpua3hg3onq6tu2j5vyfiqejoq6wjoeey");
    }

    #[test]
    fn test_hello() {
        let cid = Cid::new(b"hello");
        assert_eq!(cid.to_string(), "b3:d4qovdywhwzynauslzcjdrpfrvf3gudo7dau5n4kq3uqrrlcjjtsady");
    }

    #[test]
    fn test_cbor_empty_map() {
        let cid = Cid::new(&[0xa0]);
        assert_eq!(cid.to_string(), "b3:d4qb7fgl6mj3htrdev5heupkb7evujcvn2tbdzhy6r26ksmxdoxnwaq");
    }

    #[test]
    fn test_cbor_map_sorted() {
        let cid = Cid::new(&[0xa2, 0x61, 0x61, 0x01, 0x61, 0x62, 0x02]);
        assert_eq!(cid.to_string(), "b3:d4qh5zm6u2ykvng5xbrmeczj5qjt7qx4nuv27ipq4r3igfdrd4g5m5a");
    }

    #[test]
    fn test_cbor_map_unsorted() {
        let cid = Cid::new(&[0xa2, 0x61, 0x62, 0x02, 0x61, 0x61, 0x01]);
        assert_eq!(cid.to_string(), "b3:d4qgt7gzjq2prwxtyaycgmwejeo45rqquxu37dws3ji5jxx6n63owni");
    }

    #[test]
    fn test_unicode_snowman() {
        let cid = Cid::new(&[0xe2, 0x98, 0x83]);
        assert_eq!(cid.to_string(), "b3:d4qbm6kemwstuvynx4msciz2dsllqefmzojizucljvbk5ko4faiyhqy");
    }

    #[test]
    fn test_binary_sequence_32() {
        let bytes: Vec<u8> = (0x00..=0x1f).collect();
        let cid = Cid::new(&bytes);
        assert_eq!(cid.to_string(), "b3:d4qokkhjk6mag7pucbkd3hzr4olozxkfrvy3cv6wafbzroxdf62wyzi");
    }

    #[test]
    fn test_nested_cbor() {
        let cid = Cid::new(&[0xa1, 0x61, 0x6b, 0xa1, 0x6b, 0x61, 0xa2, 0x01, 0x02]);
        assert_eq!(cid.to_string(), "b3:d4qnkjlz3ruigfing4pmiq5gtm2i33btyltegzzldfnukt6vahcjhei");
    }

    #[test]
    fn test_large_256_bytes() {
        let bytes: Vec<u8> = (0x00..=0xff).collect();
        let cid = Cid::new(&bytes);
        assert_eq!(cid.to_string(), "b3:d4qeusk3uqsgc5eozkh5vvqy7f3ku4tmykid32p4wqdtlj4gvqobs2y");
    }

    #[test]
    fn test_all_zero_32() {
        let cid = Cid::new(&[0u8; 32]);
        assert_eq!(cid.to_string(), "b3:d4qcvwudygazuu3s3lqshd6b33ishsaqj7nkcwdcvlxgsqukdaqpzwq");
    }

    #[test]
    fn test_cbor_with_half_precision_float() {
        let cid = Cid::new(&[0xa1, 0x61, 0x61, 0xf9, 0x3c, 0x00]);
        assert_eq!(cid.to_string(), "b3:d4qbfakx4vshvdjjuxi4r3osas4pwaohs7by5ybfldiccwfe4p5wcdy");
    }

    #[test]
    fn test_from_str_valid() {
        let cid =
            Cid::from_str("b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq").unwrap();
        assert_eq!(cid.to_string(), "b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq");
    }

    #[test]
    fn test_from_str_invalid_prefix() {
        let err = Cid::from_str("x3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq");
        assert!(matches!(err, Err(CidError::InvalidPrefix)));
    }

    #[test]
    fn test_from_str_invalid_base32() {
        let err = Cid::from_str("b3:!!!invalid!!!");
        assert!(matches!(err, Err(CidError::InvalidBase32(_))));
    }

    #[test]
    fn test_roundtrip() {
        let original = Cid::new(b"test data for roundtrip");
        let s = original.to_string();
        let parsed = Cid::from_str(&s).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_multihash_validation() {
        let bad_code = [0x20, 0x20];
        let err = Cid::from_multihash(&bad_code);
        assert!(matches!(err, Err(CidError::MultihashLengthMismatch(_))));

        let mut wrong_len = Vec::with_capacity(36);
        for _ in 0..12 {
            wrong_len.extend_from_slice(&[0x1f, 0x20, 0x00]);
        }
        let err = Cid::from_multihash(&wrong_len);
        assert!(matches!(err, Err(CidError::MultihashLengthMismatch(_))));

        let mut unsupported = [0u8; BLAKE3_256_MULTIHASH_LEN];
        unsupported[0] = 0x20;
        let err = Cid::from_multihash(&unsupported);
        assert!(matches!(err, Err(CidError::UnsupportedMultihashCode(0x20))));

        let mut bad_digest_len = [0u8; BLAKE3_256_MULTIHASH_LEN];
        bad_digest_len[0] = BLAKE3_256_CODE;
        bad_digest_len[1] = 0x1f;
        let err = Cid::from_multihash(&bad_digest_len);
        assert!(matches!(err, Err(CidError::MultihashDigestLengthMismatch(31))));
    }

    #[test]
    fn test_hash_blake3_256() {
        let hash = hash_blake3_256(b"test");
        assert_eq!(hash.len(), 32);
        assert_eq!(
            hex::encode(hash),
            "4878ca0425c739fa427f7eda20fe845f6b2e46ba5fe2a14df5b1e32f50603215"
        );
    }

    #[test]
    fn test_multihash_blake3_256() {
        let mh = multihash_blake3_256(b"test");
        assert_eq!(mh.len(), 34);
        assert_eq!(mh[0], 0x1f);
        assert_eq!(mh[1], 0x20);
    }

    #[test]
    fn test_verify_cid() {
        assert!(
            verify_cid(b"", "b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq").unwrap()
        );
        assert!(
            !verify_cid(b"wrong", "b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq")
                .unwrap()
        );
    }

    #[test]
    fn test_base32_encoding() {
        let data = b"test";
        let encoded = encode_base32(data);
        assert!(encoded.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        assert!(!encoded.contains('='));

        let decoded = decode_base32(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_all_12_vectors() {
        let vectors_json = include_str!("../../../docs/schemas/test-vectors/cid/cid-vectors.json");
        let parsed: serde_json::Value =
            serde_json::from_str(vectors_json).expect("failed to parse cid-vectors.json");
        let vecs = parsed["vectors"].as_array().expect("vectors field must be an array");

        assert_eq!(vecs.len(), 12, "expected exactly 12 ratified test vectors");

        for (i, v) in vecs.iter().enumerate() {
            let canonical_hex = v["canonicalBytes"].as_str().unwrap_or("");
            let expected_cid = v["expectedCid"].as_str().unwrap();
            let multihash_hex_expected = v["multihashHex"].as_str().unwrap();
            let digest_hex_expected = v["blake3DigestHex"].as_str().unwrap();

            let canonical_bytes = hex::decode(canonical_hex).unwrap();
            let cid = Cid::new(&canonical_bytes);

            assert_eq!(
                cid.to_string(),
                expected_cid,
                "vector {i}: expectedCid mismatch. input={canonical_hex}"
            );
            assert_eq!(
                cid.multihash_hex(),
                multihash_hex_expected,
                "vector {i}: multihashHex mismatch. input={canonical_hex}"
            );
            assert_eq!(
                cid.digest_hex(),
                digest_hex_expected,
                "vector {i}: blake3DigestHex mismatch. input={canonical_hex}"
            );
        }
    }

    #[test]
    fn test_deterministic_map_encoding() {
        let unsorted = Value::Map(vec![
            (Value::Text("b".into()), Value::Integer(2.into())),
            (Value::Text("a".into()), Value::Integer(1.into())),
        ]);

        let sorted = Value::Map(vec![
            (Value::Text("a".into()), Value::Integer(1.into())),
            (Value::Text("b".into()), Value::Integer(2.into())),
        ]);

        let unsorted_bytes = canonical_cbor_encode(&unsorted).unwrap();
        let sorted_bytes = canonical_cbor_encode(&sorted).unwrap();

        assert_eq!(
            unsorted_bytes, sorted_bytes,
            "encoding must be deterministic regardless of map insertion order"
        );
    }

    #[test]
    fn test_deterministic_nested_map_encoding() {
        let inner_unsorted = Value::Map(vec![
            (Value::Text("z".into()), Value::Integer(3.into())),
            (Value::Text("a".into()), Value::Integer(1.into())),
        ]);

        let unsorted = Value::Map(vec![
            (Value::Text("b".into()), inner_unsorted),
            (Value::Text("a".into()), Value::Integer(0.into())),
        ]);

        let inner_sorted = Value::Map(vec![
            (Value::Text("a".into()), Value::Integer(1.into())),
            (Value::Text("z".into()), Value::Integer(3.into())),
        ]);

        let sorted = Value::Map(vec![
            (Value::Text("a".into()), Value::Integer(0.into())),
            (Value::Text("b".into()), inner_sorted),
        ]);

        let unsorted_bytes = canonical_cbor_encode(&unsorted).unwrap();
        let sorted_bytes = canonical_cbor_encode(&sorted).unwrap();

        assert_eq!(
            unsorted_bytes, sorted_bytes,
            "encoding must be deterministic for nested maps regardless of insertion order"
        );
    }

    #[test]
    fn test_validate_rejects_half_float() {
        let non_canonical = hex::decode("a16161f93c00").unwrap();
        let result = canonical_cbor_validate(&non_canonical);
        assert!(
            result.is_err(),
            "half-float encoding of 1.0 (f9 3c 00) must be rejected as non-canonical"
        );
        match result {
            Err(CidError::NonCanonicalEncoding(msg)) => {
                assert!(
                    msg.contains("re-encoded"),
                    "error should mention re-encoding mismatch: {msg}"
                );
            },
            other => panic!("expected NonCanonicalEncoding error, got {other:?}"),
        }
    }

    #[test]
    fn test_validate_rejects_unsorted_map() {
        let sorted = Value::Map(vec![
            (Value::Text("a".into()), Value::Integer(1.into())),
            (Value::Text("b".into()), Value::Integer(2.into())),
        ]);
        let sorted_bytes = canonical_cbor_encode(&sorted).unwrap();
        assert!(
            canonical_cbor_validate(&sorted_bytes).is_ok(),
            "sorted map bytes should pass validation"
        );

        let unsorted_raw = hex::decode("a2616202616101").unwrap();
        assert!(
            canonical_cbor_validate(&unsorted_raw).is_err(),
            "unsorted map bytes (b before a) must be rejected"
        );
    }

    #[test]
    fn test_validate_accepts_canonical() {
        let empty_map: Value = Value::Map(vec![]);
        let bytes = canonical_cbor_encode(&empty_map).unwrap();
        assert!(canonical_cbor_validate(&bytes).is_ok());

        let value = Value::Map(vec![(Value::Text("key".into()), Value::Integer(42.into()))]);
        let bytes = canonical_cbor_encode(&value).unwrap();
        assert!(canonical_cbor_validate(&bytes).is_ok());
    }

    #[test]
    fn test_float_to_int_normalization() {
        let value = Value::Map(vec![(Value::Text("a".into()), Value::Float(1.0))]);
        let bytes = canonical_cbor_encode(&value).unwrap();

        let canonical = Value::Map(vec![(Value::Text("a".into()), Value::Integer(1.into()))]);
        let canonical_bytes = canonical_cbor_encode(&canonical).unwrap();

        assert_eq!(
            bytes, canonical_bytes,
            "float 1.0 must be normalized to integer 1 in canonical encoding"
        );
    }
}
