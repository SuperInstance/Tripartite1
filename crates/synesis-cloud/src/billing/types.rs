//! Billing and usage types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Billing tier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BillingTier {
    /// Free tier with monthly limit
    Free {
        /// Monthly spending limit in cents
        monthly_limit_cents: u32,
    },

    /// Managed tier: 3% markup on costs
    Managed {
        /// Markup percentage on wholesale costs
        markup_percent: f32,
    },

    /// BYOK tier: 30% licensing fee
    Byok {
        /// Licensing fee percentage
        licensing_percent: f32,
        /// Optional Anthropic API key
        anthropic_key: Option<String>,
        /// Optional OpenAI API key
        openai_key: Option<String>,
    },
}

impl Default for BillingTier {
    fn default() -> Self {
        Self::Managed { markup_percent: 3.0 }
    }
}

/// Usage event for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Unique event identifier
    pub id: String,

    /// Request ID this usage is for
    pub request_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Input tokens
    pub tokens_in: u32,

    /// Output tokens
    pub tokens_out: u32,

    /// Model used
    pub model: String,

    /// Base cost (before markup)
    pub cost_basis_cents: u32,

    /// Final cost (after markup, before credits)
    pub final_charge_cents: u32,

    /// Credits applied
    pub credits_applied_cents: u32,

    /// Net charge
    pub net_charge_cents: u32,
}

/// Local billing ledger (placeholder for Session 2.6)
#[derive(Debug, Clone, Default)]
pub struct LocalLedger {
    /// Accumulated unbilled charges in cents
    pub unbilled_cents: u64,
    /// Available knowledge credits in cents
    pub knowledge_credits_cents: u64,
    /// Maximum spending limit in cents
    pub credit_ceiling_cents: u64,
    /// Current billing tier
    pub tier: BillingTier,
}

/// Account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Unbilled charges
    pub unbilled_cents: u32,

    /// Available credits
    pub credits_cents: u32,

    /// Credit ceiling (spending limit)
    pub ceiling_cents: u32,

    /// Current tier
    pub tier: BillingTier,

    /// Next invoice date
    pub next_invoice: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_billing_tier_default() {
        let tier = BillingTier::default();
        assert!(matches!(tier, BillingTier::Managed { .. }));
    }

    #[test]
    fn test_billing_tier_markup() {
        let tier = BillingTier::Managed { markup_percent: 3.0 };
        assert!(matches!(tier, BillingTier::Managed { markup_percent: 3.0 }));
    }
}
