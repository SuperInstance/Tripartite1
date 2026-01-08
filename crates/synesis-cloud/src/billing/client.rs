//! Billing client
//!
//! Tracks usage and calculates costs with tier-based pricing.
//!
//! ## Billing Tiers
//!
//! - **Free**: Monthly quota with no charges until depleted
//! - **Managed**: 3% markup on Cloudflare wholesale costs
//! - **BYOK**: 30% licensing fee for bringing your own key
//!
//! ## Cost Calculation Algorithm
//!
//! 1. Determine base pricing from model (input/output per 1M tokens)
//! 2. Calculate raw cost: `(tokens_in / 1M) * input_price + (tokens_out / 1M) * output_price`
//! 3. Apply tier-specific markup (percentage of base cost)
//! 4. Round to nearest cent for final charge
//!
//! ## Performance
//!
//! - **Balance queries**: O(1) - Single RwLock read
//! - **Usage recording**: O(1) - Single RwLock write
//! - **Cost calculation**: O(1) - Simple arithmetic, no I/O
//!
//! ## Thread Safety
//!
//! Uses `Arc<RwLock<LocalLedger>>` for concurrent access.
//! Multiple readers can query balance simultaneously while writes are serialized.

use crate::billing::types::{BillingTier, UsageEvent, Balance, LocalLedger};
use crate::error::{CloudError, CloudResult};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// CONSTANTS: Billing Configuration
// ============================================================================

/// Number of tokens in pricing unit (1 million)
///
/// All model pricing is expressed as dollars per 1M tokens.
/// This constant is used to scale actual token counts to pricing units.
const TOKENS_PER_PRICING_UNIT: u64 = 1_000_000;

/// Convert dollars to cents (multiplier)
///
/// All internal calculations use cents to avoid floating-point precision issues.
/// This is multiplied after dollar-based pricing.
const CENTS_PER_DOLLAR: f64 = 100.0;

/// Default markup percentage for Managed tier (3%)
///
/// Represents the operational overhead for managed cloud infrastructure.
/// Based on Cloudflare wholesale costs plus small margin.
///
/// TODO: Use in calculate_cost() when implementing managed tier pricing
#[allow(dead_code)]
const DEFAULT_MANAGED_MARKUP_PERCENT: f64 = 3.0;

/// Licensing fee percentage for BYOK tier (30%)
///
/// Fee for users who bring their own API keys but use Synesis protocol.
/// Covers protocol licensing and infrastructure costs.
///
/// TODO: Use in calculate_cost() when implementing BYOK tier pricing
#[allow(dead_code)]
const DEFAULT_BYOK_LICENSING_PERCENT: f64 = 30.0;

/// Billing client for cost tracking and calculation
///
/// Tracks usage and calculates costs based on billing tier.
/// All billing is local-first before cloud synchronization.
///
/// # Tiers
///
/// - **Free**: Monthly quota, no charges until depleted
/// - **Managed**: 3% markup on Cloudflare wholesale costs
/// - **BYOK**: 30% licensing fee for bringing your own key
///
/// # Thread Safety
///
/// This struct is clone-safe and uses `Arc<RwLock<>>` internally
/// for thread-safe ledger access.
///
/// # Example
///
/// ```rust,no_run
/// use synesis_cloud::billing::client::BillingClient;
/// use synesis_cloud::billing::types::BillingTier;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let billing = BillingClient::new(
///     "api-key".to_string(),
///     BillingTier::Managed { markup_percent: 3.0 }
/// );
///
/// let cost = billing.calculate_cost("claude-sonnet", 1000, 500)?;
/// println!("Total cost: {}¢", cost.final_charge_cents);
/// # Ok(())
/// # }
/// ```
///
/// # Production Status
///
/// The `api_key` field is reserved for future cloud authentication.
pub struct BillingClient {
    ledger: Arc<RwLock<LocalLedger>>,
    #[allow(dead_code)]
    api_key: String,
}

impl BillingClient {
    /// Create new billing client
    pub fn new(api_key: String, tier: BillingTier) -> Self {
        Self {
            ledger: Arc::new(RwLock::new(LocalLedger {
                tier,
                ..Default::default()
            })),
            api_key,
        }
    }

    /// Record usage event
    pub async fn record_usage(&self, event: UsageEvent) -> CloudResult<()> {
        let mut ledger = self.ledger.write().await;

        // Add to unbilled
        ledger.unbilled_cents += event.net_charge_cents as u64;

        tracing::debug!(
            "Usage recorded: {} tokens, {}¢ (tier: {:?})",
            event.tokens_in + event.tokens_out,
            event.net_charge_cents,
            ledger.tier
        );

        Ok(())
    }

