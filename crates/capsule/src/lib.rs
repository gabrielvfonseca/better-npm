use std::io::{Cursor, Read};

use ciborium::Value;
use ciborium::value::Integer;
use bnpm_cid::Cid;

// ── Constants ──────────────────────────────────────────────────────────────

pub const BNPMCAP1_MAGIC: &[u8; 8] = b"BNPMCAP1";
pub const BNPMCAP1_VERSION: u16 = 1;
const COMPRESSION_LEVEL: i32 = 3;

const FLAG_COMPRESSED_OBJECTS: u32 = 1 << 0;
const FLAG_HAS_ATTESTATIONS: u32 = 1 << 1;
const FLAG_SIGNED: u32 = 1 << 2;
const FLAG_RANGE_INDEX: u32 = 1 << 3;

// ── CapsuleFlags ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapsuleFlags {
    bits: u32,
}

impl CapsuleFlags {
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn compressed(&self) -> bool {
        self.bits & FLAG_COMPRESSED_OBJECTS != 0
    }

    pub fn set_compressed(&mut self, val: bool) {
        if val {
            self.bits |= FLAG_COMPRESSED_OBJECTS;
        } else {
            self.bits &= !FLAG_COMPRESSED_OBJECTS;
        }
    }

    pub fn has_attestations(&self) -> bool {
        self.bits & FLAG_HAS_ATTESTATIONS != 0
    }

    pub fn set_has_attestations(&mut self, val: bool) {
        if val {
            self.bits |= FLAG_HAS_ATTESTATIONS;
        } else {
            self.bits &= !FLAG_HAS_ATTESTATIONS;
        }
    }

    pub fn signed(&self) -> bool {
        self.bits & FLAG_SIGNED != 0
    }

    pub fn set_signed(&mut self, val: bool) {
        if val {
            self.bits |= FLAG_SIGNED;
        } else {
            self.bits &= !FLAG_SIGNED;
        }
    }

    pub fn range_index(&self) -> bool {
        self.bits & FLAG_RANGE_INDEX != 0
    }

    pub fn set_range_index(&mut self, val: bool) {
        if val {
            self.bits |= FLAG_RANGE_INDEX;
        } else {
            self.bits &= !FLAG_RANGE_INDEX;
        }
    }

    pub fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    pub fn bits(&self) -> u32 {
        self.bits
    }
}

impl Default for CapsuleFlags {
    fn default() -> Self {
        Self::new()
    }
}

// ── ModeClass ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModeClass {
    Regular = 0,
    Executable = 1,
    Directory = 2,
    Symlink = 3,
}

impl ModeClass {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Regular),
            1 => Some(Self::Executable),
            2 => Some(Self::Directory),
            3 => Some(Self::Symlink),
            _ => None,
        }
    }

    pub fn permissions(&self) -> u32 {
        match self {
            Self::Regular => 0o644,
            Self::Executable => 0o755,
            Self::Directory => 0o755,
            Self::Symlink => 0o644,
        }
    }
}

// ── FileEntry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    pub path: String,
    pub mode_class: ModeClass,
    pub size: u64,
    pub blake3_hash: [u8; 32],
    pub offset: u64,
    pub compressed_size: u64,
}

impl FileEntry {
    pub fn new(path: String, data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self {
            path,
            mode_class: ModeClass::Regular,
            size: data.len() as u64,
            blake3_hash: *hash.as_bytes(),
            offset: 0,
            compressed_size: 0,
        }
    }
}

fn integer_to_u64(int: Integer) -> Option<u64> {
    let v: i128 = int.into();
    if v >= 0 { Some(v as u64) } else { None }
}

// ── FileTable ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct FileTable(Vec<FileEntry>);

impl FileTable {
    pub fn new(entries: Vec<FileEntry>) -> Self {
        Self(entries)
    }

    pub fn entries(&self) -> &[FileEntry] {
        &self.0
    }

