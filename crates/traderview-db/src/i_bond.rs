//! Series I Savings Bond calculator.
//!
//! US Series I savings bonds (TreasuryDirect) inflation-indexed
//! mechanics:
//!
//!   composite_rate = fixed_rate + 2 × semi_inflation
//!                    + (fixed_rate × semi_inflation)
//!   Per TreasuryDirect formula; rate is set every May 1 and Nov 1.
//!
//! Holding-period rules:
//!   - 12-month minimum lock-up (cannot redeem at all)
//!   - 1y to 5y: 3-month interest penalty on redemption
//!   - 5y+:     no penalty
//!   - 30y max: stops earning interest after 30 years
//!
//! Reports composite rate, final value after holding period,
//! interest accrued, early-withdrawal penalty (if applicable), net
//! value after penalty.
//!
//! Pure compute (assumes the current rates persist for the full hold;
//! actual I-bond rates reset every 6 months but the user supplies the
//! current snapshot for projection).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct IBondInput {
    pub purchase_amount_usd: f64,
    /// Fixed rate (set at purchase, locked for life). E.g. 1.30 = 1.30%.
    pub fixed_rate_pct: f64,
    /// Semi-annual inflation rate (resets every 6 months). E.g. 1.5 = 1.5%/6mo.
    pub semi_annual_inflation_pct: f64,
    pub holding_period_months: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct IBondReport {
    pub composite_rate_pct: f64,
    pub holding_period_months: u32,
    pub final_value_before_penalty_usd: f64,
    pub interest_earned_usd: f64,
    pub early_withdrawal_penalty_usd: f64,
    pub net_value_after_penalty_usd: f64,
    pub penalty_status: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

/// Composite rate per TreasuryDirect formula.
pub fn composite_rate(fixed_pct: f64, semi_inflation_pct: f64) -> f64 {
    // Formula in published TD documents: composite = fixed + 2*semi + fixed*semi
    // where fixed and semi are expressed as decimals.
    let f = fixed_pct / 100.0;
    let s = semi_inflation_pct / 100.0;
    let composite = f + 2.0 * s + f * s;
    (composite * 100.0).max(0.0)
}

pub fn early_withdrawal_penalty_status(months: u32) -> &'static str {
    if months < 12 { "locked" }
    else if months < 60 { "3mo_interest_penalty" }
    else { "none" }
}

pub fn compute(input: &IBondInput) -> IBondReport {
    let composite = composite_rate(input.fixed_rate_pct, input.semi_annual_inflation_pct);
    let monthly_rate = composite / 100.0 / 12.0;
    let hold_months = input.holding_period_months.min(360);  // 30y cap
    let mut bal = input.purchase_amount_usd;
    for _ in 0..hold_months {
        bal *= 1.0 + monthly_rate;
    }
    let interest = bal - input.purchase_amount_usd;
    let status = early_withdrawal_penalty_status(hold_months);
    let penalty = if status == "3mo_interest_penalty" {
        // Penalty = 3 months of interest at composite rate on current balance.
        bal * monthly_rate * 3.0
    } else { 0.0 };
    let net = if status == "locked" {
        // Bond cannot be redeemed — net value is the principal only (no access).
        input.purchase_amount_usd
    } else {
        bal - penalty
    };
    IBondReport {
        composite_rate_pct: composite,
        holding_period_months: hold_months,
        final_value_before_penalty_usd: bal,
        interest_earned_usd: interest,
        early_withdrawal_penalty_usd: penalty,
        net_value_after_penalty_usd: net.max(0.0),
        penalty_status: status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_rate_basic() {
        // Fixed 1.30%, inflation 1.97% (Nov 2024 actual): composite = 5.27%
        // (formula: 1.30 + 2*1.97 + 1.30*1.97/100 = 1.30 + 3.94 + 0.02561 = 5.2656)
        let c = composite_rate(1.30, 1.97);
        assert!((c - 5.2656).abs() < 0.01, "got {c}");
    }

    #[test]
    fn composite_rate_zero_inflation() {
        // Composite = fixed alone when inflation is 0.
        let c = composite_rate(2.0, 0.0);
        assert!((c - 2.0).abs() < 1e-6);
    }

    #[test]
    fn composite_rate_zero_fixed() {
        // Composite = 2 × semi inflation when fixed is 0.
        let c = composite_rate(0.0, 1.5);
        assert!((c - 3.0).abs() < 1e-6);
    }

    #[test]
    fn composite_rate_clamps_negative() {
        let c = composite_rate(-5.0, -3.0);
        assert!(c >= 0.0);
    }

    #[test]
    fn penalty_status_locked_under_12mo() {
        assert_eq!(early_withdrawal_penalty_status(11), "locked");
        assert_eq!(early_withdrawal_penalty_status(0), "locked");
    }

    #[test]
    fn penalty_status_3mo_between_12_and_60() {
        assert_eq!(early_withdrawal_penalty_status(12), "3mo_interest_penalty");
        assert_eq!(early_withdrawal_penalty_status(36), "3mo_interest_penalty");
        assert_eq!(early_withdrawal_penalty_status(59), "3mo_interest_penalty");
    }

    #[test]
    fn penalty_status_none_5y_plus() {
        assert_eq!(early_withdrawal_penalty_status(60), "none");
        assert_eq!(early_withdrawal_penalty_status(120), "none");
    }

    #[test]
    fn compute_basic_5y_hold() {
        let r = compute(&IBondInput {
            purchase_amount_usd: 10_000.0,
            fixed_rate_pct: 2.0,
            semi_annual_inflation_pct: 1.5,
            holding_period_months: 60,
        });
        // Composite 5.03%; 60 months ≈ 12,832 (compound monthly)
        assert!(r.composite_rate_pct > 5.0 && r.composite_rate_pct < 5.1);
        assert_eq!(r.penalty_status, "none");
        assert_eq!(r.early_withdrawal_penalty_usd, 0.0);
        assert!(r.interest_earned_usd > 2_000.0);
    }

    #[test]
    fn compute_locked_under_12mo_returns_principal() {
        let r = compute(&IBondInput {
            purchase_amount_usd: 10_000.0,
            fixed_rate_pct: 2.0,
            semi_annual_inflation_pct: 1.5,
            holding_period_months: 6,
        });
        assert_eq!(r.penalty_status, "locked");
        assert_eq!(r.net_value_after_penalty_usd, 10_000.0);
    }

    #[test]
    fn compute_3mo_penalty_applied_in_window() {
        let r = compute(&IBondInput {
            purchase_amount_usd: 10_000.0,
            fixed_rate_pct: 2.0,
            semi_annual_inflation_pct: 1.5,
            holding_period_months: 36,
        });
        assert_eq!(r.penalty_status, "3mo_interest_penalty");
        assert!(r.early_withdrawal_penalty_usd > 0.0);
        assert!(r.net_value_after_penalty_usd < r.final_value_before_penalty_usd);
    }

    #[test]
    fn compute_caps_at_30_years() {
        let r = compute(&IBondInput {
            purchase_amount_usd: 10_000.0,
            fixed_rate_pct: 2.0,
            semi_annual_inflation_pct: 1.5,
            holding_period_months: 600,  // 50 years input
        });
        assert_eq!(r.holding_period_months, 360);  // capped at 30y
    }
}
