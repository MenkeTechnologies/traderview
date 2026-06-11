//! Hedge-fund fee waterfall — what 2-and-20 actually costs.
//!
//! Conventions (stated, since funds vary):
//!   - management fee charged on BEGINNING-of-year NAV
//!   - incentive = rate × max(nav_after_mgmt − threshold, 0), where
//!     threshold = max(high-water mark, begin × (1 + hurdle))
//!   - HWM ratchets to the net NAV when exceeded; no clawback
//!
//! Walks a multi-year gross-return path and reports per-year fees, the
//! investor's net CAGR vs gross, and the annualized fee drag — the
//! number the "2-and-20 eats half your alpha" argument is about.
//!
//! Pure compute. Companion to `high_water_mark` (per-period HWM
//! tracking on equity curves).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FundFeesInput {
    pub initial_investment: f64,
    /// Gross fund returns per year, % (e.g. [20, -10, 15]).
    pub gross_returns_pct: Vec<f64>,
    /// Management fee, % of beginning NAV (the "2").
    pub management_fee_pct: f64,
    /// Incentive fee, % of gains above the threshold (the "20").
    pub incentive_fee_pct: f64,
    /// Hurdle rate, %/yr (0 = none).
    #[serde(default)]
    pub hurdle_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeeYear {
    pub year: usize,
    pub gross_return_pct: f64,
    pub management_fee: f64,
    pub incentive_fee: f64,
    pub net_nav: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FundFeesReport {
    pub years: Vec<FeeYear>,
    pub final_net_nav: f64,
    pub final_gross_nav: f64,
    pub total_fees: f64,
    pub gross_cagr_pct: f64,
    pub net_cagr_pct: f64,
    /// gross − net CAGR, pp/yr.
    pub fee_drag_pp: f64,
}

pub fn compute(inp: &FundFeesInput) -> Option<FundFeesReport> {
    if !inp.initial_investment.is_finite()
        || inp.initial_investment <= 0.0
        || inp.gross_returns_pct.is_empty()
        || inp.gross_returns_pct.len() > 100
        || inp.gross_returns_pct.iter().any(|r| !r.is_finite() || *r <= -100.0)
        || !inp.management_fee_pct.is_finite()
        || !(0.0..100.0).contains(&inp.management_fee_pct)
        || !inp.incentive_fee_pct.is_finite()
        || !(0.0..100.0).contains(&inp.incentive_fee_pct)
        || !inp.hurdle_pct.is_finite()
        || inp.hurdle_pct < 0.0
    {
        return None;
    }
    let mut nav = inp.initial_investment;
    let mut gross_nav = inp.initial_investment;
    let mut hwm = inp.initial_investment;
    let mut total_fees = 0.0;
    let mut years = Vec::with_capacity(inp.gross_returns_pct.len());
    for (i, &r) in inp.gross_returns_pct.iter().enumerate() {
        let begin = nav;
        let growth = 1.0 + r / 100.0;
        gross_nav *= growth;
        let mgmt = begin * inp.management_fee_pct / 100.0;
        let after_mgmt = begin * growth - mgmt;
        let threshold = hwm.max(begin * (1.0 + inp.hurdle_pct / 100.0));
        let incentive = inp.incentive_fee_pct / 100.0 * (after_mgmt - threshold).max(0.0);
        nav = after_mgmt - incentive;
        if nav <= 0.0 {
            return None; // fees + losses wiped the account — degenerate
        }
        hwm = hwm.max(nav);
        total_fees += mgmt + incentive;
        years.push(FeeYear {
            year: i + 1,
            gross_return_pct: r,
            management_fee: mgmt,
            incentive_fee: incentive,
            net_nav: nav,
        });
    }
    let n = inp.gross_returns_pct.len() as f64;
    let cagr = |fin: f64| ((fin / inp.initial_investment).powf(1.0 / n) - 1.0) * 100.0;
    let gross_cagr = cagr(gross_nav);
    let net_cagr = cagr(nav);
    Some(FundFeesReport {
        years,
        final_net_nav: nav,
        final_gross_nav: gross_nav,
        total_fees,
        gross_cagr_pct: gross_cagr,
        net_cagr_pct: net_cagr,
        fee_drag_pp: gross_cagr - net_cagr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(returns: Vec<f64>) -> FundFeesInput {
        FundFeesInput {
            initial_investment: 100.0,
            gross_returns_pct: returns,
            management_fee_pct: 2.0,
            incentive_fee_pct: 20.0,
            hurdle_pct: 0.0,
        }
    }

    #[test]
    fn single_up_year_two_and_twenty_hand_walk() {
        // +20% on 100: mgmt 2, after-mgmt 118, incentive 20%·18 = 3.6,
        // net 114.4 — a 20% gross year nets 14.4%.
        let r = compute(&base(vec![20.0])).unwrap();
        let y = &r.years[0];
        assert!((y.management_fee - 2.0).abs() < 1e-12);
        assert!((y.incentive_fee - 3.6).abs() < 1e-12);
        assert!((r.final_net_nav - 114.4).abs() < 1e-12);
        assert!((r.net_cagr_pct - 14.4).abs() < 1e-9);
        assert!((r.fee_drag_pp - (20.0 - 14.4)).abs() < 1e-9);
    }

    #[test]
    fn high_water_mark_blocks_double_charging() {
        // −10% then +15%: year 1 pays mgmt only (no gains); year 2's
        // incentive applies only ABOVE the original 100 HWM.
        let r = compute(&base(vec![-10.0, 15.0])).unwrap();
        let y1 = &r.years[0];
        assert_eq!(y1.incentive_fee, 0.0);
        // Net after year 1: 90 − 2 = 88. Year 2: mgmt 1.76,
        // after-mgmt = 88·1.15 − 1.76 = 99.44 < HWM 100 ⇒ NO incentive
        // even on a +15% year.
        let y2 = &r.years[1];
        assert!((y2.management_fee - 1.76).abs() < 1e-12);
        assert_eq!(y2.incentive_fee, 0.0);
        assert!((r.final_net_nav - 99.44).abs() < 1e-12);
    }

    #[test]
    fn hurdle_exempts_returns_below_it() {
        // 5% hurdle, +10% year: after-mgmt 108, threshold 105,
        // incentive 20%·3 = 0.6.
        let mut inp = base(vec![10.0]);
        inp.hurdle_pct = 5.0;
        let r = compute(&inp).unwrap();
        assert!((r.years[0].incentive_fee - 0.6).abs() < 1e-12);
    }

    #[test]
    fn fees_compound_into_material_drag() {
        // Ten +10% years at 2/20: the drag must exceed 3.5pp/yr.
        let r = compute(&base(vec![10.0; 10])).unwrap();
        assert!(r.fee_drag_pp > 3.5, "{}", r.fee_drag_pp);
        assert!(r.final_net_nav < r.final_gross_nav);
        assert!(r.total_fees > 0.0);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&base(vec![])).is_none());
        assert!(compute(&base(vec![-100.0])).is_none());
        let mut bad = base(vec![10.0]);
        bad.management_fee_pct = 100.0;
        assert!(compute(&bad).is_none());
        let mut bad = base(vec![10.0]);
        bad.initial_investment = 0.0;
        assert!(compute(&bad).is_none());
        // Catastrophic loss + fees wiping the account is degenerate.
        let mut bad = base(vec![-99.0]);
        bad.management_fee_pct = 2.0;
        assert!(compute(&bad).is_none());
    }
}
