//! Bond Convexity — second-order sensitivity of bond price to yield.
//!
//! Modified duration is the linear (first-order) approximation;
//! convexity is the quadratic correction:
//!
//!   ΔP/P ≈ −D_mod · Δy + ½ · C · (Δy)²
//!
//! For a series of cash flows {(t_i, c_i)} discounted at YTM y (with
//! `compounding_periods_per_year` m):
//!
//!   PV  = Σ c_i · (1 + y/m)^(−m·t_i)
//!   C   = (1 / PV) · Σ t_i · (t_i + 1/m) · c_i · (1 + y/m)^(−m·t_i − 2)
//!
//! Effective convexity (numerical via ±Δy bumps):
//!
//!   C_eff = (P_dn + P_up − 2P_0) / (P_0 · (Δy)²)
//!
//! Returns both the closed-form analytic and the numerical effective
//! convexity for cross-validation.
//!
//! Pure compute. Companion to `macaulay_duration`, `key_rate_duration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvexityReport {
    pub present_value: f64,
    pub modified_duration: f64,
    pub analytic_convexity: f64,
    pub effective_convexity: f64,
    pub convexity_adjustment_50bp: f64,
}

pub fn compute(
    cash_flows: &[CashFlow],
    ytm: f64,
    compounding_periods_per_year: u32,
) -> Option<ConvexityReport> {
    if cash_flows.is_empty() || !ytm.is_finite() || compounding_periods_per_year == 0 {
        return None;
    }
    if cash_flows
        .iter()
        .any(|c| !c.time_years.is_finite() || c.time_years < 0.0 || !c.amount.is_finite())
    {
        return None;
    }
    let m = compounding_periods_per_year as f64;
    let one_plus_y_m = 1.0 + ytm / m;
    if one_plus_y_m <= 0.0 {
        return None;
    }
    let pv: f64 = cash_flows
        .iter()
        .map(|c| c.amount * one_plus_y_m.powf(-m * c.time_years))
        .sum();
    if pv <= 0.0 {
        return None;
    }
    // Analytic Macaulay duration first, then modified.
    let mac_dur: f64 = cash_flows
        .iter()
        .map(|c| {
            let disc = one_plus_y_m.powf(-m * c.time_years);
            c.time_years * c.amount * disc
        })
        .sum::<f64>()
        / pv;
    let mod_dur = mac_dur / one_plus_y_m;
    // Analytic convexity: weighted sum of t·(t + 1/m) at (1+y/m)^(−mt−2).
    let analytic: f64 = cash_flows
        .iter()
        .map(|c| {
            let t = c.time_years;
            let disc = one_plus_y_m.powf(-m * t - 2.0);
            t * (t + 1.0 / m) * c.amount * disc
        })
        .sum::<f64>()
        / pv;
    // Effective convexity via numerical ±1 bp.
    let dy = 0.0001_f64;
    let pv_up = pv_at(cash_flows, ytm + dy, m)?;
    let pv_dn = pv_at(cash_flows, ytm - dy, m)?;
    let effective = (pv_up + pv_dn - 2.0 * pv) / (pv * dy * dy);
    // Convexity adjustment example for a 50bp move: ½ · C · Δy².
    let cx_adj_50bp = 0.5 * analytic * 0.005_f64.powi(2);
    Some(ConvexityReport {
        present_value: pv,
        modified_duration: mod_dur,
        analytic_convexity: analytic,
        effective_convexity: effective,
        convexity_adjustment_50bp: cx_adj_50bp,
    })
}

fn pv_at(cf: &[CashFlow], y: f64, m: f64) -> Option<f64> {
    let g = 1.0 + y / m;
    if g <= 0.0 {
        return None;
    }
    Some(
        cf.iter()
            .map(|c| c.amount * g.powf(-m * c.time_years))
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ten_year_5pct() -> Vec<CashFlow> {
        let mut cf = Vec::new();
        for t in 1..=10 {
            cf.push(CashFlow {
                time_years: t as f64,
                amount: 5.0,
            });
        }
        cf.last_mut().unwrap().amount = 105.0;
        cf
    }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[], 0.05, 1).is_none());
        let cf = ten_year_5pct();
        assert!(compute(&cf, 0.05, 0).is_none());
        assert!(compute(&cf, f64::NAN, 1).is_none());
        // YTM that makes (1+y/m) <= 0:
        assert!(compute(&cf, -1.5, 1).is_none());
    }

    #[test]
    fn nan_cash_flow_returns_none() {
        let bad = vec![CashFlow {
            time_years: 1.0,
            amount: f64::NAN,
        }];
        assert!(compute(&bad, 0.05, 1).is_none());
    }

    #[test]
    fn par_bond_pv_equals_face() {
        // 10y 5% bond at 5% YTM → PV = par = 100.
        let cf = ten_year_5pct();
        let r = compute(&cf, 0.05, 1).unwrap();
        assert!((r.present_value - 100.0).abs() < 1e-6);
    }

    #[test]
    fn analytic_and_effective_convexity_agree() {
        let cf = ten_year_5pct();
        let r = compute(&cf, 0.05, 1).unwrap();
        // The two estimators should agree to a few percent.
        let rel_diff = (r.analytic_convexity - r.effective_convexity).abs() / r.analytic_convexity;
        assert!(
            rel_diff < 0.02,
            "analytic {} vs effective {} differ by {:.2}%",
            r.analytic_convexity,
            r.effective_convexity,
            rel_diff * 100.0
        );
    }

    #[test]
    fn convexity_positive_for_vanilla_bond() {
        let cf = ten_year_5pct();
        let r = compute(&cf, 0.05, 1).unwrap();
        assert!(r.analytic_convexity > 0.0);
        assert!(r.effective_convexity > 0.0);
    }

    #[test]
    fn longer_maturity_yields_higher_convexity() {
        // 30y bond should have higher convexity than 10y bond at same coupon.
        let cf10 = ten_year_5pct();
        let mut cf30 = Vec::new();
        for t in 1..=30 {
            cf30.push(CashFlow {
                time_years: t as f64,
                amount: 5.0,
            });
        }
        cf30.last_mut().unwrap().amount = 105.0;
        let r10 = compute(&cf10, 0.05, 1).unwrap();
        let r30 = compute(&cf30, 0.05, 1).unwrap();
        assert!(
            r30.analytic_convexity > r10.analytic_convexity,
            "30y convexity {} should exceed 10y {}",
            r30.analytic_convexity,
            r10.analytic_convexity
        );
    }

    #[test]
    fn modified_duration_smaller_than_maturity() {
        let cf = ten_year_5pct();
        let r = compute(&cf, 0.05, 1).unwrap();
        // 10y 5% bond modified duration is ~7.7 years, below maturity.
        assert!(r.modified_duration < 10.0);
        assert!(r.modified_duration > 5.0);
    }

    #[test]
    fn convexity_adjustment_for_50bp_positive() {
        let cf = ten_year_5pct();
        let r = compute(&cf, 0.05, 1).unwrap();
        // ½ · C · (0.005)² > 0 for any positive convexity.
        assert!(r.convexity_adjustment_50bp > 0.0);
    }
}
