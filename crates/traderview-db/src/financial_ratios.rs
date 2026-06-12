//! Standard personal-finance ratios + traffic-light health rating.
//!
//! Computes the canonical set of household financial-health ratios
//! cited across CFP / Bogleheads / consumer-finance literature, each
//! with a published benchmark and a traffic-light rating:
//!
//!   - savings_rate        — (gross_income − total_expenses) / gross_income
//!     target ≥ 20% (good ≥ 15%, ok ≥ 5%, poor < 5%)
//!   - debt_to_income      — total_monthly_debt_payments / gross_monthly_income
//!     target ≤ 36% (good ≤ 36%, ok ≤ 43%, poor > 43%)
//!     (CFPB 43% qualified-mortgage cap)
//!   - front_end_ratio     — housing_payment / gross_monthly_income
//!     target ≤ 28% (good ≤ 28%, ok ≤ 33%, poor > 33%)
//!     (28/36 rule; FHA caps 31%)
//!   - liquidity_ratio     — liquid_assets / monthly_expenses
//!     target ≥ 6 months
//!   - solvency_ratio      — net_worth / total_assets × 100
//!     target ≥ 50% (under 20% = high leverage)
//!   - emergency_fund_ratio— emergency_fund_balance / monthly_expenses
//!     target ≥ 6 months
//!   - retirement_savings_ratio — retirement_assets / annual_gross_income
//!     target ≥ age × Fidelity benchmark (1x@30,
//!     3x@40, 6x@50, 8x@60, 10x@67) — we accept
//!     a multiplier-only target since age is
//!     user-supplied
//!   - composite score     — average of per-ratio scores (0/1/2 per
//!     poor/ok/good) → 0-100
//!
//! Pure compute — no DB I/O.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RatiosInput {
    pub gross_monthly_income_usd: f64,
    pub total_monthly_expenses_usd: f64,
    pub monthly_debt_payments_usd: f64,
    pub monthly_housing_payment_usd: f64,
    pub liquid_assets_usd: f64,
    pub emergency_fund_balance_usd: f64,
    pub total_assets_usd: f64,
    pub total_liabilities_usd: f64,
    pub retirement_assets_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RatioReport {
    pub savings_rate_pct: f64,
    pub debt_to_income_pct: f64,
    pub front_end_ratio_pct: f64,
    pub liquidity_ratio_months: f64,
    pub solvency_ratio_pct: f64,
    pub emergency_fund_ratio_months: f64,
    pub retirement_savings_ratio_x: f64,
    pub composite_score_pct: f64,
    pub ratings: Vec<RatingCell>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RatingCell {
    pub key: &'static str,
    pub value: f64,
    pub rating: &'static str, // "good" | "ok" | "poor"
    pub benchmark: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn safe_pct(num: f64, denom: f64) -> f64 {
    if denom <= 0.0 {
        return 0.0;
    }
    num / denom * 100.0
}

pub fn safe_div(num: f64, denom: f64) -> f64 {
    if denom <= 0.0 {
        return 0.0;
    }
    num / denom
}

pub fn rate_savings(pct: f64) -> &'static str {
    if pct >= 20.0 { "good" } else if pct >= 5.0 { "ok" } else { "poor" }
}

pub fn rate_dti(pct: f64) -> &'static str {
    if pct <= 36.0 { "good" } else if pct <= 43.0 { "ok" } else { "poor" }
}

pub fn rate_front_end(pct: f64) -> &'static str {
    if pct <= 28.0 { "good" } else if pct <= 33.0 { "ok" } else { "poor" }
}

pub fn rate_liquidity(months: f64) -> &'static str {
    if months >= 6.0 { "good" } else if months >= 3.0 { "ok" } else { "poor" }
}

pub fn rate_solvency(pct: f64) -> &'static str {
    if pct >= 50.0 { "good" } else if pct >= 20.0 { "ok" } else { "poor" }
}

pub fn rate_emergency(months: f64) -> &'static str {
    if months >= 6.0 { "good" } else if months >= 3.0 { "ok" } else { "poor" }
}

