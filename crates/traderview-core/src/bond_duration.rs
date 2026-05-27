//! Bond duration calculator — Macaulay + Modified.
//!
//! For a bond with cash flows CF_t at times t, yield-to-maturity y,
//! and price P:
//!
//!   Macaulay duration  = Σ (t × PV(CF_t)) / P
//!   Modified duration  = Macaulay / (1 + y/m)   where m = compounding freq
//!
//! Modified duration approximates: ΔP/P ≈ -ModDur × Δy.
//!
//! Used for fixed-income risk assessment + interest-rate-sensitivity
//! sizing. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DurationReport {
    pub price: f64,
    pub macaulay_duration: f64,
    pub modified_duration: f64,
    pub yield_to_maturity: f64,
}

/// Computes both Macaulay and Modified duration given cash flows + YTM.
/// `compounding_per_year` is typically 2 for US bonds (semi-annual).
pub fn compute(cash_flows: &[CashFlow], ytm: f64, compounding_per_year: usize) -> DurationReport {
    let mut report = DurationReport {
        yield_to_maturity: ytm,
        ..Default::default()
    };
    if cash_flows.is_empty() {
        return report;
    }
    let m = compounding_per_year.max(1) as f64;
    let mut price = 0.0;
    let mut weighted_time = 0.0;
    for cf in cash_flows {
        // PV factor: (1 + y/m)^(t × m).
        let factor = (1.0 + ytm / m).powf(cf.time_years * m);
        let pv = cf.amount / factor;
        price += pv;
        weighted_time += cf.time_years * pv;
    }
    if price <= 0.0 {
        return report;
    }
    report.price = price;
    report.macaulay_duration = weighted_time / price;
    report.modified_duration = report.macaulay_duration / (1.0 + ytm / m);
    report
}

/// Quick: given modified duration + yield change in basis points,
/// estimate the price change percent.
pub fn price_change_pct(modified_duration: f64, yield_change_bps: f64) -> f64 {
    -modified_duration * (yield_change_bps / 10_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cf(t: f64, amount: f64) -> CashFlow {
        CashFlow {
            time_years: t,
            amount,
        }
    }

    #[test]
    fn empty_cash_flows_returns_default() {
        let r = compute(&[], 0.05, 2);
        assert_eq!(r.macaulay_duration, 0.0);
    }

    #[test]
    fn zero_coupon_bond_duration_equals_maturity() {
        // Single $100 cash flow at year 5 → Macaulay = 5 years.
        let r = compute(&[cf(5.0, 100.0)], 0.05, 2);
        assert!((r.macaulay_duration - 5.0).abs() < 1e-9);
    }

    #[test]
    fn modified_less_than_macaulay() {
        let r = compute(&[cf(5.0, 100.0)], 0.05, 2);
        assert!(r.modified_duration < r.macaulay_duration);
    }

    #[test]
    fn coupon_bond_duration_less_than_maturity() {
        // 5-year 5% annual coupon bond, par 100. CFs: 5 at years 1-4, 105 at year 5.
        let cfs = vec![
            cf(1.0, 5.0),
            cf(2.0, 5.0),
            cf(3.0, 5.0),
            cf(4.0, 5.0),
            cf(5.0, 105.0),
        ];
        let r = compute(&cfs, 0.05, 1);
        assert!(
            r.macaulay_duration < 5.0,
            "coupon bond has earlier cash flows → duration < maturity"
        );
        assert!(
            r.macaulay_duration > 4.0,
            "but still close to maturity for low coupon"
        );
    }

    #[test]
    fn price_at_par_when_ytm_equals_coupon() {
        let cfs = vec![
            cf(1.0, 5.0),
            cf(2.0, 5.0),
            cf(3.0, 5.0),
            cf(4.0, 5.0),
            cf(5.0, 105.0),
        ];
        let r = compute(&cfs, 0.05, 1);
        assert!(
            (r.price - 100.0).abs() < 1e-9,
            "5% coupon at 5% YTM should price at par"
        );
    }

    #[test]
    fn higher_ytm_shorter_duration() {
        let cfs = vec![
            cf(1.0, 5.0),
            cf(2.0, 5.0),
            cf(3.0, 5.0),
            cf(4.0, 5.0),
            cf(5.0, 105.0),
        ];
        let low_ytm = compute(&cfs, 0.03, 1);
        let high_ytm = compute(&cfs, 0.08, 1);
        // Higher YTM → near-term CFs weighted more → shorter duration.
        assert!(high_ytm.macaulay_duration < low_ytm.macaulay_duration);
    }

    #[test]
    fn price_change_pct_one_year_duration_100bps_is_minus_1pct() {
        // ΔP/P ≈ -ModDur × Δy = -1 × 0.01 = -1%.
        let pct = price_change_pct(1.0, 100.0);
        assert!((pct + 0.01).abs() < 1e-12);
    }

    #[test]
    fn price_change_pct_negative_yield_change_positive_price() {
        // Yields drop 50bps with 5-year duration → +2.5% price.
        let pct = price_change_pct(5.0, -50.0);
        assert!((pct - 0.025).abs() < 1e-12);
    }

    #[test]
    fn semi_annual_compounding_changes_pv_calc() {
        // Same CFs, m=2 vs m=1 → different prices.
        let cfs = vec![cf(1.0, 100.0)];
        let annual = compute(&cfs, 0.06, 1);
        let semi = compute(&cfs, 0.06, 2);
        // Semi-annual PV: 100/(1.03)^2 = 94.26. Annual PV: 100/1.06 = 94.34.
        assert!(semi.price < annual.price);
    }
}