    pub fn to_cbor(&self) -> Result<Vec<u8>, CapsuleError> {
        let entries: Vec<Value> = self
            .0
            .iter()
            .map(|e| {
                Value::Array(vec![
                    Value::from(e.path.as_str()),
                    Value::from(e.mode_class as u64),
                    Value::from(e.size),
                    Value::from(e.blake3_hash.to_vec()),
                    Value::from(e.offset),
                    Value::from(e.compressed_size),
                ])
            })
            .collect();
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&Value::Array(entries), &mut buf)
            .map_err(|e| CapsuleError::CorruptFileTable(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_cbor(value: &Value) -> Result<Self, CapsuleError> {
        let arr = value
            .as_array()
            .ok_or_else(|| CapsuleError::CorruptFileTable("expected array".into()))?;
        let mut entries = Vec::with_capacity(arr.len());
        for item in arr {
            let entry_arr = item
                .as_array()
                .ok_or_else(|| CapsuleError::CorruptFileTable("entry not array".into()))?;
            if entry_arr.len() < 4 {
                return Err(CapsuleError::CorruptFileTable("entry too short".into()));
            }
            let path = entry_arr[0]
                .as_text()
                .ok_or_else(|| CapsuleError::CorruptFileTable("invalid path".into()))?
                .to_string();
            let mode_val = integer_to_u64(
                entry_arr[1]
                    .as_integer()
                    .ok_or_else(|| CapsuleError::CorruptFileTable("invalid mode_class".into()))?,
            )
            .ok_or_else(|| CapsuleError::CorruptFileTable("invalid mode_class value".into()))?;
            let mode_class = ModeClass::from_u8(mode_val as u8).ok_or_else(|| {
                CapsuleError::CorruptFileTable(format!("invalid mode_class: {}", mode_val))
            })?;
            let size = integer_to_u64(
                entry_arr[2]
                    .as_integer()
                    .ok_or_else(|| CapsuleError::CorruptFileTable("invalid size".into()))?,
            )
            .ok_or_else(|| CapsuleError::CorruptFileTable("invalid size value".into()))?;
            let hash_bytes: &[u8] = entry_arr[3]
                .as_bytes()
                .ok_or_else(|| CapsuleError::CorruptFileTable("invalid hash".into()))?;
            if hash_bytes.len() != 32 {
                return Err(CapsuleError::CorruptFileTable("hash must be 32 bytes".into()));
            }
            let mut blake3_hash = [0u8; 32];
            blake3_hash.copy_from_slice(hash_bytes);
            let offset = if entry_arr.len() > 4 {
                integer_to_u64(
                    entry_arr[4]
                        .as_integer()
                        .ok_or_else(|| CapsuleError::CorruptFileTable("invalid offset".into()))?,
                )
                .unwrap_or(0)
            } else {
                0
            };
            let compressed_size = if entry_arr.len() > 5 {
                integer_to_u64(entry_arr[5].as_integer().ok_or_else(|| {
                    CapsuleError::CorruptFileTable("invalid compressed_size".into())
                })?)
                .unwrap_or(0)
            } else {
                0
            };
            entries.push(FileEntry {
                path,
                mode_class,
                size,
                blake3_hash,
                offset,
                compressed_size,
            });
        }
        Ok(Self(entries))
    }
}

// ── AttestationType ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttestationType {
    InTotoStatement = 0,
    SbomSpdx = 1,
    SbomCycloneDx = 2,
    SignatureEd25519 = 3,
    SignatureX509 = 4,
    TransparencyProof = 5,
    PolicyEvaluation = 6,
    PublisherIdentity = 7,
}

impl AttestationType {
    pub fn from_u64(v: u64) -> Option<Self> {
        match v {
            0 => Some(Self::InTotoStatement),
            1 => Some(Self::SbomSpdx),
            2 => Some(Self::SbomCycloneDx),
            3 => Some(Self::SignatureEd25519),
            4 => Some(Self::SignatureX509),
            5 => Some(Self::TransparencyProof),
            6 => Some(Self::PolicyEvaluation),
            7 => Some(Self::PublisherIdentity),
            _ => None,
        }
    }
}

// ── Attestation ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Attestation {
    pub type_: AttestationType,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
    pub media_type: Option<String>,
    pub signer: Option<String>,
}

impl Attestation {
    pub fn new(type_: AttestationType, payload: Vec<u8>, signature: Vec<u8>) -> Self {
        Self { type_, payload, signature, media_type: None, signer: None }
    }

    pub fn with_media_type(mut self, media_type: String) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn with_signer(mut self, signer: String) -> Self {
        self.signer = Some(signer);
        self
    }
}

fn attestations_to_cbor(attestations: &[Attestation]) -> Result<Vec<u8>, CapsuleError> {
    let arr: Vec<Value> = attestations
        .iter()
        .map(|a| {
            let mut map = vec![
                (Value::from("type"), Value::from(a.type_ as u64)),
                (Value::from("payload"), Value::from(a.payload.clone())),
                (Value::from("signature"), Value::from(a.signature.clone())),
            ];
            if let Some(ref mt) = a.media_type {
                map.push((Value::from("media-type"), Value::from(mt.as_str())));
            }
            if let Some(ref s) = a.signer {
                map.push((Value::from("signer"), Value::from(s.as_str())));
            }
            sort_cbor_map(&mut map);
            Value::Map(map)
        })
        .collect();
    let mut buf = Vec::new();
    ciborium::ser::into_writer(&Value::Array(arr), &mut buf)
        .map_err(|e| CapsuleError::CorruptAttestations(e.to_string()))?;
    Ok(buf)
}

