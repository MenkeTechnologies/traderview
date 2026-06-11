//! Asset location — which asset suffers the most tax drag in a
//! taxable account, and therefore earns the sheltered slot first.
//!
//! Framed as ASSET drag (not account comparison) to dodge the
//! contribution-asymmetry trap: for an asset returning `growth` +
//! `yield` per year held `years` in taxable,
//!
//!   - the yield is taxed annually at the ordinary rate and
//!     reinvested (raising basis),
//!   - the growth is taxed once at the capital-gains rate on the
//!     terminal unrealized gain.
//!
//! Tax drag = pre-tax CAGR − after-tax CAGR. High-yield assets
//! (bonds, REITs) drag multiples of what low-turnover growth drags —
//! the classic result that bonds go in the IRA.
//!
//! Pure compute. Companion to `tax_loss_harvest`, `fund_fees`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AssetLocationInput {
    /// Price appreciation, %/yr (taxed at cap-gains on liquidation).
    pub growth_pct: f64,
    /// Income yield, %/yr (taxed annually at the ordinary rate).
    pub yield_pct: f64,
    /// Marginal ordinary-income rate, %.
    pub ordinary_rate_pct: f64,
    /// Long-term capital-gains rate, %.
    pub cap_gains_rate_pct: f64,
    pub years: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetLocationReport {
    pub pre_tax_cagr_pct: f64,
    pub taxable_after_tax_cagr_pct: f64,
    /// pre-tax − after-tax CAGR: the annual cost of holding this
    /// asset in taxable — rank assets by this to fill the IRA.
    pub tax_drag_pp: f64,
    pub final_value_pre_tax: f64,
    pub final_value_taxable: f64,
}

pub fn compute(inp: &AssetLocationInput) -> Option<AssetLocationReport> {
    if ![
        inp.growth_pct,
        inp.yield_pct,
        inp.ordinary_rate_pct,
        inp.cap_gains_rate_pct,
    ]
    .iter()
    .all(|v| v.is_finite())
        || inp.growth_pct <= -100.0
        || inp.yield_pct < 0.0
        || !(0.0..100.0).contains(&inp.ordinary_rate_pct)
        || !(0.0..100.0).contains(&inp.cap_gains_rate_pct)
        || inp.years == 0
        || inp.years > 100
    {
        return None;
    }
    let g = inp.growth_pct / 100.0;
    let y = inp.yield_pct / 100.0;
    let t_ord = inp.ordinary_rate_pct / 100.0;
    let t_cg = inp.cap_gains_rate_pct / 100.0;
    let n = inp.years as f64;
    // Pre-tax: full reinvestment of growth + yield.
    let pre_tax = (1.0 + g + y).powf(n);
    // Taxable: value compounds at g + y(1−t_ord); basis grows by the
    // reinvested after-tax yield each year; terminal cap-gains tax on
    // (value − basis).
    let mut value = 1.0_f64;
    let mut basis = 1.0_f64;
    for _ in 0..inp.years {
        let income_after_tax = value * y * (1.0 - t_ord);
        value = value * (1.0 + g) + income_after_tax;
        basis += income_after_tax;
    }
    let after_liquidation = value - (value - basis).max(0.0) * t_cg;
    if after_liquidation <= 0.0 {
        return None;
    }
    let pre_cagr = (pre_tax.powf(1.0 / n) - 1.0) * 100.0;
    let post_cagr = (after_liquidation.powf(1.0 / n) - 1.0) * 100.0;
    Some(AssetLocationReport {
        pre_tax_cagr_pct: pre_cagr,
        taxable_after_tax_cagr_pct: post_cagr,
        tax_drag_pp: pre_cagr - post_cagr,
        final_value_pre_tax: pre_tax,
        final_value_taxable: after_liquidation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_yield_bond_drags_at_the_ordinary_rate() {
        // 6% all-yield, 35% ordinary, 1 year, no growth: after-tax
        // 3.9% exactly — 2.1pp drag, no cap-gains layer (basis = value).
        let r = compute(&AssetLocationInput {
            growth_pct: 0.0,
            yield_pct: 6.0,
            ordinary_rate_pct: 35.0,
            cap_gains_rate_pct: 15.0,
            years: 1,
        })
        .unwrap();
        assert!((r.taxable_after_tax_cagr_pct - 3.9).abs() < 1e-9);
        assert!((r.tax_drag_pp - 2.1).abs() < 1e-9);
    }

    #[test]
    fn pure_growth_stock_hand_walk() {
        // 6% all-growth, 15% cap gains, 10 years: 1.06^10 = 1.79085,
        // tax 0.15·0.79085, net 1.67222 ⇒ CAGR ≈ 5.276%.
        let r = compute(&AssetLocationInput {
            growth_pct: 6.0,
            yield_pct: 0.0,
            ordinary_rate_pct: 35.0,
            cap_gains_rate_pct: 15.0,
            years: 10,
        })
        .unwrap();
        let gross = 1.06_f64.powi(10);
        let net = gross - (gross - 1.0) * 0.15;
        let want = (net.powf(0.1) - 1.0) * 100.0;
        assert!((r.taxable_after_tax_cagr_pct - want).abs() < 1e-9);
        assert!(r.tax_drag_pp < 1.0); // sub-1pp drag
    }

    #[test]
    fn bonds_outdrag_growth_the_classic_location_result() {
        // Same 6% total return: the all-yield version drags more than
        // the all-growth version ⇒ bonds claim the sheltered slot.
        let mk = |g: f64, y: f64| {
            compute(&AssetLocationInput {
                growth_pct: g,
                yield_pct: y,
                ordinary_rate_pct: 35.0,
                cap_gains_rate_pct: 15.0,
                years: 20,
            })
            .unwrap()
        };
        let bond = mk(0.0, 6.0);
        let stock = mk(6.0, 0.0);
        assert!(bond.tax_drag_pp > stock.tax_drag_pp, "{} vs {}", bond.tax_drag_pp, stock.tax_drag_pp);
    }

    #[test]
    fn longer_deferral_shrinks_growth_drag() {
        let mk = |years: u32| {
            compute(&AssetLocationInput {
                growth_pct: 6.0,
                yield_pct: 0.0,
                ordinary_rate_pct: 35.0,
                cap_gains_rate_pct: 15.0,
                years,
            })
            .unwrap()
            .tax_drag_pp
        };
        assert!(mk(30) < mk(5)); // deferral compounds pre-tax
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&AssetLocationInput {
            growth_pct: 6.0,
            yield_pct: 0.0,
            ordinary_rate_pct: 100.0,
            cap_gains_rate_pct: 15.0,
            years: 10,
        })
        .is_none());
        assert!(compute(&AssetLocationInput {
            growth_pct: 6.0,
            yield_pct: -1.0,
            ordinary_rate_pct: 35.0,
            cap_gains_rate_pct: 15.0,
            years: 10,
        })
        .is_none());
        assert!(compute(&AssetLocationInput {
            growth_pct: 6.0,
            yield_pct: 0.0,
            ordinary_rate_pct: 35.0,
            cap_gains_rate_pct: 15.0,
            years: 0,
        })
        .is_none());
    }
}
