//! Protocol message definitions
//!
//! This module defines the wire protocol for QUIC tunnel communication.

pub mod messages;
pub mod frame;

pub use messages::{
    TunnelMessage, HeartbeatData, HeartbeatAckData, EscalationRequestData,
    EscalationResponseData, StreamChunkData, StreamEndData, ErrorData,
    PrewarmSignalData, EscalationContextData, KnowledgeChunkData,
    MessageData, UserPreferencesData, TokenUsageData,
};
pub use frame::{Frame, FrameType};
