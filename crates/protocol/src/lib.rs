//! Wire protocol types and serialization for registry communication.
//!
//! Defines the message format used between Bnpm clients and registries,
//! including framing, message types, and serialization helpers.

use std::time::SystemTime;

/// Error type for protocol operations.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("deserialization error: {0}")]
    Deserialize(String),
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u32),
    #[error("invalid message signature")]
    InvalidSignature,
    #[error("message too large: {0} bytes")]
    MessageTooLarge(usize),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Protocol magic bytes for identifying Bnpm messages.
pub const PROTOCOL_MAGIC: &[u8; 4] = b"BNPM";

/// Current protocol version.
pub const PROTOCOL_VERSION: u32 = 1;

/// The maximum allowed message payload size (100 MB).
pub const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

/// Types of messages in the Bnpm wire protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessageType {
    /// A request message (client → server).
    Request,
    /// A response message (server → client).
    Response,
    /// A streaming message (chunked data).
    Stream,
    /// An error message.
    Error,
    /// A heartbeat / keepalive message.
    Heartbeat,
}

impl MessageType {
    /// Return the message type as a byte.
    pub fn as_u8(&self) -> u8 {
        match self {
            MessageType::Request => 0x01,
            MessageType::Response => 0x02,
            MessageType::Stream => 0x03,
            MessageType::Error => 0x04,
            MessageType::Heartbeat => 0x05,
        }
    }

    /// Parse a message type from a byte.
    pub fn from_u8(byte: u8) -> Result<Self, ProtocolError> {
        match byte {
            0x01 => Ok(MessageType::Request),
            0x02 => Ok(MessageType::Response),
            0x03 => Ok(MessageType::Stream),
            0x04 => Ok(MessageType::Error),
            0x05 => Ok(MessageType::Heartbeat),
            _ => Err(ProtocolError::Deserialize(format!("unknown message type: {byte}"))),
        }
    }
}

/// A protocol message with framing and optional signature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtocolMessage {
    /// Magic bytes identifying this as a Bnpm protocol message.
    pub magic: [u8; 4],
    /// Protocol version.
    pub version: u32,
    /// Message type.
    pub message_type: MessageType,
    /// Message payload (serialized JSON or protobuf).
    pub payload: Vec<u8>,
    /// Optional message ID for correlation.
    pub message_id: uuid::Uuid,
    /// Optional HMAC or signature for integrity.
    pub signature: Option<Vec<u8>>,
    /// Timestamp of message creation.
    pub timestamp: SystemTime,
}

impl ProtocolMessage {
    /// Create a new protocol message.
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            magic: *PROTOCOL_MAGIC,
            version: PROTOCOL_VERSION,
            message_type,
            payload,
            message_id: uuid::Uuid::new_v4(),
            signature: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a request message.
    pub fn request(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Request, payload)
    }

    /// Create a response message.
    pub fn response(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Response, payload)
    }

    /// Create an error message.
    pub fn error(error_code: &str, error_message: &str) -> Self {
        let payload = serde_json::json!({
            "error": error_code,
            "message": error_message,
        });
        Self::new(
            MessageType::Error,
            serde_json::to_vec(&payload).unwrap_or_default(),
        )
    }

    /// Validate the message header.
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.magic != *PROTOCOL_MAGIC {
            return Err(ProtocolError::InvalidMagic);
        }
        if self.version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(self.version));
        }
        if self.payload.len() > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(self.payload.len()));
        }
        Ok(())
    }

    /// Sign this message with the given key.
    pub fn sign(&mut self, key: &[u8]) {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key");
        mac.update(&self.magic);
        mac.update(&self.version.to_be_bytes());
        mac.update(&[self.message_type.as_u8()]);
        mac.update(&self.payload);
        mac.update(self.message_id.as_bytes());

        self.signature = Some(mac.finalize().into_bytes().to_vec());
    }

    /// Verify the message signature.
    pub fn verify_signature(&self, key: &[u8]) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let signature = match &self.signature {
            Some(s) => s,
            None => return true, // No signature to verify
        };

        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key");
        mac.update(&self.magic);
        mac.update(&self.version.to_be_bytes());
        mac.update(&[self.message_type.as_u8()]);
        mac.update(&self.payload);
        mac.update(self.message_id.as_bytes());

        mac.verify_slice(signature).is_ok()
    }

    /// Serialize this message to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(self).map_err(|e| ProtocolError::Serialize(e.to_string()))
    }

    /// Deserialize a message from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        let msg: Self =
            serde_json::from_slice(data).map_err(|e| ProtocolError::Deserialize(e.to_string()))?;
        msg.validate()?;
        Ok(msg)
    }

    /// Encode the payload as a protobuf-like binary format using prost.
    pub fn encode_protobuf<T: prost::Message>(payload: &T) -> Vec<u8> {
        payload.encode_to_vec()
    }

    /// Decode the payload from protobuf format.
    pub fn decode_protobuf<T: prost::Message + Default>(data: &[u8]) -> Result<T, ProtocolError> {
        T::decode(data).map_err(|e| ProtocolError::Deserialize(e.to_string()))
    }
}

