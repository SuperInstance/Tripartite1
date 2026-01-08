//! Frame format for tunnel messages
//!
//! All messages sent over the QUIC tunnel use this frame format:
//! +------+----------------+------------------------+
//! | Type | Length (4B, BE) | Payload (JSON)         |
//! +------+----------------+------------------------+

use crate::error::{CloudError, CloudResult};
use crate::protocol::messages::TunnelMessage;

/// Message type byte
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FrameType {
    /// Heartbeat from client
    Heartbeat = 0x01,

    /// Heartbeat acknowledgment
    HeartbeatAck = 0x02,

    /// Escalation request
    EscalationRequest = 0x03,

    /// Escalation response
    EscalationResponse = 0x04,

    /// Stream chunk
    StreamChunk = 0x05,

    /// Stream end
    StreamEnd = 0x06,

    /// Error
    Error = 0x07,

    /// Pre-warm signal
    PrewarmSignal = 0x08,
}

impl FrameType {
    /// Convert from byte
    pub fn from_byte(b: u8) -> CloudResult<Self> {
        match b {
            0x01 => Ok(FrameType::Heartbeat),
            0x02 => Ok(FrameType::HeartbeatAck),
            0x03 => Ok(FrameType::EscalationRequest),
            0x04 => Ok(FrameType::EscalationResponse),
            0x05 => Ok(FrameType::StreamChunk),
            0x06 => Ok(FrameType::StreamEnd),
            0x07 => Ok(FrameType::Error),
            0x08 => Ok(FrameType::PrewarmSignal),
            _ => Err(CloudError::validation(format!("Invalid frame type: 0x{:02x}", b))),
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// Wire frame
///
/// Binary format: [Type: 1B][Length: 4BE][Payload: JSON]
#[derive(Debug, Clone)]
pub struct Frame {
    /// Message type byte
    pub frame_type: FrameType,
    /// Serialized message payload (JSON)
    pub payload: Vec<u8>,
}

impl Frame {
    /// Maximum frame size (10MB)
    pub const MAX_SIZE: usize = 10 * 1024 * 1024;

    /// Create a new frame
    pub fn new(frame_type: FrameType, payload: Vec<u8>) -> CloudResult<Self> {
        if payload.len() > Self::MAX_SIZE {
            return Err(CloudError::validation(format!(
                "Frame too large: {} bytes (max {})",
                payload.len(),
                Self::MAX_SIZE
            )));
        }

        Ok(Self { frame_type, payload })
    }

    /// Create frame from message
    pub fn from_message(message: TunnelMessage) -> CloudResult<Self> {
        let frame_type = match &message {
            TunnelMessage::Heartbeat(_) => FrameType::Heartbeat,
            TunnelMessage::HeartbeatAck(_) => FrameType::HeartbeatAck,
            TunnelMessage::EscalationRequest(_) => FrameType::EscalationRequest,
            TunnelMessage::EscalationResponse(_) => FrameType::EscalationResponse,
            TunnelMessage::StreamChunk(_) => FrameType::StreamChunk,
            TunnelMessage::StreamEnd(_) => FrameType::StreamEnd,
            TunnelMessage::Error(_) => FrameType::Error,
            TunnelMessage::PrewarmSignal(_) => FrameType::PrewarmSignal,
        };

        let payload = serde_json::to_vec(&message)
            .map_err(CloudError::Serialization)?;

        Self::new(frame_type, payload)
    }

    /// Encode frame to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5 + self.payload.len());

        // Type byte
        bytes.push(self.frame_type.to_byte());

        // Length (4 bytes big-endian)
        let len = self.payload.len() as u32;
        bytes.extend_from_slice(&len.to_be_bytes());

        // Payload
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    /// Decode frame from bytes
    pub fn decode(data: &[u8]) -> CloudResult<Self> {
        if data.len() < 5 {
            return Err(CloudError::validation("Frame too short (min 5 bytes)"));
        }

        let frame_type = FrameType::from_byte(data[0])?;

        let len_bytes = [data[1], data[2], data[3], data[4]];
        let payload_len = u32::from_be_bytes(len_bytes) as usize;

        if data.len() < 5 + payload_len {
            return Err(CloudError::validation(format!(
                "Incomplete frame: expected {} bytes, got {}",
                5 + payload_len,
                data.len()
            )));
        }

        let payload = data[5..5 + payload_len].to_vec();

        Ok(Self { frame_type, payload })
    }

    /// Parse payload as message
    pub fn to_message(&self) -> CloudResult<TunnelMessage> {
        serde_json::from_slice(&self.payload)
            .map_err(CloudError::Serialization)
    }

    /// Get payload length
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::messages::*;

    #[test]
    fn test_frame_type_conversion() {
        assert_eq!(FrameType::Heartbeat.to_byte(), 0x01);
        assert_eq!(FrameType::from_byte(0x01).unwrap(), FrameType::Heartbeat);
    }

    #[test]
    fn test_frame_encode_decode() {
        let frame = Frame::new(
            FrameType::Heartbeat,
            b"test payload".to_vec(),
        ).unwrap();

        let encoded = frame.encode();
        assert_eq!(encoded.len(), 5 + 12); // 5 header + 12 payload
        assert_eq!(encoded[0], 0x01);

        let decoded = Frame::decode(&encoded).unwrap();
        assert_eq!(decoded.frame_type, FrameType::Heartbeat);
        assert_eq!(decoded.payload, b"test payload");
    }

    #[test]
    fn test_frame_too_large() {
        let payload = vec![0u8; Frame::MAX_SIZE + 1];
        let result = Frame::new(FrameType::Heartbeat, payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_frame_from_message() {
        let msg = TunnelMessage::Heartbeat(HeartbeatData {
            device_id: "test".to_string(),
            timestamp: 123456,
            sequence: 1,
            vitals: serde_json::json!({}),
        });

        let frame = Frame::from_message(msg).unwrap();
        assert_eq!(frame.frame_type, FrameType::Heartbeat);

        let encoded = frame.encode();
        let decoded = Frame::decode(&encoded).unwrap();
        let recovered_msg = decoded.to_message().unwrap();

        assert!(matches!(recovered_msg, TunnelMessage::Heartbeat(_)));
    }

    #[test]
    fn test_frame_invalid_type() {
        let result = FrameType::from_byte(0xFF);
        assert!(result.is_err());
    }

    #[test]
    fn test_frame_too_short() {
        let data = vec![0x01, 0x00, 0x00, 0x00]; // Only 4 bytes
        let result = Frame::decode(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_frame_incomplete() {
        let mut data = vec![0x01, 0x00, 0x00, 0x00, 0x0A]; // Says 10 bytes payload
        data.extend_from_slice(&[0u8; 5]); // But only provides 5

        let result = Frame::decode(&data);
        assert!(result.is_err());
    }
}
