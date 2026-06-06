//! Bond duration suite — Macaulay, modified, dollar (DV01), convexity.
//!
//! Conventions:
//!   - Cash flows priced at constant `yield_to_maturity` (annualized,
//!     compounded `freq` times per year — semi-annual = 2 for US Treasuries).
//!   - Each cash flow CF_i occurs at time t_i (years from settlement).
//!   - PV_i = CF_i / (1 + y/freq)^(t_i · freq)
//!
//!   price          = Σ PV_i
//!   macaulay_dur   = Σ (t_i · PV_i) / price
//!   modified_dur   = macaulay_dur / (1 + y/freq)
//!   dv01           = price · modified_dur · 0.0001
//!   convexity      = Σ (t_i · (t_i + 1/freq) · PV_i) / price / (1 + y/freq)²
//!
//! Distinct from the existing `bond_duration` module — that one uses a
//! flat formula assuming continuous compounding; this one is the
//! market-standard discrete-compounding form with explicit cash flow
//! schedule, supporting irregular coupons (e.g. step-up, callable).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DurationReport {
    pub price: f64,
    pub macaulay_duration: f64,
    pub modified_duration: f64,
    pub dv01: f64,
    pub convexity: f64,
}

pub fn compute(
    cash_flows: &[CashFlow],
    yield_to_maturity: f64,
    compounding_freq: u32,
) -> Option<DurationReport> {
    if cash_flows.is_empty()
        || !yield_to_maturity.is_finite()
        || yield_to_maturity <= -1.0
        || compounding_freq == 0
    {
        return None;
    }
    if cash_flows
        .iter()
        .any(|cf| !cf.time_years.is_finite() || !cf.amount.is_finite() || cf.time_years < 0.0)
    {
        return None;
    }
    let f = compounding_freq as f64;
    let y_per = yield_to_maturity / f;
    let one_plus = 1.0 + y_per;
    if one_plus <= 0.0 {
        return None;
    }
    let mut price = 0.0_f64;
    let mut weighted_time = 0.0_f64;
    let mut convexity_num = 0.0_f64;
    for cf in cash_flows {
        let n_periods = cf.time_years * f;
        let discount = one_plus.powf(n_periods);
        if !discount.is_finite() || discount == 0.0 {
            return None;
        }
        let pv = cf.amount / discount;
        if !pv.is_finite() {
            return None;
        }
        price += pv;
        weighted_time += cf.time_years * pv;
        convexity_num += cf.time_years * (cf.time_years + 1.0 / f) * pv;
    }
    if price <= 0.0 || !price.is_finite() {
        return None;
    }
    let macaulay = weighted_time / price;
    let modified = macaulay / one_plus;
    let dv01 = price * modified * 0.0001;
    let convexity = convexity_num / price / (one_plus * one_plus);
    Some(DurationReport {
        price,
        macaulay_duration: macaulay,
        modified_duration: modified,
        dv01,
        convexity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cf(t: f64, a: f64) -> CashFlow {
        CashFlow {
            time_years: t,
            amount: a,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 0.05, 2).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        let cfs = vec![cf(1.0, 100.0)];
        assert!(compute(&cfs, f64::NAN, 2).is_none());
        assert!(compute(&cfs, -1.5, 2).is_none());
        assert!(compute(&cfs, 0.05, 0).is_none());
        assert!(compute(&[cf(-1.0, 100.0)], 0.05, 2).is_none());
        assert!(compute(&[cf(1.0, f64::NAN)], 0.05, 2).is_none());
    }

    #[test]
    fn zero_coupon_bond_duration_equals_maturity() {
        // Single cash flow at T=5 → Macaulay duration = 5 exactly.
        let cfs = vec![cf(5.0, 100.0)];
        let r = compute(&cfs, 0.05, 2).unwrap();
        assert!((r.macaulay_duration - 5.0).abs() < 1e-9);
    }

    #[test]
    fn coupon_bond_duration_less_than_maturity() {
        // 5-year 5% semi-annual bond at 5% yield → priced at par. Macaulay < 5.
        let mut cfs = Vec::new();
        for k in 1..=10 {
            cfs.push(cf(k as f64 / 2.0, 2.5)); // semi-annual coupons
        }
        cfs.push(cf(5.0, 100.0)); // principal at maturity
        let r = compute(&cfs, 0.05, 2).unwrap();
        assert!(
            r.macaulay_duration < 5.0 && r.macaulay_duration > 4.0,
            "expected Macaulay duration ~4.5y for 5y 5% bond, got {}",
            r.macaulay_duration
        );
    }

    #[test]
    fn par_bond_priced_at_par() {
        // 5y 5% coupon at 5% yield → exactly $100.
        let mut cfs = Vec::new();
        for k in 1..=10 {
            cfs.push(cf(k as f64 / 2.0, 2.5));
        }
        cfs.push(cf(5.0, 100.0));
        let r = compute(&cfs, 0.05, 2).unwrap();
        assert!(
            (r.price - 100.0).abs() < 1e-6,
            "expected par price, got {}",
            r.price
        );
    }

    #[test]
    fn modified_duration_less_than_macaulay_when_yield_positive() {
        let cfs = vec![cf(1.0, 5.0), cf(2.0, 5.0), cf(3.0, 105.0)];
        let r = compute(&cfs, 0.05, 1).unwrap();
        assert!(r.modified_duration < r.macaulay_duration);
        // Specifically: modified = macaulay / (1 + 0.05)
        assert!((r.modified_duration - r.macaulay_duration / 1.05).abs() < 1e-9);
    }

    #[test]
    fn dv01_positive_for_positive_price() {
        let cfs = vec![cf(1.0, 100.0)];
        let r = compute(&cfs, 0.05, 2).unwrap();
        assert!(r.dv01 > 0.0);
    }

    #[test]
    fn convexity_positive_for_standard_bond() {
        let mut cfs = Vec::new();
        for k in 1..=10 {
            cfs.push(cf(k as f64 / 2.0, 2.5));
        }
        cfs.push(cf(5.0, 100.0));
        let r = compute(&cfs, 0.05, 2).unwrap();
        assert!(r.convexity > 0.0);
    }

    #[test]
    fn higher_yield_lowers_price() {
        let mut cfs = Vec::new();
        for k in 1..=10 {
            cfs.push(cf(k as f64 / 2.0, 2.5));
        }
        cfs.push(cf(5.0, 100.0));
        let r_low = compute(&cfs, 0.03, 2).unwrap();
        let r_high = compute(&cfs, 0.08, 2).unwrap();
        assert!(r_low.price > r_high.price);
    }
}