impl std::fmt::Display for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProtoMsg({:?}, id={}, len={})",
            self.message_type,
            self.message_id,
            self.payload.len()
        )
    }
}

/// Status codes for protocol responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    Conflict = 409,
    TooManyRequests = 429,
    InternalError = 500,
    ServiceUnavailable = 503,
}

impl StatusCode {
    /// Create from a u16.
    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            200 => Some(StatusCode::Ok),
            201 => Some(StatusCode::Created),
            204 => Some(StatusCode::NoContent),
            400 => Some(StatusCode::BadRequest),
            401 => Some(StatusCode::Unauthorized),
            403 => Some(StatusCode::Forbidden),
            404 => Some(StatusCode::NotFound),
            409 => Some(StatusCode::Conflict),
            429 => Some(StatusCode::TooManyRequests),
            500 => Some(StatusCode::InternalError),
            503 => Some(StatusCode::ServiceUnavailable),
            _ => None,
        }
    }

    /// Check if this is a success status.
    pub fn is_success(&self) -> bool {
        matches!(self, StatusCode::Ok | StatusCode::Created | StatusCode::NoContent)
    }

    /// Check if this is a client error.
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            StatusCode::BadRequest
                | StatusCode::Unauthorized
                | StatusCode::Forbidden
                | StatusCode::NotFound
                | StatusCode::Conflict
                | StatusCode::TooManyRequests
        )
    }
}

/// Serialization format for protocol messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    Protobuf,
    Cbor,
}

/// Helper to serialize a value to JSON bytes.
pub fn to_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, ProtocolError> {
    serde_json::to_vec(value).map_err(|e| ProtocolError::Serialize(e.to_string()))
}

/// Helper to deserialize a value from JSON bytes.
pub fn from_json<'a, T: serde::Deserialize<'a>>(data: &'a [u8]) -> Result<T, ProtocolError> {
    serde_json::from_slice(data).map_err(|e| ProtocolError::Deserialize(e.to_string()))
}

// Re-export prost for protocol implementations
pub use prost;

/// Helper to create a tonic (gRPC) service trait for registry operations.
/// This module provides placeholder types for tonic-based communication.
pub mod grpc {
    use super::*;

