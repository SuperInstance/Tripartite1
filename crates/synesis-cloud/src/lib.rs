// synesis-cloud: Cloud connectivity and QUIC tunnel for SuperInstance AI platform
//
// This crate provides:
// - QUIC tunnel with mTLS for secure cloud communication
// - Cloud escalation client for offloading queries to cloud LLMs
// - Billing client with local-first usage tracking
// - LoRA upload and hot-swap functionality
// - Collaborator system for sharing projects
// - Telemetry and heartbeat system
//
//! # Cloud Mesh Infrastructure
//!
//! This crate provides the complete cloud connectivity layer for the SuperInstance AI platform.
//! It enables secure QUIC tunnels, billing management, LoRA sharing, and collaboration features.
//!
//! ## Main Components
//!
//! - **Tunnel**: QUIC-based bidirectional communication with state machine
//! - **Escalation**: Client for offloading queries to cloud LLMs
//! - **Billing**: Cost tracking with tier-based pricing (Free/Managed/BYOK)
//! - **LoRA**: Upload, hot-swap, and cloud management of LoRA adapters
//! - **Collaborator**: Invite-based collaboration system
//! - **Telemetry**: Device vitals collection and heartbeat protocol
//! - **Protocol**: Binary message protocol for tunnel communication
//! - **Streaming**: Server-sent events for real-time responses
//!
//! ## Architecture
//!
//! The cloud mesh follows a local-first approach:
//! 1. All billing is tracked locally before cloud sync
//! 2. LoRAs can be managed locally and uploaded to cloud
//! 3. Collaborator invites are generated locally
//! 4. Escalation contexts are built locally before transmission
//!
//! ## Thread Safety
//!
//! All shared state uses `Arc<RwLock<T>>` for async-safe mutations.
//! Metrics use `Arc<AtomicU64>` for lock-free counting.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use synesis_cloud::billing::client::BillingClient;
//! use synesis_cloud::billing::types::BillingTier;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create billing client
//! let billing = BillingClient::new(
//!     "api-key".to_string(),
//!     BillingTier::Free { monthly_limit_cents: 1000 }
//! );
//!
//! // Calculate cost
//! let cost = billing.calculate_cost("claude-sonnet", 1000, 500)?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![warn(unused_imports)]

pub mod error;
pub mod tunnel;
pub mod escalation;
pub mod billing;
pub mod lora;
pub mod telemetry;
pub mod collaborator;
pub mod protocol;
pub mod streaming;

// Re-exports
pub use error::{CloudError, CloudResult};

pub mod prelude {
    //! Common imports for using synesis-cloud

    pub use crate::error::{CloudError, CloudResult};
    pub use crate::tunnel::types::TunnelConfig;
    pub use crate::escalation::types::{EscalationRequest, CloudModel};
    pub use crate::billing::types::{BillingTier, LocalLedger};
}
