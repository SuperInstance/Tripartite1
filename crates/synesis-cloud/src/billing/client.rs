//! Billing client
//!
//! Tracks usage and calculates costs

use crate::billing::types::{BillingTier, UsageEvent, Balance, LocalLedger};
use crate::error::{CloudError, CloudResult};
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub fn calculate_cost(
        &self,
        model: &str,
        tokens_in: u32,
        tokens_out: u32,
    ) -> CloudResult<CostCalculation> {
        // Base pricing (per 1M tokens)
        let (input_price_per_1m, output_price_per_1m) = self.get_model_pricing(model)?;

        // Calculate base cost
        let input_cost = (tokens_in as f64 / 1_000_000.0) * input_price_per_1m;
        let output_cost = (tokens_out as f64 / 1_000_000.0) * output_price_per_1m;
        let base_cost_cents = (input_cost + output_cost) * 100.0;

        // Round to nearest cent
        let base_cost_cents = base_cost_cents.round() as u32;

        // Apply tier markup
        let (final_charge_cents, markup_cents) = {
            // Clone the tier for calculation
            let tier = {
                let ledger = self.ledger.try_read()
                    .map_err(|_| CloudError::other("Ledger locked"))?;
                ledger.tier.clone()
            };

            match tier {
                BillingTier::Free { .. } => {
                    // Free tier: no charge up to limit
                    (0, 0)
                }
                BillingTier::Managed { markup_percent } => {
                    let markup = ((base_cost_cents as f64) * (markup_percent as f64)) / 100.0;
                    let markup = markup.round() as u32;
                    (base_cost_cents + markup, markup)
                }
                BillingTier::Byok { licensing_percent, .. } => {
                    let licensing = ((base_cost_cents as f64) * (licensing_percent as f64)) / 100.0;
                    let licensing = licensing.round() as u32;
                    (base_cost_cents + licensing, licensing)
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
    fn get_model_pricing(&self, model: &str) -> CloudResult<(f64, f64)> {
        // Pricing as of 2025 (Anthropic/OpenAI)
        let pricing = match model {
            "claude-opus" => (15.0, 75.0),
            "claude-sonnet" => (3.0, 15.0),
            "gpt4-turbo" => (10.0, 30.0),
            _ => return Err(CloudError::validation(format!("Unknown model: {}", model))),
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