    /// A generic gRPC request wrapper.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct GrpcRequest<T> {
        pub message_id: uuid::Uuid,
        pub payload: T,
        pub metadata: std::collections::HashMap<String, String>,
    }

    /// A generic gRPC response wrapper.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct GrpcResponse<T> {
        pub message_id: uuid::Uuid,
        pub payload: T,
        pub status: StatusCode,
    }

    impl<T> GrpcRequest<T> {
        /// Wrap a payload in a gRPC request.
        pub fn new(payload: T) -> Self {
            Self {
                message_id: uuid::Uuid::new_v4(),
                payload,
                metadata: std::collections::HashMap::new(),
            }
        }
    }

    impl<T> GrpcResponse<T> {
        /// Wrap a payload in a successful gRPC response.
        pub fn ok(payload: T) -> Self {
            Self {
                message_id: uuid::Uuid::new_v4(),
                payload,
                status: StatusCode::Ok,
            }
        }

        /// Wrap an error in a gRPC response.
        pub fn error(status: StatusCode, payload: T) -> Self {
            Self {
                message_id: uuid::Uuid::new_v4(),
                payload,
                status,
            }
        }
    }
}

use hmac::{Hmac, Mac};
use sha2::Sha256;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_message_new() {
        let payload = b"test payload".to_vec();
        let msg = ProtocolMessage::new(MessageType::Request, payload.clone());
        assert_eq!(msg.magic, *PROTOCOL_MAGIC);
        assert_eq!(msg.version, PROTOCOL_VERSION);
        assert_eq!(msg.message_type, MessageType::Request);
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_protocol_message_validate() {
        let msg = ProtocolMessage::new(MessageType::Request, vec![1, 2, 3]);
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_protocol_message_invalid_magic() {
        let mut msg = ProtocolMessage::new(MessageType::Request, vec![]);
        msg.magic = [0, 0, 0, 0];
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_protocol_message_sign_verify() {
        let mut msg = ProtocolMessage::new(MessageType::Request, b"sensitive data".to_vec());
        let key = b"secret-key-12345";
        msg.sign(key);
        assert!(msg.signature.is_some());
        assert!(msg.verify_signature(key));
        assert!(!msg.verify_signature(b"wrong-key"));
    }

    #[test]
    fn test_protocol_message_roundtrip() {
        let msg = ProtocolMessage::new(MessageType::Response, b"response data".to_vec());
        let bytes = msg.to_bytes().unwrap();
        let decoded = ProtocolMessage::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.payload, msg.payload);
        assert_eq!(decoded.message_type, msg.message_type);
        assert_eq!(decoded.message_id, msg.message_id);
    }

    #[test]
    fn test_message_type_u8() {
        assert_eq!(MessageType::from_u8(0x01).unwrap(), MessageType::Request);
        assert_eq!(MessageType::from_u8(0x02).unwrap(), MessageType::Response);
        assert_eq!(MessageType::from_u8(0x03).unwrap(), MessageType::Stream);
        assert_eq!(MessageType::from_u8(0x04).unwrap(), MessageType::Error);
        assert!(MessageType::from_u8(0xFF).is_err());
    }

    #[test]
    fn test_status_code() {
        assert!(StatusCode::from_u16(200).unwrap().is_success());
        assert!(StatusCode::from_u16(400).unwrap().is_client_error());
        assert_eq!(StatusCode::from_u16(500), Some(StatusCode::InternalError));
        assert_eq!(StatusCode::from_u16(999), None);
    }

    #[test]
    fn test_grpc_wrappers() {
        let req = grpc::GrpcRequest::new(42);
        assert_eq!(req.payload, 42);

        let resp: grpc::GrpcResponse<i32> = grpc::GrpcResponse::ok(200);
        assert_eq!(resp.status, StatusCode::Ok);
    }

    #[test]
    fn test_error_message() {
        let msg = ProtocolMessage::error("NOT_FOUND", "package not found");
        assert_eq!(msg.message_type, MessageType::Error);
        let parsed: serde_json::Value = serde_json::from_slice(&msg.payload).unwrap();
        assert_eq!(parsed["error"], "NOT_FOUND");
    }

    #[test]
    fn test_message_too_large() {
        let large_payload = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let msg = ProtocolMessage::new(MessageType::Request, large_payload);
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_display() {
        let msg = ProtocolMessage::new(MessageType::Heartbeat, vec![]);
        let display = format!("{msg}");
        assert!(display.contains("Heartbeat"));
    }
}