    /// Get current balance
    pub async fn balance(&self) -> CloudResult<Balance> {
        let ledger = self.ledger.read().await;

        Ok(Balance {
            unbilled_cents: ledger.unbilled_cents as u32,
            credits_cents: ledger.knowledge_credits_cents as u32,
            ceiling_cents: ledger.credit_ceiling_cents as u32,
            tier: ledger.tier.clone(),
            next_invoice: None, // TODO: Calculate from billing cycle
        })
    }

    /// Calculate cost for tokens
    ///
    /// # Algorithm
    ///
    /// 1. Get model pricing (input/output per 1M tokens in dollars)
    /// 2. Calculate base cost in cents:
    ///    - `input_cost = (tokens_in / 1,000,000) * input_price * 100`
    ///    - `output_cost = (tokens_out / 1,000,000) * output_price * 100`
    /// 3. Apply tier-specific markup (percentage of base cost)
    /// 4. Round to nearest cent for final charge
    ///
    /// # Performance
    ///
    /// O(1) time complexity - arithmetic operations only, no I/O.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use synesis_cloud::billing::client::BillingClient;
    /// # use synesis_cloud::billing::types::BillingTier;
    /// # let client = BillingClient::new("key".to_string(), BillingTier::Managed { markup_percent: 3.0 });
    /// // Claude Sonnet: 1K input + 500 output tokens
    /// let cost = client.calculate_cost("claude-sonnet", 1000, 500)?;
    /// // Base: ~1.05¢, Markup (3%): ~0.03¢, Final: ~1.08¢ → 1¢
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn calculate_cost(
        &self,
        model: &str,
        tokens_in: u32,
        tokens_out: u32,
    ) -> CloudResult<CostCalculation> {
        // Step 1: Get model pricing (per 1M tokens in dollars)
        let (input_price_per_1m, output_price_per_1m) = self.get_model_pricing(model)?;

        // Step 2: Calculate base cost in cents
        // Formula: (tokens / 1,000,000) * price_per_1m * 100
        let input_cost = (tokens_in as f64 / TOKENS_PER_PRICING_UNIT as f64) * input_price_per_1m * CENTS_PER_DOLLAR;
        let output_cost = (tokens_out as f64 / TOKENS_PER_PRICING_UNIT as f64) * output_price_per_1m * CENTS_PER_DOLLAR;
        let base_cost_cents = (input_cost + output_cost).round() as u32;

        // Step 3: Apply tier markup
        let (final_charge_cents, markup_cents) = {
            // Clone the tier for calculation (avoid holding lock across arithmetic)
            let tier = {
                let ledger = self.ledger.try_read()
                    .map_err(|_| CloudError::other("Ledger is currently locked, please retry"))?;
                ledger.tier.clone()
            };

            match tier {
                BillingTier::Free { .. } => {
                    // Free tier: no charge until quota depleted
                    (0, 0)
                }
                BillingTier::Managed { markup_percent } => {
                    // Managed tier: 3% markup on wholesale costs
                    let markup = (base_cost_cents as f64 * markup_percent as f64 / 100.0).round() as u32;
                    (base_cost_cents.saturating_add(markup), markup)
                }
                BillingTier::Byok { licensing_percent, .. } => {
                    // BYOK tier: 30% licensing fee
                    let licensing = (base_cost_cents as f64 * licensing_percent as f64 / 100.0).round() as u32;
                    (base_cost_cents.saturating_add(licensing), licensing)
                }
            }
        };

        Ok(CostCalculation {
            base_cost_cents,
            markup_cents,
            final_charge_cents,
        })
    }

    /// Get model pricing (per 1M tokens, in dollars)
    ///
    /// Returns tuple of (input_price_per_1m, output_price_per_1m).
    ///
    /// # Pricing Source
    ///
    /// Prices as of 2025 from Anthropic/OpenAI public pricing pages.
    /// Subject to change - update when providers change pricing.
    ///
    /// # Errors
    ///
    /// Returns validation error if model is not recognized.
    fn get_model_pricing(&self, model: &str) -> CloudResult<(f64, f64)> {
        // Model pricing: (input_per_1m_dollars, output_per_1m_dollars)
        // Source: Anthropic/OpenAI pricing as of 2025
        let pricing = match model {
            // Claude Opus: Most capable, most expensive
            "claude-opus" => (15.0, 75.0),

            // Claude Sonnet: Balanced performance/cost
            "claude-sonnet" => (3.0, 15.0),

            // GPT-4 Turbo: OpenAI's flagship model
            "gpt4-turbo" => (10.0, 30.0),

            _ => {
                return Err(CloudError::validation(format!(
                    "Unknown model '{}'. Supported models: claude-opus, claude-sonnet, gpt4-turbo",
                    model
                )))
            }
        };

        Ok(pricing)
    }