fn attestations_from_cbor(value: &Value) -> Result<Vec<Attestation>, CapsuleError> {
    let arr = value
        .as_array()
        .ok_or_else(|| CapsuleError::CorruptAttestations("expected array".into()))?;
    let mut result = Vec::with_capacity(arr.len());
    for item in arr {
        let map = item
            .as_map()
            .ok_or_else(|| CapsuleError::CorruptAttestations("entry not map".into()))?;
        let mut type_v: Option<u64> = None;
        let mut payload: Option<Vec<u8>> = None;
        let mut signature: Option<Vec<u8>> = None;
        let mut media_type: Option<String> = None;
        let mut signer: Option<String> = None;
        for (k, v) in map {
            let key = k
                .as_text()
                .ok_or_else(|| CapsuleError::CorruptAttestations("non-text key".into()))?;
            match key {
                "type" => {
                    type_v = Some(
                        integer_to_u64(v.as_integer().ok_or_else(|| {
                            CapsuleError::CorruptAttestations("invalid type".into())
                        })?)
                        .ok_or_else(|| {
                            CapsuleError::CorruptAttestations("invalid type value".into())
                        })?,
                    );
                },
                "payload" => {
                    payload = Some(
                        v.as_bytes()
                            .ok_or_else(|| {
                                CapsuleError::CorruptAttestations("invalid payload".into())
                            })?
                            .clone(),
                    );
                },
                "signature" => {
                    signature = Some(
                        v.as_bytes()
                            .ok_or_else(|| {
                                CapsuleError::CorruptAttestations("invalid signature".into())
                            })?
                            .clone(),
                    );
                },
                "media-type" => {
                    media_type = Some(
                        v.as_text()
                            .ok_or_else(|| {
                                CapsuleError::CorruptAttestations("invalid media-type".into())
                            })?
                            .to_string(),
                    );
                },
                "signer" => {
                    signer = Some(
                        v.as_text()
                            .ok_or_else(|| {
                                CapsuleError::CorruptAttestations("invalid signer".into())
                            })?
                            .to_string(),
                    );
                },
                _ => {},
            }
        }
        let type_v =
            type_v.ok_or_else(|| CapsuleError::CorruptAttestations("missing type".into()))?;
        let type_ = AttestationType::from_u64(type_v).ok_or_else(|| {
            CapsuleError::CorruptAttestations(format!("unknown attestation type: {}", type_v))
        })?;
        let payload =
            payload.ok_or_else(|| CapsuleError::CorruptAttestations("missing payload".into()))?;
        let signature = signature
            .ok_or_else(|| CapsuleError::CorruptAttestations("missing signature".into()))?;
        result.push(Attestation { type_, payload, signature, media_type, signer });
    }
    Ok(result)
}

// ── CapsuleHeader ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct CapsuleHeader {
    pub magic: [u8; 8],
    pub version: u16,
    pub flags: CapsuleFlags,
}

impl CapsuleHeader {
    pub fn new(flags: CapsuleFlags) -> Self {
        Self { magic: *BNPMCAP1_MAGIC, version: BNPMCAP1_VERSION, flags }
    }

    fn to_bytes(&self) -> [u8; 13] {
        let mut buf = [0u8; 13];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8] = self.version as u8;
        buf[9..13].copy_from_slice(&self.flags.bits().to_be_bytes());
        buf
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CapsuleError> {
        if data.len() < 13 {
            return Err(CapsuleError::InvalidMagic);
        }
        if &data[0..8] != BNPMCAP1_MAGIC {
            return Err(CapsuleError::InvalidMagic);
        }
        let version = data[8] as u16;
        if version != 1 {
            return Err(CapsuleError::UnsupportedVersion(version));
        }
        let flags_bits = u32::from_be_bytes([data[9], data[10], data[11], data[12]]);
        let flags = CapsuleFlags::from_bits(flags_bits);
        if !flags.compressed() {
            return Err(CapsuleError::InvalidFlags);
        }
        Ok(Self { magic: *BNPMCAP1_MAGIC, version, flags })
    }
}

// ── Canonical CBOR helpers ─────────────────────────────────────────────────

fn sort_cbor_map(map: &mut [(Value, Value)]) {
    map.sort_by(|a, b| {
        let a_key = cbor_value_bytes(&a.0);
        let b_key = cbor_value_bytes(&b.0);
        a_key.cmp(&b_key)
    });
}

