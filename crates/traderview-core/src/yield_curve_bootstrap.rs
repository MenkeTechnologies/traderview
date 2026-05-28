//! Yield-curve bootstrap from coupon bond prices.
//!
//! Builds a discount factor curve d(t) by sequentially solving for the
//! zero rate at each maturity. Given a series of bonds sorted by
//! maturity:
//!
//!   For each bond i (in order of ascending maturity):
//!     known_pv  = Σ d(t_k) · CF_i,k     for cash flows at t_k < T_i
//!     unknown   = d(T_i) · CF_i,T_i
//!     d(T_i)    = (price_i − known_pv) / CF_i,T_i
//!     zero_rate(T_i) = −ln(d(T_i)) / T_i        (continuously compounded)
//!
//! Cash flows BETWEEN known maturities are linearly interpolated on
//! zero rates (the canonical market practice for ad-hoc bootstrapping).
//!
//! Pure compute. Distinct from a full Nelson-Siegel fit — this is the
//! exact-fit bootstrap used in trading desks for repricing constraints.
//! Distinct from `yield_curve` (which models the published yield-curve
//! product) — this module *builds* the curve from raw bond quotes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouponBond {
    pub price: f64,
    /// Per-bond cash flows, ascending by time_years (final entry is
    /// principal + final coupon).
    pub cash_flows: Vec<CashFlow>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YieldCurveReport {
    /// Knot points (time, zero_rate, discount_factor) sorted ascending.
    pub knots: Vec<KnotPoint>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KnotPoint {
    pub time_years: f64,
    pub zero_rate: f64,
    pub discount_factor: f64,
}

pub fn bootstrap(bonds: &[CouponBond]) -> Option<YieldCurveReport> {
    if bonds.is_empty() { return None; }
    let mut sorted: Vec<&CouponBond> = bonds.iter().collect();
    if sorted.iter().any(|b| b.cash_flows.is_empty()
        || !b.price.is_finite()
        || b.price <= 0.0
        || b.cash_flows.iter().any(|cf| !cf.time_years.is_finite() || !cf.amount.is_finite()
            || cf.time_years <= 0.0))
    {
        return None;
    }
    sorted.sort_by(|a, b| {
        let am = a.cash_flows.last().unwrap().time_years;
        let bm = b.cash_flows.last().unwrap().time_years;
        am.partial_cmp(&bm).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut knots: Vec<KnotPoint> = Vec::with_capacity(sorted.len());
    for bond in &sorted {
        let maturity = bond.cash_flows.last().unwrap().time_years;
        let final_cf = bond.cash_flows.last().unwrap().amount;
        let mut known_pv = 0.0_f64;
        for cf in &bond.cash_flows[..bond.cash_flows.len() - 1] {
            let df = discount_factor_for(cf.time_years, &knots)?;
            known_pv += cf.amount * df;
        }
        let unknown_pv = bond.price - known_pv;
        if unknown_pv <= 0.0 || final_cf <= 0.0 { return None; }
        let df_t = unknown_pv / final_cf;
        if df_t <= 0.0 || df_t > 1.0 { return None; }
        let zero_rate = -df_t.ln() / maturity;
        if !zero_rate.is_finite() { return None; }
        knots.push(KnotPoint {
            time_years: maturity,
            zero_rate,
            discount_factor: df_t,
        });
    }
    Some(YieldCurveReport { knots })
}

fn discount_factor_for(t: f64, knots: &[KnotPoint]) -> Option<f64> {
    if knots.is_empty() {
        return None;
    }
    for k in knots {
        if (k.time_years - t).abs() < 1e-12 {
            return Some(k.discount_factor);
        }
    }
    if t <= knots[0].time_years {
        let r = knots[0].zero_rate;
        return Some((-r * t).exp());
    }
    if t >= knots.last().unwrap().time_years {
        let r = knots.last().unwrap().zero_rate;
        return Some((-r * t).exp());
    }
    for w in knots.windows(2) {
        if t >= w[0].time_years && t <= w[1].time_years {
            let frac = (t - w[0].time_years) / (w[1].time_years - w[0].time_years);
            let r = w[0].zero_rate + frac * (w[1].zero_rate - w[0].zero_rate);
            return Some((-r * t).exp());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cf(t: f64, a: f64) -> CashFlow { CashFlow { time_years: t, amount: a } }
    fn bond(price: f64, cfs: Vec<CashFlow>) -> CouponBond {
        CouponBond { price, cash_flows: cfs }
    }

    #[test]
    fn empty_returns_none() {
        assert!(bootstrap(&[]).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(bootstrap(&[bond(f64::NAN, vec![cf(1.0, 100.0)])]).is_none());
        assert!(bootstrap(&[bond(95.0, vec![cf(-1.0, 100.0)])]).is_none());
        assert!(bootstrap(&[bond(95.0, vec![])]).is_none());
    }

    #[test]
    fn single_zero_coupon_bond_recovers_zero_rate() {
        let r = bootstrap(&[bond(95.123, vec![cf(1.0, 100.0)])]).unwrap();
        assert_eq!(r.knots.len(), 1);
        assert!((r.knots[0].zero_rate - 0.05).abs() < 0.001);
        assert!((r.knots[0].discount_factor - 0.95123).abs() < 1e-4);
    }

    #[test]
    fn two_zero_coupons_build_two_knots() {
        let r = bootstrap(&[
            bond(95.0, vec![cf(1.0, 100.0)]),
            bond(90.0, vec![cf(2.0, 100.0)]),
        ]).unwrap();
        assert_eq!(r.knots.len(), 2);
        assert!(r.knots[0].zero_rate.is_finite());
        assert!(r.knots[1].zero_rate.is_finite());
    }

    #[test]
    fn coupon_bond_bootstraps_after_zero_anchor() {
        let df_1 = (-0.045_f64).exp();
        let target_df_2 = 0.91;
        let price = 5.0 * df_1 + 105.0 * target_df_2;
        let bonds = vec![
            bond(100.0 * df_1, vec![cf(1.0, 100.0)]),
            bond(price, vec![cf(1.0, 5.0), cf(2.0, 105.0)]),
        ];
        let r = bootstrap(&bonds).unwrap();
        assert_eq!(r.knots.len(), 2);
        assert!((r.knots[1].discount_factor - target_df_2).abs() < 1e-3);
    }

    #[test]
    fn unsorted_bonds_handled_by_internal_sort() {
        let r = bootstrap(&[
            bond(90.0, vec![cf(2.0, 100.0)]),
            bond(95.0, vec![cf(1.0, 100.0)]),
        ]).unwrap();
        assert!(r.knots[0].time_years < r.knots[1].time_years);
    }

    #[test]
    fn discount_factor_decreases_with_time_under_positive_rates() {
        let r = bootstrap(&[
            bond(95.0, vec![cf(1.0, 100.0)]),
            bond(85.0, vec![cf(2.0, 100.0)]),
        ]).unwrap();
        assert!(r.knots[1].discount_factor < r.knots[0].discount_factor);
    }

    #[test]
    fn arbitrage_violation_returns_none() {
        let r = bootstrap(&[bond(110.0, vec![cf(1.0, 100.0)])]);
        assert!(r.is_none());
    }
}
