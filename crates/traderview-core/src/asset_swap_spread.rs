//! Asset Swap Spread — the running spread over a benchmark swap curve
//! that makes a fixed-coupon bond price-fair vs the swap.
//!
//!   ASW = (bond_price − dirty_par_value) / annuity + (fixed_coupon −
//!         par_swap_rate)
//!
//! Equivalently: the spread an investor receives over the floating leg
//! of a par-equivalent interest-rate swap when buying the bond at
//! market price. Used for relative-value across cash bonds, MBS, and
//! corporate-vs-treasury arb books.
//!
//! Simplified par-asset-swap form (Choudhry 2004):
//!   ASW = ((bond_price − 1) − Σ_i (coupon − par_swap_rate)·τ_i·DF_i)
//!         / Σ_i τ_i · DF_i
//!         + par_swap_rate − fixed_coupon
//!
//! We implement the straightforward Quoting-Method form: take the
//! observed bond all-in PV, subtract the PV of the bond's coupons
//! priced at the par swap rate, divide by the floating-leg annuity.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub accrual: f64,
    pub discount_factor: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct AssetSwapReport {
    pub asset_swap_spread_bps: f64,
    pub annuity: f64,
    pub par_value_difference: f64,
}

pub fn analyze(
    bond_clean_price: f64,
    par_value: f64,
    fixed_coupon_rate: f64,
    par_swap_rate: f64,
    cash_flows: &[CashFlow],
) -> Option<AssetSwapReport> {
    if !bond_clean_price.is_finite() || bond_clean_price <= 0.0
        || !par_value.is_finite() || par_value <= 0.0
        || !fixed_coupon_rate.is_finite() || fixed_coupon_rate < 0.0
        || !par_swap_rate.is_finite() || par_swap_rate < 0.0
        || cash_flows.is_empty()
    {
        return None;
    }
    if cash_flows.iter().any(|cf| !cf.time_years.is_finite() || cf.time_years <= 0.0
        || !cf.accrual.is_finite() || cf.accrual <= 0.0
        || !cf.discount_factor.is_finite() || cf.discount_factor <= 0.0
        || cf.discount_factor > 1.0 + 1e-9)
    {
        return None;
    }
    let annuity: f64 = cash_flows.iter().map(|cf| cf.accrual * cf.discount_factor).sum();
    if annuity <= 0.0 { return None; }
    // PV difference between bond and par.
    let par_value_diff = bond_clean_price - par_value;
    // Closed-form ASW (per unit notional, as a decimal):
    //   ASW = par_swap_rate − fixed_coupon + (par_value − bond_price) / annuity_per_unit
    let annuity_per_unit = annuity / par_value;
    let asw = par_swap_rate - fixed_coupon_rate + (par_value - bond_clean_price) / (annuity_per_unit * par_value);
    let asw_bps = asw * 10_000.0;
    if !asw_bps.is_finite() { return None; }
    Some(AssetSwapReport {
        asset_swap_spread_bps: asw_bps,
        annuity,
        par_value_difference: par_value_diff,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cf(t: f64, acc: f64, df: f64) -> CashFlow {
        CashFlow { time_years: t, accrual: acc, discount_factor: df }
    }

    fn typical_cfs() -> Vec<CashFlow> {
        // 5y semi-annual bond, flat 4% swap curve → DF_i = exp(-0.04·t_i).
        (1..=10).map(|i| {
            let t = i as f64 / 2.0;
            cf(t, 0.5, (-0.04_f64 * t).exp())
        }).collect()
    }

    #[test]
    fn invalid_inputs_return_none() {
        let cfs = typical_cfs();
        assert!(analyze(0.0, 100.0, 0.05, 0.04, &cfs).is_none());
        assert!(analyze(100.0, 0.0, 0.05, 0.04, &cfs).is_none());
        assert!(analyze(100.0, 100.0, -0.01, 0.04, &cfs).is_none());
        assert!(analyze(100.0, 100.0, 0.05, 0.04, &[]).is_none());
        assert!(analyze(f64::NAN, 100.0, 0.05, 0.04, &cfs).is_none());
    }

    #[test]
    fn par_priced_bond_with_matching_coupon_yields_zero_spread() {
        // Bond priced at par AND coupon = par swap rate → ASW = 0.
        let cfs = typical_cfs();
        let r = analyze(100.0, 100.0, 0.04, 0.04, &cfs).unwrap();
        assert!(r.asset_swap_spread_bps.abs() < 1e-6);
    }

    #[test]
    fn premium_bond_yields_negative_spread() {
        // Bond trades at premium (e.g. coupon > swap rate AND priced > par).
        let cfs = typical_cfs();
        let r = analyze(105.0, 100.0, 0.05, 0.04, &cfs).unwrap();
        // ASW = swap_rate − coupon + (par − price)/annuity_per_unit
        //     = 4% − 5% + (100−105)/annuity_per_unit < 0.
        assert!(r.asset_swap_spread_bps < 0.0);
    }

    #[test]
    fn discount_bond_yields_positive_spread() {
        let cfs = typical_cfs();
        let r = analyze(95.0, 100.0, 0.03, 0.04, &cfs).unwrap();
        // ASW = 4% − 3% + (100−95)/annuity_per_unit > 0.
        assert!(r.asset_swap_spread_bps > 0.0);
    }

    #[test]
    fn higher_coupon_at_par_lowers_asw() {
        let cfs = typical_cfs();
        let r_low_coupon  = analyze(100.0, 100.0, 0.03, 0.04, &cfs).unwrap();
        let r_high_coupon = analyze(100.0, 100.0, 0.06, 0.04, &cfs).unwrap();
        assert!(r_low_coupon.asset_swap_spread_bps > r_high_coupon.asset_swap_spread_bps);
    }

    #[test]
    fn annuity_positive_for_valid_cashflows() {
        let cfs = typical_cfs();
        let r = analyze(100.0, 100.0, 0.04, 0.04, &cfs).unwrap();
        assert!(r.annuity > 0.0);
    }
}