fn cbor_value_bytes(val: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(val, &mut buf).unwrap();
    buf
}

/// Encode a manifest as canonical CBOR with sorted map keys.
fn encode_manifest_cbor(manifest: &Value) -> Result<Vec<u8>, CapsuleError> {
    let sorted = sort_cbor_map_recursive(manifest);
    let mut buf = Vec::new();
    ciborium::ser::into_writer(&sorted, &mut buf)
        .map_err(|e| CapsuleError::CorruptManifest(e.to_string()))?;
    Ok(buf)
}

fn sort_cbor_map_recursive(value: &Value) -> Value {
    match value {
        Value::Map(entries) => {
            let mut sorted: Vec<(Value, Value)> = entries
                .iter()
                .map(|(k, v)| (sort_cbor_map_recursive(k), sort_cbor_map_recursive(v)))
                .collect();
            sorted.sort_by(|a, b| {
                let a_bytes = cbor_value_bytes(&a.0);
                let b_bytes = cbor_value_bytes(&b.0);
                a_bytes.cmp(&b_bytes)
            });
            Value::Map(sorted)
        },
        Value::Array(items) => Value::Array(items.iter().map(sort_cbor_map_recursive).collect()),
        other => other.clone(),
    }
}

// ── Capsule ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Capsule {
    header: CapsuleHeader,
    manifest_cbor: Vec<u8>,
    file_table: FileTable,
    file_table_cbor: Vec<u8>,
    file_objects_cbor: Vec<u8>,
    attestations: Vec<Attestation>,
}

impl Capsule {
    /// Pack source files into a BNPMCAP1 capsule with an empty manifest.
    pub fn pack(files: Vec<(String, Vec<u8>)>) -> Result<Self, CapsuleError> {
        let empty_manifest = encode_manifest_cbor(&Value::Map(Vec::new()))?;
        Self::pack_with_manifest(empty_manifest, files)
    }

    /// Pack source files into a BNPMCAP1 capsule with the given canonical CBOR manifest.
    ///
    /// The caller is responsible for producing canonical manifest CBOR (e.g. via
    /// `bnpm-manifest`). The manifest bytes are validated as well-formed CBOR
    /// before packing.
    pub fn pack_with_manifest(
        manifest_cbor: Vec<u8>,
        files: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, CapsuleError> {
        // Validate manifest_cbor is well-formed CBOR
        {
            let mut cursor = Cursor::new(&manifest_cbor);
            ciborium::de::from_reader::<Value, _>(&mut cursor)
                .map_err(|e| CapsuleError::CorruptManifest(e.to_string()))?;
        }

        let mut sorted: Vec<(String, Vec<u8>)> = files;
        sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let mut entries = Vec::with_capacity(sorted.len());
        for (path, data) in &sorted {
            let entry = FileEntry::new(path.clone(), data);
            entries.push(entry);
        }

        let mut compressed_chunks: Vec<Vec<u8>> = Vec::with_capacity(sorted.len());
        for (_, data) in &sorted {
            let compressed = zstd::encode_all(&data[..], COMPRESSION_LEVEL)
                .map_err(|e| CapsuleError::CompressionError(e.to_string()))?;
            compressed_chunks.push(compressed);
        }

        let mut running_offset: u64 = 0;
        let mut file_objects = Vec::new();
        for (i, chunk) in compressed_chunks.iter().enumerate() {
            entries[i].offset = running_offset;
            entries[i].compressed_size = chunk.len() as u64;
            file_objects.extend_from_slice(chunk);
            running_offset += chunk.len() as u64;
        }

        let file_objects_cbor = {
            let mut buf = Vec::new();
            ciborium::ser::into_writer(&Value::from(file_objects.clone()), &mut buf)
                .map_err(|e| CapsuleError::CorruptFileObjects(e.to_string()))?;
            buf
        };

        let file_table = FileTable::new(entries);
        let file_table_cbor = file_table.to_cbor()?;

        let mut flags = CapsuleFlags::new();
        flags.set_compressed(true);
        if !sorted.is_empty() {
            flags.set_has_attestations(false);
        }

        let header = CapsuleHeader::new(flags);

        Ok(Self {
            header,
            manifest_cbor,
            file_table,
            file_table_cbor,
            file_objects_cbor,
            attestations: Vec::new(),
        })
    }

    /// Unpack a capsule, returning raw bytes grouped by file.
    pub fn unpack(&self) -> Result<Vec<(String, Vec<u8>)>, CapsuleError> {
        let zstd_data = extract_bstr_payload(&self.file_objects_cbor)?;
        let cursor = Cursor::new(zstd_data);
        let mut decoder = zstd::Decoder::new(cursor)
            .map_err(|e| CapsuleError::CompressionError(e.to_string()))?;
        let mut results = Vec::with_capacity(self.file_table.entries().len());
        for entry in self.file_table.entries() {
            let mut buf = vec![0u8; entry.size as usize];
            decoder
                .read_exact(&mut buf)
                .map_err(|e| CapsuleError::CorruptFileObjects(e.to_string()))?;
            results.push((entry.path.clone(), buf));
        }
        Ok(results)
    }

    /// Compute the content CID (hash of capsule EXCLUDING attestations).
    pub fn cid(&self) -> Cid {
        let canonical = self.canonical_bytes();
        Cid::new(&canonical)
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.manifest_cbor);
        buf.extend_from_slice(&self.file_table_cbor);
        buf.extend_from_slice(&self.file_objects_cbor);
        buf
    }

