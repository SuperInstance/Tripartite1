//! Billing and usage tracking
//!
//! This module provides local-first billing with cloud sync.

pub mod r#types;
pub mod client;

pub use r#types::{BillingTier, LocalLedger, UsageEvent, Balance};
pub use client::{BillingClient, CostCalculation};