pub fn rate_retirement(x: f64) -> &'static str {
    // Generic: ≥ 5× income = good, ≥ 1× = ok, < 1× = poor.
    if x >= 5.0 { "good" } else if x >= 1.0 { "ok" } else { "poor" }
}

pub fn rating_score(r: &str) -> u8 {
    match r {
        "good" => 2,
        "ok" => 1,
        _ => 0,
    }
}

pub fn compute(input: &RatiosInput) -> RatioReport {
    let savings = safe_pct(
        input.gross_monthly_income_usd - input.total_monthly_expenses_usd,
        input.gross_monthly_income_usd,
    );
    let dti = safe_pct(input.monthly_debt_payments_usd, input.gross_monthly_income_usd);
    let front = safe_pct(
        input.monthly_housing_payment_usd,
        input.gross_monthly_income_usd,
    );
    let liquidity = safe_div(input.liquid_assets_usd, input.total_monthly_expenses_usd);
    let net_worth = input.total_assets_usd - input.total_liabilities_usd;
    let solvency = safe_pct(net_worth, input.total_assets_usd);
    let emergency = safe_div(input.emergency_fund_balance_usd, input.total_monthly_expenses_usd);
    let annual_income = input.gross_monthly_income_usd * 12.0;
    let retirement_x = safe_div(input.retirement_assets_usd, annual_income);

    let r_savings = rate_savings(savings);
    let r_dti = rate_dti(dti);
    let r_front = rate_front_end(front);
    let r_liquidity = rate_liquidity(liquidity);
    let r_solvency = rate_solvency(solvency);
    let r_emergency = rate_emergency(emergency);
    let r_retirement = rate_retirement(retirement_x);

    let scores = [
        rating_score(r_savings),
        rating_score(r_dti),
        rating_score(r_front),
        rating_score(r_liquidity),
        rating_score(r_solvency),
        rating_score(r_emergency),
        rating_score(r_retirement),
    ];
    let total: u32 = scores.iter().map(|s| *s as u32).sum();
    // 7 ratios × 2 max points = 14 max.
    let composite = total as f64 / 14.0 * 100.0;

    let ratings = vec![
        RatingCell { key: "savings_rate",          value: savings,    rating: r_savings,    benchmark: "≥ 20%" },
        RatingCell { key: "debt_to_income",        value: dti,        rating: r_dti,        benchmark: "≤ 36% (CFPB QM cap 43%)" },
        RatingCell { key: "front_end_ratio",       value: front,      rating: r_front,      benchmark: "≤ 28% (FHA 31%)" },
        RatingCell { key: "liquidity_ratio",       value: liquidity,  rating: r_liquidity,  benchmark: "≥ 6 months" },
        RatingCell { key: "solvency_ratio",        value: solvency,   rating: r_solvency,   benchmark: "≥ 50%" },
        RatingCell { key: "emergency_fund_ratio",  value: emergency,  rating: r_emergency,  benchmark: "≥ 6 months" },
        RatingCell { key: "retirement_savings_x",  value: retirement_x, rating: r_retirement, benchmark: "≥ 5× annual income (Fidelity 60s = 8×, 67 = 10×)" },
    ];

    RatioReport {
        savings_rate_pct: savings,
        debt_to_income_pct: dti,
        front_end_ratio_pct: front,
        liquidity_ratio_months: liquidity,
        solvency_ratio_pct: solvency,
        emergency_fund_ratio_months: emergency,
        retirement_savings_ratio_x: retirement_x,
        composite_score_pct: composite,
        ratings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_pct_zero_denom() {
        assert_eq!(safe_pct(100.0, 0.0), 0.0);
    }

    #[test]
    fn safe_pct_basic() {
        assert_eq!(safe_pct(25.0, 100.0), 25.0);
    }

    #[test]
    fn safe_div_zero_denom() {
        assert_eq!(safe_div(100.0, 0.0), 0.0);
    }

    #[test]
    fn rate_savings_thresholds() {
        assert_eq!(rate_savings(25.0), "good");
        assert_eq!(rate_savings(10.0), "ok");
        assert_eq!(rate_savings(1.0),  "poor");
    }

    #[test]
    fn rate_dti_thresholds() {
        assert_eq!(rate_dti(20.0), "good");
        assert_eq!(rate_dti(40.0), "ok");
        assert_eq!(rate_dti(50.0), "poor");
    }

    #[test]
    fn rate_front_end_thresholds() {
        assert_eq!(rate_front_end(20.0), "good");
        assert_eq!(rate_front_end(30.0), "ok");
        assert_eq!(rate_front_end(40.0), "poor");
    }

    #[test]
    fn rate_liquidity_thresholds() {
        assert_eq!(rate_liquidity(8.0), "good");
        assert_eq!(rate_liquidity(4.0), "ok");
        assert_eq!(rate_liquidity(1.0), "poor");
    }

    #[test]
    fn rate_solvency_thresholds() {
        assert_eq!(rate_solvency(60.0), "good");
        assert_eq!(rate_solvency(30.0), "ok");
        assert_eq!(rate_solvency(10.0), "poor");
    }

    #[test]
    fn rate_retirement_thresholds() {
        assert_eq!(rate_retirement(6.0), "good");
        assert_eq!(rate_retirement(2.0), "ok");
        assert_eq!(rate_retirement(0.5), "poor");
    }

    #[test]
    fn rating_score_mapping() {
        assert_eq!(rating_score("good"), 2);
        assert_eq!(rating_score("ok"), 1);
        assert_eq!(rating_score("poor"), 0);
        assert_eq!(rating_score("bogus"), 0);
    }

    #[test]
    fn compute_healthy_household_full_marks() {
        let r = compute(&RatiosInput {
            gross_monthly_income_usd: 10_000.0,
            total_monthly_expenses_usd: 5_000.0,   // 50% saving rate
            monthly_debt_payments_usd: 2_000.0,    // 20% DTI
            monthly_housing_payment_usd: 1_500.0,  // 15% front-end
            liquid_assets_usd: 60_000.0,           // 12 months
            emergency_fund_balance_usd: 60_000.0,  // 12 months
            total_assets_usd: 800_000.0,
            total_liabilities_usd: 200_000.0,      // solvency 75%
            retirement_assets_usd: 800_000.0,      // 6.67× annual
        });
        assert!((r.savings_rate_pct - 50.0).abs() < 1e-6);
        assert!((r.debt_to_income_pct - 20.0).abs() < 1e-6);
        assert!((r.front_end_ratio_pct - 15.0).abs() < 1e-6);
        assert!((r.liquidity_ratio_months - 12.0).abs() < 1e-6);
        assert!((r.solvency_ratio_pct - 75.0).abs() < 1e-6);
        assert!((r.emergency_fund_ratio_months - 12.0).abs() < 1e-6);
        assert!((r.retirement_savings_ratio_x - 800_000.0 / 120_000.0).abs() < 1e-6);
        assert!(r.composite_score_pct > 99.0);
    }

    #[test]
    fn compute_struggling_household_low_score() {
        let r = compute(&RatiosInput {
            gross_monthly_income_usd: 3_000.0,
            total_monthly_expenses_usd: 2_950.0,   // 1.7% saving rate
            monthly_debt_payments_usd: 1_400.0,    // 47% DTI
            monthly_housing_payment_usd: 1_100.0,  // 37% front-end
            liquid_assets_usd: 500.0,              // 0.17 months
            emergency_fund_balance_usd: 200.0,
            total_assets_usd: 50_000.0,
            total_liabilities_usd: 80_000.0,
            retirement_assets_usd: 5_000.0,
        });
        assert!(r.composite_score_pct < 25.0);
    }

    #[test]
    fn compute_composite_seven_ratios_returned() {
        let r = compute(&RatiosInput {
            gross_monthly_income_usd: 5_000.0,
            total_monthly_expenses_usd: 3_500.0,
            monthly_debt_payments_usd: 1_000.0,
            monthly_housing_payment_usd: 1_200.0,
            liquid_assets_usd: 15_000.0,
            emergency_fund_balance_usd: 12_000.0,
            total_assets_usd: 200_000.0,
            total_liabilities_usd: 60_000.0,
            retirement_assets_usd: 100_000.0,
        });
        assert_eq!(r.ratings.len(), 7);
    }
}
