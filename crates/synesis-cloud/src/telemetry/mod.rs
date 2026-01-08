//! Telemetry and device vitals collection
//!
//! This module provides heartbeat and telemetry functionality.

pub mod r#types;
pub mod vitals;

pub use r#types::{DeviceVitals, HeartbeatAck, Heartbeat};
pub use vitals::collect_device_vitals;
