//! QUIC tunnel for secure cloud communication
//!
//! This module provides a persistent, low-latency QUIC tunnel
//! with mTLS authentication and automatic reconnection.

pub mod r#types;
pub mod tls;
pub mod endpoint;
pub mod state;
pub mod heartbeat;
pub mod reconnect;
pub mod tunnel;

pub use r#types::{TunnelConfig, TunnelState, TunnelStats};
pub use tunnel::CloudTunnel;
