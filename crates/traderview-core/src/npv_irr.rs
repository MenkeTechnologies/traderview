//! NPV / IRR capital-budgeting analysis for a uniform-period cash-flow series.
//!
//! Period-indexed flows (index 0 is today, usually the negative outlay):
//!
//! ```text
//! NPV = Σ cf_t / (1+r)^t
//! IRR = the rate r where NPV = 0
//! profitability index = PV(future flows) / initial outlay
//! ```
//!
//! Plus simple and discounted payback — the period at which cumulative
//! (undiscounted / discounted) cash flow turns positive. Distinct from
//! `position_irr`, which is a date-based XIRR for trading positions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct NpvIrrInput {
    /// Cash flows by period; index 0 = today (typically negative).
    pub cash_flows: Vec<f64>,
    /// Discount rate for NPV and the profitability index, percent.
    pub discount_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NpvIrrResult {
    /// NPV at the supplied discount rate.
    pub npv_usd: f64,
    /// The rate that zeroes NPV, percent; `None` if it can't be bracketed.
    pub irr_pct: Option<f64>,
    /// PV(future flows) / initial outlay; `None` if flow 0 is not an outlay.
    pub profitability_index: Option<f64>,
    /// Years until cumulative undiscounted cash flow turns positive; `None`
    /// if it never does.
    pub payback_years: Option<f64>,
    /// Years until cumulative discounted cash flow turns positive; `None`
    /// if it never does.
    pub discounted_payback_years: Option<f64>,
    /// Sum of the raw cash flows (undiscounted).
    pub total_undiscounted_usd: f64,
}

fn npv_at(flows: &[f64], rate: f64) -> f64 {
    flows
        .iter()
        .enumerate()
        .map(|(t, &cf)| cf / (1.0 + rate).powi(t as i32))
        .sum()
}

/// Bisection IRR: bracket the sign change between −99.99% and 10,000%.
fn solve_irr(flows: &[f64]) -> Option<f64> {
    let (mut lo, mut hi) = (-0.9999, 100.0);
    let (flo, fhi) = (npv_at(flows, lo), npv_at(flows, hi));
    if flo * fhi > 0.0 {
        return None; // no sign change in range
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if npv_at(flows, lo) * npv_at(flows, mid) <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Period (with linear interpolation) at which a running sum first turns
/// non-negative.
fn crossover<F: Fn(usize) -> f64>(n: usize, value_at: F) -> Option<f64> {
    let mut cum = 0.0;
    for t in 0..n {
        let d = value_at(t);
        let prev = cum;
        cum += d;
        if prev < 0.0 && cum >= 0.0 && d != 0.0 {
            return Some(t as f64 - 1.0 + (-prev) / d);
        }
        if prev < 0.0 && cum == 0.0 {
            return Some(t as f64);
        }
    }
    if cum >= 0.0 && n > 0 {
        // Already non-negative at t=0 (no outlay) — recovered immediately.
        if value_at(0) >= 0.0 {
            return Some(0.0);
        }
    }
    None
}

pub fn analyze(input: &NpvIrrInput) -> NpvIrrResult {
    let r = input.discount_rate_pct / 100.0;
    let flows = &input.cash_flows;

    let npv = npv_at(flows, r);
    let irr = solve_irr(flows).map(|x| x * 100.0);

    let profitability_index = match flows.first() {
        Some(&cf0) if cf0 < 0.0 => {
            let pv_future: f64 = flows
                .iter()
                .enumerate()
                .skip(1)
                .map(|(t, &cf)| cf / (1.0 + r).powi(t as i32))
                .sum();
            Some(pv_future / -cf0)
        }
        _ => None,
    };

    let payback = crossover(flows.len(), |t| flows[t]);
    let discounted_payback =
        crossover(flows.len(), |t| flows[t] / (1.0 + r).powi(t as i32));

    NpvIrrResult {
        npv_usd: npv,
        irr_pct: irr,
        profitability_index,
        payback_years: payback,
        discounted_payback_years: discounted_payback,
        total_undiscounted_usd: flows.iter().sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    fn base() -> NpvIrrInput {
        NpvIrrInput {
            cash_flows: vec![-1000.0, 500.0, 500.0, 500.0],
            discount_rate_pct: 10.0,
        }
    }

    #[test]
    fn npv_value() {
        assert!(close(analyze(&base()).npv_usd, 243.425995));
    }

    #[test]
    fn irr_value() {
        let r = analyze(&base());
        assert!(close(r.irr_pct.unwrap(), 23.375193));
    }

    #[test]
    fn irr_is_self_consistent() {
        // NPV evaluated at the IRR is ~0.
        let r = analyze(&base());
        let flows = base().cash_flows;
        assert!(npv_at(&flows, r.irr_pct.unwrap() / 100.0).abs() < 1e-3);
    }

    #[test]
    fn profitability_index() {
        assert!(close(analyze(&base()).profitability_index.unwrap(), 1.243426));
    }

    #[test]
    fn simple_payback_exact_period() {
        // -1000 +500 +500 → cumulative hits 0 at end of year 2.
        let r = analyze(&base());
        assert!(close(r.payback_years.unwrap(), 2.0));
    }

    #[test]
    fn discounted_payback_interpolates() {
        assert!(close(analyze(&base()).discounted_payback_years.unwrap(), 2.352));
    }

    #[test]
    fn npv_negative_when_rate_above_irr() {
        let r = analyze(&NpvIrrInput {
            cash_flows: vec![-1000.0, 500.0, 500.0, 500.0],
            discount_rate_pct: 30.0,
        });
        assert!(r.npv_usd < 0.0);
    }

    #[test]
    fn never_recovers_has_no_payback() {
        // Cumulative −1000, −900, −800 never turns positive.
        let r = analyze(&NpvIrrInput {
            cash_flows: vec![-1000.0, 100.0, 100.0],
            discount_rate_pct: 10.0,
        });
        assert!(r.payback_years.is_none());
    }

    #[test]
    fn all_negative_series_has_no_irr() {
        // No sign change in NPV → IRR can't be bracketed.
        let r = analyze(&NpvIrrInput {
            cash_flows: vec![-1000.0, -100.0, -100.0],
            discount_rate_pct: 10.0,
        });
        assert!(r.irr_pct.is_none());
        assert!(r.payback_years.is_none());
    }
}
