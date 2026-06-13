//! SPIA — single-premium immediate annuity payout.
//!
//! Hand an insurer a lump sum and get a guaranteed income stream starting
//! now. The monthly payment is the annuity that exhausts the premium over
//! the payout period at the insurer's assumed rate — the same present-value
//! annuity math as a loan payment, run in reverse:
//!
//!   * monthly = premium × i / (1 − (1 + i)^−n), i = rate/12, n = months
//!   * payout rate = annual income / premium (what insurers quote)
//!   * total received = monthly × n (premium + the interest credited)
//!
//! This is the period-certain core; a true life annuity prices in mortality,
//! but the payout-over-life-expectancy approximation is the deterministic
//! heart of it. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SpiaInput {
    pub premium_usd: f64,
    /// Expected payout period in years (e.g. life expectancy − current age).
    pub payout_years: f64,
    /// Insurer's assumed/credited annual rate.
    pub annual_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpiaResult {
    pub monthly_income_usd: f64,
    pub annual_income_usd: f64,
    /// Annual income as a percent of the premium — the quoted payout rate.
    pub payout_rate_pct: f64,
    /// Nominal total received over the payout period.
    pub total_received_usd: f64,
    /// Total received − premium (the interest credited over the period).
    pub interest_earned_usd: f64,
}

pub fn analyze(i: &SpiaInput) -> SpiaResult {
    let premium = i.premium_usd.max(0.0);
    let n = (i.payout_years * 12.0).max(0.0);
    let monthly_rate = i.annual_rate_pct / 100.0 / 12.0;

    let monthly = if n <= 0.0 {
        0.0
    } else if monthly_rate.abs() < 1e-12 {
        premium / n
    } else {
        premium * monthly_rate / (1.0 - (1.0 + monthly_rate).powf(-n))
    };

    let annual = monthly * 12.0;
    let total = monthly * n;

    SpiaResult {
        monthly_income_usd: monthly,
        annual_income_usd: annual,
        payout_rate_pct: if premium > 0.0 { annual / premium * 100.0 } else { 0.0 },
        total_received_usd: total,
        interest_earned_usd: total - premium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(premium: f64, years: f64, rate: f64) -> SpiaInput {
        SpiaInput { premium_usd: premium, payout_years: years, annual_rate_pct: rate }
    }

    #[test]
    fn zero_rate_is_straight_line() {
        // 100k over 20y = 240 months → 416.67/mo.
        let r = analyze(&inp(100_000.0, 20.0, 0.0));
        assert!((r.monthly_income_usd - 100_000.0 / 240.0).abs() < 1e-6);
    }

    #[test]
    fn with_rate_uses_annuity_formula() {
        // 100k, 20y, 5%: i=0.0041667, n=240 → ~659.96/mo.
        let r = analyze(&inp(100_000.0, 20.0, 5.0));
        let i: f64 = 0.05 / 12.0;
        let expected = 100_000.0 * i / (1.0 - (1.0 + i).powf(-240.0));
        assert!((r.monthly_income_usd - expected).abs() < 1e-6);
        assert!(r.monthly_income_usd > 600.0 && r.monthly_income_usd < 700.0);
    }

    #[test]
    fn annual_is_monthly_times_12() {
        let r = analyze(&inp(100_000.0, 20.0, 5.0));
        assert!((r.annual_income_usd - r.monthly_income_usd * 12.0).abs() < 1e-9);
    }

    #[test]
    fn payout_rate_is_annual_over_premium() {
        let r = analyze(&inp(100_000.0, 20.0, 5.0));
        assert!((r.payout_rate_pct - r.annual_income_usd / 100_000.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn total_received_is_monthly_times_months() {
        let r = analyze(&inp(100_000.0, 20.0, 5.0));
        assert!((r.total_received_usd - r.monthly_income_usd * 240.0).abs() < 1e-6);
    }

    #[test]
    fn higher_rate_raises_monthly() {
        let low = analyze(&inp(100_000.0, 20.0, 3.0));
        let high = analyze(&inp(100_000.0, 20.0, 6.0));
        assert!(high.monthly_income_usd > low.monthly_income_usd);
    }

    #[test]
    fn longer_period_lowers_monthly() {
        let short = analyze(&inp(100_000.0, 10.0, 5.0));
        let long = analyze(&inp(100_000.0, 30.0, 5.0));
        assert!(long.monthly_income_usd < short.monthly_income_usd);
    }

    #[test]
    fn interest_earned_positive_with_rate() {
        // Zero-rate: total = premium, no interest.
        let flat = analyze(&inp(100_000.0, 20.0, 0.0));
        assert!(flat.interest_earned_usd.abs() < 1e-3);
        // With rate: total received exceeds the premium.
        let r = analyze(&inp(100_000.0, 20.0, 5.0));
        assert!(r.interest_earned_usd > 0.0);
    }
}
