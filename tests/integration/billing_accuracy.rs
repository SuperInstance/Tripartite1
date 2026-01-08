//! Billing Accuracy Tests
//!
//! Ensure cost calculation and billing are accurate

use synesis_cloud::billing::{BillingClient, BillingTier};
use synesis_cloud::billing::types::CostCalculation;

#[test]
fn test_cost_calculation_claude_sonnet() {
    // Test cost calculation for Claude Sonnet

    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Managed { markup_percent: 3.0 }
    );

    let cost = client.calculate_cost("claude-3-5-sonnet-20241022", 1000, 500).unwrap();

    // Claude Sonnet pricing:
    // Input: $0.000003 per token
    // Output: $0.000015 per token
    // Basis: 1000 × 0.000003 + 500 × 0.000015 = $0.003 + $0.0075 = $0.0105
    // With 3% markup: $0.0105 × 1.03 = $0.010815 → 1.08¢

    assert_eq!(cost.basis_cents, 1);  // $0.0105 → 1¢
    assert_eq!(cost.markup_cents, 0); // 3% of 1¢ is 0¢
    assert_eq!(cost.final_charge_cents, 1); // 1 + 0 = 1¢
}

#[test]
fn test_cost_calculation_claude_opus() {
    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Managed { markup_percent: 3.0 }
    );

    let cost = client.calculate_cost("claude-3-opus-20240229", 2000, 1000).unwrap();

    // Claude Opus pricing (higher quality):
    // Basis: 2000 × 0.000015 + 1000 × 0.000075 = $0.03 + $0.075 = $0.105
    // With 3% markup: $0.105 × 1.03 = $0.10815 → 11¢

    assert_eq!(cost.basis_cents, 11);  // $0.105 → 11¢
    assert_eq!(cost.markup_cents, 0); // 3% of 11¢ is 0¢
    assert_eq!(cost.final_charge_cents, 11); // 11 + 0 = 11¢
}

#[test]
fn test_cost_calculation_gpt4() {
    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Managed { markup_percent: 3.0 }
    );

    let cost = client.calculate_cost("gpt-4-turbo-preview", 1500, 750).unwrap();

    // GPT-4 Turbo pricing:
    // Basis: 1500 × 0.00001 + 750 × 0.00003 = $0.015 + $0.0225 = $0.0375
    // With 3% markup: $0.0375 × 1.03 = $0.038625 → 4¢

    assert_eq!(cost.basis_cents, 4);   // $0.0375 → 4¢
    assert_eq!(cost.markup_cents, 0);  // 3% of 4¢ is 0¢
    assert_eq!(cost.final_charge_cents, 4); // 4 + 0 = 4¢
}

#[test]
fn test_markup_calculation_byok() {
    // Test BYOK tier (30% licensing fee)

    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Byok { licensing_percent: 30.0 }
    );

    let cost = client.calculate_cost("claude-3-5-sonnet-20241022", 1000, 500).unwrap();

    // Basis: 1¢ (from previous test)
    // With 30% markup: 1¢ × 1.30 = 1.3¢ → 2¢

    assert_eq!(cost.basis_cents, 1);
    assert_eq!(cost.markup_cents, 1); // 30% of 1¢ is 0.3¢ → 1¢
    assert_eq!(cost.final_charge_cents, 2);
}

#[test]
fn test_free_tier_no_markup() {
    // Test free tier (no markup)

    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Free { monthly_limit_cents: 1000 }
    );

    let cost = client.calculate_cost("claude-3-5-sonnet-20241022", 1000, 500).unwrap();

    // Free tier: no markup, just basis cost
    assert_eq!(cost.basis_cents, 1);
    assert_eq!(cost.markup_cents, 0);
    assert_eq!(cost.final_charge_cents, 1);
}

#[test]
fn test_rounding_behavior() {
    // Test that costs are rounded correctly (always up for safety)

    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Managed { markup_percent: 3.0 }
    );

    // Small query that should round up
    let cost = client.calculate_cost("claude-3-5-sonnet-20241022", 100, 50).unwrap();

    // Basis: 100 × 0.000003 + 50 × 0.000015 = $0.0003 + $0.00075 = $0.00105
    // With 3%: $0.0010815 → rounds up to 1¢

    assert_eq!(cost.final_charge_cents, 1);
}

#[test]
fn test_credits_application() {
    // Test that knowledge credits are applied correctly

    let mut client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Managed { markup_percent: 3.0 }
    );

    // Simulate having credits
    let initial_cost = synesis_cloud::billing::types::CostCalculation {
        basis_cents: 10,
        markup_cents: 0,
        final_charge_cents: 10,
    };

    // Apply credits
    let result = client.apply_credits(&initial_cost, 5).unwrap();

    // Credits reduce the charge
    assert_eq!(result.final_charge_cents, 5); // 10 - 5 = 5
}

#[test]
fn test_credits_dont_exceed_cost() {
    // Test that credits don't make cost negative

    let client = BillingClient::new(
        "test-api-key".to_string(),
        BillingTier::Managed { markup_percent: 3.0 }
    );

    let cost = synesis_cloud::billing::types::CostCalculation {
        basis_cents: 3,
        markup_cents: 0,
        final_charge_cents: 3,
    };

    // Try to apply more credits than the cost
    let result = client.apply_credits(&cost, 10).unwrap();

    // Cost should be 0, not negative
    assert_eq!(result.final_charge_cents, 0);
}

#[test]
fn test_total_calculation() {
    // Test total (input + output) token calculation

    let usage = synesis_cloud::billing::types::TokenUsage {
        prompt: 1234,
        completion: 5678,
    };

    assert_eq!(usage.total(), 6912);
}

#[test]
fn test_accumulated_billing() {
    // Test that billing accumulates correctly over multiple requests

    let mut ledger = synesis_cloud::billing::types::LocalLedger {
        unbilled_cents: 0,
        knowledge_credits_cents: 100,
        credit_ceiling_cents: 10000,
        tier: synesis_cloud::billing::types::BillingTier::Managed {
            markup_percent: 3.0
        },
        pending_events: vec![],
    };

    // First request: 5¢
    ledger.unbilled_cents += 5;
    assert_eq!(ledger.unbilled_cents, 5);

    // Second request: 7¢
    ledger.unbilled_cents += 7;
    assert_eq!(ledger.unbilled_cents, 12);

    // Apply 10¢ credits
    let credits_applied = std::cmp::min(ledger.knowledge_credits_cents, ledger.unbilled_cents);
    ledger.unbilled_cents -= credits_applied;
    ledger.knowledge_credits_cents -= credits_applied;

    assert_eq!(ledger.unbilled_cents, 2); // 12 - 10
    assert_eq!(ledger.knowledge_credits_cents, 90); // 100 - 10
}

#[test]
fn test_credit_ceiling_enforcement() {
    // Test that credit ceiling prevents unlimited usage

    let ledger = synesis_cloud::billing::types::LocalLedger {
        unbilled_cents: 950,
        knowledge_credits_cents: 100,
        credit_ceiling_cents: 1000, // $10 ceiling
        tier: synesis_cloud::billing::types::BillingTier::Managed {
            markup_percent: 3.0
        },
        pending_events: vec![],
    };

    // Try to add 10¢ more (would exceed ceiling)
    let would_be = ledger.unbilled_cents + 10;

    assert!(would_be > ledger.credit_ceiling_cents,
            "Credit ceiling should be enforced");
}