    /// Serialize capsule to bytes (with optional attestations).
    pub fn to_bytes(&self) -> Result<Vec<u8>, CapsuleError> {
        let mut buf = self.canonical_bytes();
        if self.attestations.is_empty() {
            let empty_arr = Value::Array(Vec::new());
            ciborium::ser::into_writer(&empty_arr, &mut buf)
                .map_err(|e| CapsuleError::CorruptAttestations(e.to_string()))?;
        } else {
            let att_cbor = attestations_to_cbor(&self.attestations)?;
            buf.extend_from_slice(&att_cbor);
        }
        Ok(buf)
    }

    /// Deserialize capsule from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, CapsuleError> {
        if data.len() < 13 {
            return Err(CapsuleError::InvalidMagic);
        }

        let header = CapsuleHeader::from_bytes(&data[..13])?;

        let mut offset = 13usize;

        let manifest_cbor = {
            let mut cursor = Cursor::new(&data[offset..]);
            let _: Value = ciborium::de::from_reader(&mut cursor)
                .map_err(|e| CapsuleError::CorruptManifest(e.to_string()))?;
            let consumed = cursor.position() as usize;
            let bytes = data[offset..offset + consumed].to_vec();
            offset += consumed;
            bytes
        };

        let (file_table, file_table_cbor) = {
            let mut cursor = Cursor::new(&data[offset..]);
            let val: Value = ciborium::de::from_reader(&mut cursor)
                .map_err(|e| CapsuleError::CorruptFileTable(e.to_string()))?;
            let consumed = cursor.position() as usize;
            let cbor = data[offset..offset + consumed].to_vec();
            let ft = FileTable::from_cbor(&val)?;
            offset += consumed;
            (ft, cbor)
        };

        let file_objects_cbor = {
            let mut cursor = Cursor::new(&data[offset..]);
            let _: Value = ciborium::de::from_reader(&mut cursor)
                .map_err(|e| CapsuleError::CorruptFileObjects(e.to_string()))?;
            let consumed = cursor.position() as usize;
            let bytes = data[offset..offset + consumed].to_vec();
            offset += consumed;
            bytes
        };

        let attestations = if offset < data.len() {
            let mut cursor = Cursor::new(&data[offset..]);
            let val: Value = ciborium::de::from_reader(&mut cursor)
                .map_err(|e| CapsuleError::CorruptAttestations(e.to_string()))?;
            attestations_from_cbor(&val)?
        } else {
            Vec::new()
        };

        Ok(Self {
            header,
            manifest_cbor,
            file_table,
            file_table_cbor,
            file_objects_cbor,
            attestations,
        })
    }

    /// Add/replace attestations (doesn't change CID).
    pub fn with_attestations(mut self, attestations: Vec<Attestation>) -> Self {
        self.attestations = attestations;
        self
    }

    /// Get file table.
    pub fn file_table(&self) -> &FileTable {
        &self.file_table
    }

    /// Get the raw canonical CBOR manifest bytes.
    pub fn manifest_cbor(&self) -> &[u8] {
        &self.manifest_cbor
    }

    /// Extract range of file objects.
    pub fn read_range(
        &self,
        path: &str,
        offset: u64,
        length: u64,
    ) -> Result<Vec<u8>, CapsuleError> {
        let entry = self
            .file_table
            .0
            .iter()
            .find(|e| e.path == path)
            .ok_or_else(|| CapsuleError::IoError(format!("file not found: {}", path)))?;

        if offset + length > entry.size {
            return Err(CapsuleError::IoError("range out of bounds".into()));
        }

        let zstd_data = extract_bstr_payload(&self.file_objects_cbor)?;
        let zstd_slice = &zstd_data
            [entry.offset as usize..entry.offset as usize + entry.compressed_size as usize];
        let decompressed = zstd::decode_all(zstd_slice)
            .map_err(|e| CapsuleError::CompressionError(e.to_string()))?;

        Ok(decompressed[offset as usize..(offset + length) as usize].to_vec())
    }
}