    /// Apply knowledge credits
    pub async fn apply_credits(&self, amount_cents: u64) -> CloudResult<()> {
        let mut ledger = self.ledger.write().await;
        ledger.knowledge_credits_cents += amount_cents;

        tracing::info!("Credits applied: {}¢", amount_cents);

        Ok(())
    }

    /// Get ledger
    pub async fn ledger(&self) -> LocalLedger {
        self.ledger.read().await.clone()
    }
}

/// Cost calculation result
#[derive(Debug, Clone)]
pub struct CostCalculation {
    /// Base cost before markup (in cents)
    pub base_cost_cents: u32,
    /// Markup or licensing fee (in cents)
    pub markup_cents: u32,
    /// Final charge including markup (in cents)
    pub final_charge_cents: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_test_client() -> BillingClient {
        BillingClient::new(
            "test-key".to_string(),
            BillingTier::Managed { markup_percent: 3.0 },
        )
    }

    #[test]
    fn test_calculate_cost_sonnet() {
        let client = make_test_client();

        let calc = client.calculate_cost("claude-sonnet", 1000, 500).unwrap();

        // Input: 1K tokens @ $3/1M = $0.003 = 0.3¢
        // Output: 0.5K tokens @ $15/1M = $0.0075 = 0.75¢
        // Base: 1.05¢
        // Markup (3%): 0.0315¢
        // Final: ~1.08¢
        assert_eq!(calc.base_cost_cents, 1);
        assert_eq!(calc.final_charge_cents, 1);
    }

    #[test]
    fn test_calculate_cost_opus() {
        let client = make_test_client();

        let calc = client.calculate_cost("claude-opus", 10_000, 5_000).unwrap();

        // Input: 10K tokens @ $15/1M = $0.15 = 15¢
        // Output: 5K tokens @ $75/1M = $0.375 = 37.5¢
        // Base: 52.5¢ → 53¢ (rounded)
        // Markup (3%): 53 * 0.03 = 1.59¢ → 2¢ (rounded)
        // Final: 55¢
        assert_eq!(calc.base_cost_cents, 53);
        assert_eq!(calc.markup_cents, 2);
        assert_eq!(calc.final_charge_cents, 55);
    }

    #[test]
    fn test_calculate_cost_invalid_model() {
        let client = make_test_client();

        let result = client.calculate_cost("unknown-model", 1000, 500);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_usage() {
        let client = make_test_client();

        let event = UsageEvent {
            id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            tokens_in: 1000,
            tokens_out: 500,
            model: "claude-sonnet".to_string(),
            cost_basis_cents: 1,
            final_charge_cents: 1,
            credits_applied_cents: 0,
            net_charge_cents: 1,
        };

        assert!(client.record_usage(event).await.is_ok());

        let balance = client.balance().await.unwrap();
        assert_eq!(balance.unbilled_cents, 1);
    }

    #[tokio::test]
    async fn test_apply_credits() {
        let client = make_test_client();

        client.apply_credits(100).await.unwrap();

        let balance = client.balance().await.unwrap();
        assert_eq!(balance.credits_cents, 100);
    }

    #[test]
    fn test_billing_tier_free() {
        let client = BillingClient::new(
            "test-key".to_string(),
            BillingTier::Free { monthly_limit_cents: 1000 },
        );

        let calc = client.calculate_cost("claude-sonnet", 1000, 500).unwrap();

        // Free tier has no charge
        assert_eq!(calc.final_charge_cents, 0);
    }

    #[test]
    fn test_billing_tier_byok() {
        let client = BillingClient::new(
            "test-key".to_string(),
            BillingTier::Byok {
                licensing_percent: 30.0,
                anthropic_key: None,
                openai_key: None,
            },
        );

        let calc = client.calculate_cost("claude-sonnet", 1000, 500).unwrap();

        // BYOK has 30% licensing fee
        // Base: 1¢
        // Licensing (30%): 0.3¢
        // Final: 1.3¢ -> 1¢ (truncated)
        assert_eq!(calc.base_cost_cents, 1);
        assert_eq!(calc.final_charge_cents, 1);
    }
}
