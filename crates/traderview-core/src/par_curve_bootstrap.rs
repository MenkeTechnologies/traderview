//! Par-curve bootstrap — zero rates, discount factors, and implied
//! forwards from a par yield curve.
//!
//! Input: par rates (decimal) for consecutive annual tenors 1..N with
//! annual coupons. A par bond prices at 100, so each tenor pins one
//! new discount factor:
//!
//!   1 = c_n · Σ_{i=1..n} DF_i + DF_n
//!   DF_n = (1 − c_n · Σ_{i=1..n−1} DF_i) / (1 + c_n)
//!
//! Zero rate (annual comp):  z_n = DF_n^{−1/n} − 1
//! 1y implied forward:       f_{n−1,n} = DF_{n−1}/DF_n − 1
//!
//! A flat par curve bootstraps to an identical flat zero curve; an
//! upward par curve puts long zeros above par (coupon effect).
//!
//! Related to but DISTINCT from `yield_curve_bootstrap`: that module
//! exact-fits raw coupon-bond cash flows under a no-arbitrage DF ≤ 1
//! constraint (continuously-compounded zeros, rejects negative-rate
//! curves by design — pinned by its arbitrage_violation test). This
//! one speaks the par-rate convention with annual compounding and
//! supports negative-rate regimes (DF > 1), which that contract
//! forbids — hence the standalone recurrence instead of delegation.
//!
//! Pure compute. Companion to `nelson_siegel`, `fra`, `bond_convexity`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CurvePoint {
    pub tenor_years: u32,
    pub par_rate: f64,
    pub discount_factor: f64,
    /// Annually-compounded zero rate.
    pub zero_rate: f64,
    /// 1y forward starting at tenor − 1.
    pub forward_rate: f64,
}

/// `par_rates[i]` = par rate (decimal) for tenor i+1 years.
pub fn compute(par_rates: &[f64]) -> Option<Vec<CurvePoint>> {
    if par_rates.is_empty()
        || par_rates.len() > 100
        || par_rates.iter().any(|r| !r.is_finite() || *r <= -1.0)
    {
        return None;
    }
    let mut out = Vec::with_capacity(par_rates.len());
    let mut annuity = 0.0_f64; // Σ DF_i for i < n
    let mut prev_df = 1.0_f64;
    for (i, &c) in par_rates.iter().enumerate() {
        let df = (1.0 - c * annuity) / (1.0 + c);
        if !df.is_finite() || df <= 0.0 {
            // Par rate so high the curve implies a negative discount
            // factor — inconsistent input.
            return None;
        }
        let n = (i + 1) as f64;
        out.push(CurvePoint {
            tenor_years: i as u32 + 1,
            par_rate: c,
            discount_factor: df,
            zero_rate: df.powf(-1.0 / n) - 1.0,
            forward_rate: prev_df / df - 1.0,
        });
        annuity += df;
        prev_df = df;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_par_curve_bootstraps_to_flat_zeros_and_forwards() {
        let c = 0.05;
        let pts = compute(&[c; 10]).unwrap();
        for p in &pts {
            assert!((p.zero_rate - c).abs() < 1e-12, "{p:?}");
            assert!((p.forward_rate - c).abs() < 1e-12, "{p:?}");
            // DF must equal (1+c)^-n exactly on a flat curve.
            let want = (1.0_f64 + c).powi(-(p.tenor_years as i32));
            assert!((p.discount_factor - want).abs() < 1e-12);
        }
    }

    #[test]
    fn two_point_curve_matches_hand_bootstrap() {
        // c1 = 2%, c2 = 4%:
        //   DF1 = 1/1.02
        //   DF2 = (1 − 0.04·DF1)/1.04
        let pts = compute(&[0.02, 0.04]).unwrap();
        let df1 = 1.0 / 1.02_f64;
        let df2 = (1.0 - 0.04 * df1) / 1.04;
        assert!((pts[0].discount_factor - df1).abs() < 1e-12);
        assert!((pts[1].discount_factor - df2).abs() < 1e-12);
        assert!((pts[1].zero_rate - (df2.powf(-0.5) - 1.0)).abs() < 1e-12);
        assert!((pts[1].forward_rate - (df1 / df2 - 1.0)).abs() < 1e-12);
    }

    #[test]
    fn upward_par_curve_puts_long_zeros_above_par() {
        // Coupon effect: with rising par rates, the n-year zero must
        // exceed the n-year par rate (n > 1).
        let pts = compute(&[0.02, 0.03, 0.04, 0.05]).unwrap();
        for p in pts.iter().skip(1) {
            assert!(p.zero_rate > p.par_rate, "{p:?}");
        }
        // Forwards rise faster still.
        assert!(pts[3].forward_rate > pts[3].zero_rate);
    }

    #[test]
    fn negative_rate_regimes_supported() {
        // Eurozone-style mildly negative front end must bootstrap with
        // DF > 1 and matching zero.
        let pts = compute(&[-0.005, 0.0, 0.01]).unwrap();
        assert!(pts[0].discount_factor > 1.0);
        assert!((pts[0].zero_rate + 0.005).abs() < 1e-12);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[f64::NAN]).is_none());
        assert!(compute(&[-1.5]).is_none());
        assert!(compute(&vec![0.05; 101]).is_none());
        // Absurd par rate that drives DF negative downstream.
        assert!(compute(&[0.02, 5.0, 5.0, 5.0]).is_none());
    }
}