fn extract_bstr_payload(cbor_bytes: &[u8]) -> Result<Vec<u8>, CapsuleError> {
    let mut cursor = Cursor::new(cbor_bytes);
    let val: Value = ciborium::de::from_reader(&mut cursor)
        .map_err(|e| CapsuleError::CorruptFileObjects(e.to_string()))?;
    val.as_bytes()
        .cloned()
        .ok_or_else(|| CapsuleError::CorruptFileObjects("expected byte string".into()))
}

// ── CapsuleError ───────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CapsuleError {
    #[error("invalid BNPMCAP1 magic bytes")]
    InvalidMagic,
    #[error("unsupported BNPMCAP1 version: {0}")]
    UnsupportedVersion(u16),
    #[error("invalid capsule flags")]
    InvalidFlags,
    #[error("corrupt manifest: {0}")]
    CorruptManifest(String),
    #[error("corrupt file table: {0}")]
    CorruptFileTable(String),
    #[error("corrupt file objects: {0}")]
    CorruptFileObjects(String),
    #[error("corrupt attestations: {0}")]
    CorruptAttestations(String),
    #[error("compression error: {0}")]
    CompressionError(String),
    #[error("I/O error: {0}")]
    IoError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_files() -> Vec<(String, Vec<u8>)> {
        vec![
            ("hello.txt".to_string(), b"Hello, World!".to_vec()),
            ("foo/bar.txt".to_string(), b"bar content".to_vec()),
            ("a.txt".to_string(), b"alpha".to_vec()),
        ]
    }

    #[test]
    fn test_roundtrip_empty() {
        let capsule = Capsule::pack(vec![]).unwrap();
        let bytes = capsule.to_bytes().unwrap();
        let restored = Capsule::from_bytes(&bytes).unwrap();
        let unpacked = restored.unpack().unwrap();
        assert!(unpacked.is_empty());
    }

    #[test]
    fn test_roundtrip_single_file() {
        let files = vec![("hello.txt".to_string(), b"Hello, World!".to_vec())];
        let capsule = Capsule::pack(files.clone()).unwrap();
        let bytes = capsule.to_bytes().unwrap();
        let restored = Capsule::from_bytes(&bytes).unwrap();
        let unpacked = restored.unpack().unwrap();
        assert_eq!(unpacked.len(), 1);
        assert_eq!(unpacked[0].0, "hello.txt");
        assert_eq!(unpacked[0].1, b"Hello, World!");
    }

    #[test]
    fn test_multiple_files_sorted() {
        let files = test_files();
        // Input is NOT sorted: hello.txt, foo/bar.txt, a.txt
        // After pack, should be sorted: a.txt, foo/bar.txt, hello.txt
        let capsule = Capsule::pack(files).unwrap();
        let paths: Vec<&str> =
            capsule.file_table().entries().iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["a.txt", "foo/bar.txt", "hello.txt"]);
    }

    #[test]
    fn test_determinism() {
        let files = test_files();
        let capsule1 = Capsule::pack(files.clone()).unwrap();
        let capsule2 = Capsule::pack(files).unwrap();
        let bytes1 = capsule1.to_bytes().unwrap();
        let bytes2 = capsule2.to_bytes().unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_cid_format() {
        let capsule = Capsule::pack(vec![]).unwrap();
        let cid = capsule.cid();
        let s = cid.to_string();
        assert!(s.starts_with("b3:"), "CID should start with b3:");
        // "b3:" (3) + base32(34-byte multihash) = 3 + 55 = 58
        assert_eq!(s.len(), 58);
    }

    #[test]
    fn test_attestations_detached() {
        let files = test_files();
        let capsule = Capsule::pack(files).unwrap();
        let cid_before = capsule.cid();

        let attestation = Attestation::new(
            AttestationType::InTotoStatement,
            b"payload data".to_vec(),
            b"signature bytes".to_vec(),
        );
        let capsule = capsule.with_attestations(vec![attestation]);
        let cid_after = capsule.cid();

        assert_eq!(cid_before, cid_after, "CID must not change when attestations are added");
    }

    #[test]
    fn test_file_hash_correctness() {
        let data = b"hello blake3";
        let files = vec![("test.bin".to_string(), data.to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        let entry = &capsule.file_table().entries()[0];
        let hash = blake3::hash(data);
        assert_eq!(entry.blake3_hash, *hash.as_bytes());
    }

    #[test]
    fn test_invalid_magic() {
        let err = Capsule::from_bytes(b"NOTBNPMCAP1\x01\x00\x00\x00\x01\x00\x00\x00");
        assert!(matches!(err, Err(CapsuleError::InvalidMagic)));
    }

    #[test]
    fn test_unsupported_version() {
        let mut data = vec![];
        data.extend_from_slice(b"BNPMCAP1");
        data.push(2u8); // version 2
        data.extend_from_slice(&99u32.to_be_bytes()); // flags
        let err = Capsule::from_bytes(&data);
        assert!(matches!(err, Err(CapsuleError::UnsupportedVersion(2))));
    }

    #[test]
    fn test_corrupted_data() {
        let capsule = Capsule::pack(test_files()).unwrap();
        let mut bytes = capsule.to_bytes().unwrap();
        // Corrupt a byte in the manifest
        if bytes.len() > 20 {
            bytes[14] ^= 0xff;
        }
        let err = Capsule::from_bytes(&bytes);
        assert!(err.is_err());
    }

    #[test]
    fn test_read_range() {
        let content = b"this is a test file with enough content to read a range from".to_vec();
        let files = vec![("data.txt".to_string(), content)];
        let capsule = Capsule::pack(files).unwrap();
        let range = capsule.read_range("data.txt", 5, 7).unwrap();
        assert_eq!(range, b"is a te");
    }

    #[test]
    fn test_read_range_out_of_bounds() {
        let files = vec![("x.txt".to_string(), b"short".to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        let err = capsule.read_range("x.txt", 10, 5);
        assert!(err.is_err());
    }

    #[test]
    fn test_read_range_nonexistent_file() {
        let capsule = Capsule::pack(vec![]).unwrap();
        let err = capsule.read_range("noexist", 0, 1);
        assert!(err.is_err());
    }

    #[test]
    fn test_mode_class_default() {
        let files = vec![("f".to_string(), b"x".to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        assert_eq!(capsule.file_table().entries()[0].mode_class, ModeClass::Regular);
    }

    #[test]
    fn test_header_flags() {
        let capsule = Capsule::pack(test_files()).unwrap();
        assert!(capsule.header.flags.compressed());
        assert!(!capsule.header.flags.has_attestations());
        assert!(!capsule.header.flags.signed());
        assert!(!capsule.header.flags.range_index());

        let bytes = capsule.to_bytes().unwrap();
        let flags_bytes = u32::from_be_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
        assert_eq!(flags_bytes & 1, 1); // compressed bit set
    }

    #[test]
    fn test_empty_file() {
        let files = vec![("empty.txt".to_string(), Vec::new())];
        let capsule = Capsule::pack(files).unwrap();
        let unpacked = capsule.unpack().unwrap();
        assert_eq!(unpacked[0].1, Vec::<u8>::new());
    }

    #[test]
    fn test_binary_content() {
        let content: Vec<u8> = (0u8..=255).collect();
        let files = vec![("binary.bin".to_string(), content.clone())];
        let capsule = Capsule::pack(files).unwrap();
        let unpacked = capsule.unpack().unwrap();
        assert_eq!(unpacked[0].1, content);
    }

    #[test]
    fn test_large_file() {
        let content = vec![0xABu8; 100_000];
        let files = vec![("large.bin".to_string(), content.clone())];
        let capsule = Capsule::pack(files).unwrap();
        let unpacked = capsule.unpack().unwrap();
        assert_eq!(unpacked[0].1, content);
    }

    #[test]
    fn test_cid_determinism() {
        let files = test_files();
        let c1 = Capsule::pack(files.clone()).unwrap().cid();
        let c2 = Capsule::pack(files).unwrap().cid();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_cid_changes_with_content() {
        let f1 = vec![("a.txt".to_string(), b"hello".to_vec())];
        let f2 = vec![("a.txt".to_string(), b"world".to_vec())];
        let c1 = Capsule::pack(f1).unwrap().cid();
        let c2 = Capsule::pack(f2).unwrap().cid();
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_invalid_flags_no_compression() {
        let header = CapsuleHeader::from_bytes(b"BNPMCAP1\x01\x00\x00\x00\x00");
        assert!(matches!(header, Err(CapsuleError::InvalidFlags)));
    }

    #[test]
    fn test_attestation_roundtrip() {
        let a = Attestation::new(AttestationType::SbomSpdx, b"sbom data".to_vec(), b"sig".to_vec())
            .with_media_type("application/spdx+json".to_string())
            .with_signer("did:example:123".to_string());

        let capsule = Capsule::pack(test_files()).unwrap().with_attestations(vec![a]);

        let bytes = capsule.to_bytes().unwrap();
        let restored = Capsule::from_bytes(&bytes).unwrap();
        assert_eq!(restored.attestations.len(), 1);
        assert_eq!(restored.attestations[0].type_, AttestationType::SbomSpdx);
        assert_eq!(restored.attestations[0].payload, b"sbom data");
        assert_eq!(restored.attestations[0].signature, b"sig");
        assert_eq!(restored.attestations[0].media_type, Some("application/spdx+json".to_string()));
        assert_eq!(restored.attestations[0].signer, Some("did:example:123".to_string()));
    }

    #[test]
    fn test_unicode_paths() {
        let files = vec![
            ("résumé.md".to_string(), b"content".to_vec()),
            ("数据.txt".to_string(), b"data".to_vec()),
        ];
        let capsule = Capsule::pack(files.clone()).unwrap();
        let unpacked = capsule.unpack().unwrap();
        assert_eq!(unpacked.len(), 2);
        // Sorted by UTF-8 byte order: 'r' (0x72) < 0xe6 (start of 数)
        assert_eq!(unpacked[0].0, "résumé.md");
        assert_eq!(unpacked[1].0, "数据.txt");
    }

    #[test]
    fn test_from_bytes_validates_version() {
        let mut data = vec![];
        data.extend_from_slice(b"BNPMCAP1");
        data.push(0xFFu8); // version 255
        data.extend_from_slice(&1u32.to_be_bytes());
        let err = Capsule::from_bytes(&data);
        assert!(matches!(err, Err(CapsuleError::UnsupportedVersion(255))));
    }

    #[test]
    fn test_file_table_cbor_roundtrip() {
        let files = test_files();
        let capsule = Capsule::pack(files).unwrap();
        let cbor = capsule.file_table_cbor.clone();
        let val: Value = ciborium::de::from_reader(Cursor::new(&cbor)).unwrap();
        let ft = FileTable::from_cbor(&val).unwrap();
        assert_eq!(ft.entries().len(), 3);
        assert_eq!(ft.entries()[0].path, "a.txt");
    }

    fn non_empty_manifest_cbor() -> Vec<u8> {
        let map = Value::Map(vec![
            (Value::from("name"), Value::from("test-pkg")),
            (Value::from("version"), Value::from("1.0.0")),
        ]);
        encode_manifest_cbor(&map).unwrap()
    }

    fn other_manifest_cbor() -> Vec<u8> {
        let map = Value::Map(vec![
            (Value::from("name"), Value::from("other-pkg")),
            (Value::from("version"), Value::from("2.0.0")),
        ]);
        encode_manifest_cbor(&map).unwrap()
    }

    #[test]
    fn test_pack_with_manifest_roundtrip() {
        let manifest_cbor = non_empty_manifest_cbor();
        let files = test_files();
        let capsule = Capsule::pack_with_manifest(manifest_cbor.clone(), files).unwrap();
        let bytes = capsule.to_bytes().unwrap();
        let restored = Capsule::from_bytes(&bytes).unwrap();
        assert_eq!(restored.manifest_cbor(), manifest_cbor.as_slice());
        assert_eq!(restored.cid(), capsule.cid());
    }

    #[test]
    fn test_pack_with_manifest_determinism() {
        let manifest_cbor = non_empty_manifest_cbor();
        let files = test_files();
        let c1 = Capsule::pack_with_manifest(manifest_cbor.clone(), files.clone()).unwrap();
        let c2 = Capsule::pack_with_manifest(manifest_cbor, files).unwrap();
        assert_eq!(c1.to_bytes().unwrap(), c2.to_bytes().unwrap());
        assert_eq!(c1.cid(), c2.cid());
    }

    #[test]
    fn test_pack_with_manifest_sensitivity() {
        let files = test_files();
        let c1 = Capsule::pack_with_manifest(non_empty_manifest_cbor(), files.clone()).unwrap();
        let c2 = Capsule::pack_with_manifest(other_manifest_cbor(), files).unwrap();
        assert_ne!(c1.cid(), c2.cid());
        assert_ne!(c1.to_bytes().unwrap(), c2.to_bytes().unwrap());
    }

    #[test]
    fn test_pack_with_manifest_invalid_cbor() {
        let invalid = vec![0xFF, 0xFF, 0xFF];
        let err = Capsule::pack_with_manifest(invalid, vec![]);
        assert!(matches!(err, Err(CapsuleError::CorruptManifest(_))));
    }

    #[test]
    fn test_manifest_cbor_accessor() {
        let manifest_cbor = non_empty_manifest_cbor();
        let capsule = Capsule::pack_with_manifest(manifest_cbor.clone(), vec![]).unwrap();
        assert_eq!(capsule.manifest_cbor(), manifest_cbor.as_slice());
    }
}
