//! Leveraged-ETF decay — what daily reset does to a k× fund.
//!
//! Under the lognormal approximation, a k× daily-reset ETF over period
//! T with index total return R and annualized vol σ compounds to
//!
//!   (1 + R_letf) ≈ (1 + R)^k · e^{−k(k−1)/2 · σ² · T}
//!
//! Two effects, both reported separately:
//!   - compounding of the levered exposure: (1+R)^k vs the naive k·R
//!     (HELPS in smooth trends, hurts in chop), and
//!   - the volatility drag e^{−k(k−1)/2·σ²T}, always ≤ 1 for k > 1 —
//!     the reason 3× funds bleed in sideways tape.
//!
//! Pure compute. Companion to `vol_targeting_sizer`, `monte_carlo`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LetfInput {
    /// Leverage factor (2, 3, −1 for inverse, −2…).
    pub leverage: f64,
    /// Index total return over the period, % (e.g. 10).
    pub index_return_pct: f64,
    /// Annualized index vol, %.
    pub index_vol_pct: f64,
    /// Period, years.
    pub years: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LetfReport {
    /// Expected k× fund return, % (lognormal approx).
    pub letf_return_pct: f64,
    /// The naive k·R the marketing implies.
    pub naive_return_pct: f64,
    /// Vol drag in return points: levered-compounded minus actual.
    pub vol_drag_pp: f64,
    /// Levered compounding alone, no vol: (1+R)^k − 1.
    pub compounded_no_vol_pct: f64,
}

pub fn compute(inp: &LetfInput) -> Option<LetfReport> {
    if !inp.leverage.is_finite()
        || inp.leverage == 0.0
        || inp.leverage.abs() > 10.0
        || !inp.index_return_pct.is_finite()
        || inp.index_return_pct <= -100.0
        || !inp.index_vol_pct.is_finite()
        || inp.index_vol_pct < 0.0
        || !inp.years.is_finite()
        || inp.years <= 0.0
    {
        return None;
    }
    let k = inp.leverage;
    let growth = 1.0 + inp.index_return_pct / 100.0;
    let sigma2 = (inp.index_vol_pct / 100.0).powi(2);
    let compounded = growth.powf(k);
    let drag_factor = (-k * (k - 1.0) / 2.0 * sigma2 * inp.years).exp();
    let letf = compounded * drag_factor - 1.0;
    Some(LetfReport {
        letf_return_pct: letf * 100.0,
        naive_return_pct: k * inp.index_return_pct,
        vol_drag_pp: (compounded - 1.0) * 100.0 - letf * 100.0,
        compounded_no_vol_pct: (compounded - 1.0) * 100.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_vol_is_pure_levered_compounding() {
        // +10% index at 2× with no vol: (1.1)² − 1 = 21% — BETTER than
        // the naive 20% (smooth trends compound in your favor).
        let r = compute(&LetfInput {
            leverage: 2.0,
            index_return_pct: 10.0,
            index_vol_pct: 0.0,
            years: 1.0,
        })
        .unwrap();
        assert!((r.letf_return_pct - 21.0).abs() < 1e-9);
        assert!((r.naive_return_pct - 20.0).abs() < 1e-12);
        assert!(r.vol_drag_pp.abs() < 1e-12);
    }

    #[test]
    fn flat_choppy_index_bleeds_a_three_x() {
        // Flat index, 40% vol, 1y at 3×: drag = e^{−3·0.16} − 1 ≈
        // −38.1% while the index went nowhere.
        let r = compute(&LetfInput {
            leverage: 3.0,
            index_return_pct: 0.0,
            index_vol_pct: 40.0,
            years: 1.0,
        })
        .unwrap();
        let want = ((-3.0_f64 * 0.16).exp() - 1.0) * 100.0;
        assert!((r.letf_return_pct - want).abs() < 1e-9, "{}", r.letf_return_pct);
        assert!(r.letf_return_pct < -38.0);
        assert!((r.naive_return_pct).abs() < 1e-12);
    }

    #[test]
    fn inverse_funds_decay_too() {
        // k = −1: k(k−1)/2 = 1 > 0 ⇒ drag exists even at 1× inverse.
        let r = compute(&LetfInput {
            leverage: -1.0,
            index_return_pct: 0.0,
            index_vol_pct: 30.0,
            years: 1.0,
        })
        .unwrap();
        assert!(r.letf_return_pct < 0.0, "{}", r.letf_return_pct);
        let want = ((-0.09_f64).exp() - 1.0) * 100.0;
        assert!((r.letf_return_pct - want).abs() < 1e-9);
    }

    #[test]
    fn more_vol_more_drag_monotone() {
        let lo = compute(&LetfInput {
            leverage: 2.0,
            index_return_pct: 10.0,
            index_vol_pct: 15.0,
            years: 1.0,
        })
        .unwrap();
        let hi = compute(&LetfInput {
            leverage: 2.0,
            index_return_pct: 10.0,
            index_vol_pct: 45.0,
            years: 1.0,
        })
        .unwrap();
        assert!(hi.vol_drag_pp > lo.vol_drag_pp);
        assert!(hi.letf_return_pct < lo.letf_return_pct);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&LetfInput {
            leverage: 0.0,
            index_return_pct: 10.0,
            index_vol_pct: 20.0,
            years: 1.0,
        })
        .is_none());
        assert!(compute(&LetfInput {
            leverage: 2.0,
            index_return_pct: -100.0,
            index_vol_pct: 20.0,
            years: 1.0,
        })
        .is_none());
        assert!(compute(&LetfInput {
            leverage: 2.0,
            index_return_pct: 10.0,
            index_vol_pct: f64::NAN,
            years: 1.0,
        })
        .is_none());
    }
}
