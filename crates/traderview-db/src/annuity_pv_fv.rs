//! Annuity present value / future value calculator.
//!
//! Classic time-value-of-money for a level annuity (equal periodic
//! payments). Supports four modes:
//!
//!   - ordinary  — payment at END of each period (typical loan / bond)
//!   - due       — payment at BEGINNING of each period (rent / lease)
//!   - PV        — what the stream is worth today
//!   - FV        — what the stream grows to at the end
//!
//!   PV(ordinary) = pmt × (1 − (1 + r)^−n) / r
//!   FV(ordinary) = pmt × ((1 + r)^n − 1) / r
//!   PV(due)      = PV(ordinary) × (1 + r)
//!   FV(due)      = FV(ordinary) × (1 + r)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AnnuityInput {
    pub payment_per_period_usd: f64,
    /// Annual rate as a percentage (e.g. 6 = 6%/yr).
    pub annual_rate_pct: f64,
    pub periods_per_year: u32,
    pub years: f64,
    /// "ordinary" or "due"
    #[serde(default = "default_kind")]
    pub annuity_kind: String,
}

fn default_kind() -> String { "ordinary".into() }

#[derive(Debug, Clone, Serialize)]
pub struct AnnuityReport {
    pub n_periods: f64,
    pub periodic_rate: f64,
    pub present_value_usd: f64,
    pub future_value_usd: f64,
    pub total_payments_usd: f64,
    pub total_interest_pv_usd: f64,
    pub total_interest_fv_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn pv_ordinary(pmt: f64, rate: f64, n: f64) -> f64 {
    if pmt <= 0.0 || n <= 0.0 { return 0.0; }
    if rate.abs() < 1e-12 { return pmt * n; }
    pmt * (1.0 - (1.0 + rate).powf(-n)) / rate
}

pub fn fv_ordinary(pmt: f64, rate: f64, n: f64) -> f64 {
    if pmt <= 0.0 || n <= 0.0 { return 0.0; }
    if rate.abs() < 1e-12 { return pmt * n; }
    pmt * ((1.0 + rate).powf(n) - 1.0) / rate
}

pub fn compute(input: &AnnuityInput) -> AnnuityReport {
    let n = input.years * input.periods_per_year as f64;
    let rate = if input.periods_per_year > 0 {
        input.annual_rate_pct / 100.0 / input.periods_per_year as f64
    } else { 0.0 };
    let mut pv = pv_ordinary(input.payment_per_period_usd, rate, n);
    let mut fv = fv_ordinary(input.payment_per_period_usd, rate, n);
    if input.annuity_kind == "due" {
        pv *= 1.0 + rate;
        fv *= 1.0 + rate;
    }
    let total_payments = input.payment_per_period_usd * n;
    AnnuityReport {
        n_periods: n,
        periodic_rate: rate,
        present_value_usd: pv,
        future_value_usd: fv,
        total_payments_usd: total_payments,
        total_interest_pv_usd: (total_payments - pv).max(0.0),
        total_interest_fv_usd: (fv - total_payments).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> AnnuityInput {
        AnnuityInput {
            payment_per_period_usd: 1_000.0,
            annual_rate_pct: 6.0,
            periods_per_year: 12,
            years: 10.0,
            annuity_kind: "ordinary".into(),
        }
    }

    #[test]
    fn pv_ordinary_zero_rate_linear() {
        assert_eq!(pv_ordinary(1000.0, 0.0, 12.0), 12_000.0);
    }

    #[test]
    fn pv_ordinary_basic() {
        // $1000/mo for 120 months at 0.5%/mo (6% annual)
        let pv = pv_ordinary(1000.0, 0.005, 120.0);
        // Published: ≈ $90,073.45
        assert!((pv - 90_073.45).abs() < 1.0, "got {pv}");
    }

    #[test]
    fn fv_ordinary_zero_rate_linear() {
        assert_eq!(fv_ordinary(1000.0, 0.0, 12.0), 12_000.0);
    }

    #[test]
    fn fv_ordinary_basic() {
        // $1000/mo for 120 months at 0.5%/mo
        let fv = fv_ordinary(1000.0, 0.005, 120.0);
        // Published: ≈ $163,879.35
        assert!((fv - 163_879.35).abs() < 1.0, "got {fv}");
    }

    #[test]
    fn compute_ordinary_basic() {
        let r = compute(&input());
        assert_eq!(r.n_periods, 120.0);
        assert!((r.periodic_rate - 0.005).abs() < 1e-9);
        assert!((r.present_value_usd - 90_073.45).abs() < 1.0);
        assert!((r.future_value_usd - 163_879.35).abs() < 1.0);
        assert_eq!(r.total_payments_usd, 120_000.0);
    }

    #[test]
    fn compute_due_higher_than_ordinary() {
        let ord = compute(&input());
        let mut i = input();
        i.annuity_kind = "due".into();
        let due = compute(&i);
        // Due > ordinary by factor (1 + r).
        let r = 0.005;
        assert!((due.present_value_usd - ord.present_value_usd * (1.0 + r)).abs() < 0.01);
        assert!((due.future_value_usd - ord.future_value_usd * (1.0 + r)).abs() < 0.01);
    }

    #[test]
    fn compute_zero_payment() {
        let mut i = input();
        i.payment_per_period_usd = 0.0;
        let r = compute(&i);
        assert_eq!(r.present_value_usd, 0.0);
        assert_eq!(r.future_value_usd, 0.0);
        assert_eq!(r.total_interest_pv_usd, 0.0);
    }

    #[test]
    fn compute_zero_years() {
        let mut i = input();
        i.years = 0.0;
        let r = compute(&i);
        assert_eq!(r.n_periods, 0.0);
        assert_eq!(r.present_value_usd, 0.0);
        assert_eq!(r.future_value_usd, 0.0);
    }

    #[test]
    fn compute_total_interest_pv_positive() {
        let r = compute(&input());
        assert!(r.total_interest_pv_usd > 0.0);
        // Total interest = total payments − PV = 120k − 90.073k ≈ 29.926k
        assert!((r.total_interest_pv_usd - 29_926.55).abs() < 1.0);
    }

    #[test]
    fn compute_total_interest_fv_positive() {
        let r = compute(&input());
        assert!(r.total_interest_fv_usd > 0.0);
        // Total interest = FV − total payments = 163.879k − 120k ≈ 43.879k
        assert!((r.total_interest_fv_usd - 43_879.35).abs() < 1.0);
    }

    #[test]
    fn compute_annual_payments() {
        let mut i = input();
        i.periods_per_year = 1;
        i.payment_per_period_usd = 12_000.0;
        let r = compute(&i);
        assert_eq!(r.n_periods, 10.0);
        // $12k/yr × 10 = $120k total payments (same as $1k/mo × 12 × 10).
        assert_eq!(r.total_payments_usd, 120_000.0);
    }
}
