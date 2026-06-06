//! Burke Ratio — return per unit of drawdown-vol (Burke 1994).
//!
//!   Burke = (R_p − R_f) / √(Σ DD_i²)
//!
//! where DD_i are the per-trough drawdowns from each peak.
//!
//! Compared with Sharpe (uses full vol), Burke penalizes only the
//! drawdown component of variance — appealing to investors who view
//! upside volatility as desirable.
//!
//! Modified Burke (annualized): multiply numerator and denominator
//! by √periods_per_year to keep ratio scale-invariant.
//!
//! Pure compute. Companion to `sterling_ratio`, `recovery_factor`,
//! `pain_index`, `ulcer_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BurkeReport {
    pub burke_ratio: f64,
    pub modified_burke_ratio: f64,
    pub total_return: f64,
    pub n_drawdowns: usize,
    pub sum_squared_drawdowns: f64,
}

pub fn compute(equity: &[f64], risk_free_total: f64, periods_per_year: f64) -> Option<BurkeReport> {
    if equity.len() < 2
        || !risk_free_total.is_finite()
        || !periods_per_year.is_finite()
        || periods_per_year <= 0.0
    {
        return None;
    }
    if equity.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    let start = equity[0];
    let end = *equity.last().unwrap();
    let total_return = end / start - 1.0;
    let excess = total_return - risk_free_total;
    // Identify per-trough drawdowns as the deepest DD between each pair
    // of consecutive new highs.
    let mut hwm = start;
    let mut current_trough_dd = 0.0_f64;
    let mut drawdowns = Vec::new();
    for &v in &equity[1..] {
        if v > hwm {
            if current_trough_dd > 0.0 {
                drawdowns.push(current_trough_dd);
            }
            hwm = v;
            current_trough_dd = 0.0;
        } else {
            let dd = (hwm - v) / hwm;
            if dd > current_trough_dd {
                current_trough_dd = dd;
            }
        }
    }
    if current_trough_dd > 0.0 {
        drawdowns.push(current_trough_dd);
    }
    let sum_sq_dd: f64 = drawdowns.iter().map(|d| d * d).sum();
    let burke = if sum_sq_dd > 0.0 {
        excess / sum_sq_dd.sqrt()
    } else {
        0.0
    };
    let mod_burke = burke * periods_per_year.sqrt();
    Some(BurkeReport {
        burke_ratio: burke,
        modified_burke_ratio: mod_burke,
        total_return,
        n_drawdowns: drawdowns.len(),
        sum_squared_drawdowns: sum_sq_dd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[100.0], 0.0, 252.0).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[100.0, 110.0], 0.0, 0.0).is_none());
        assert!(compute(&[100.0, f64::NAN], 0.0, 252.0).is_none());
        assert!(compute(&[100.0, 0.0], 0.0, 252.0).is_none());
        assert!(compute(&[100.0, -10.0], 0.0, 252.0).is_none());
    }

    #[test]
    fn monotone_uptrend_yields_zero_drawdowns() {
        let eq: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert_eq!(r.n_drawdowns, 0);
        assert_eq!(r.sum_squared_drawdowns, 0.0);
        // No drawdowns → burke = 0 (per implementation convention).
        assert_eq!(r.burke_ratio, 0.0);
    }

    #[test]
    fn single_drawdown_recorded() {
        // Peak 110, trough 99 → DD = 10%, then full recovery to 120.
        let eq = vec![100.0, 110.0, 99.0, 120.0];
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert_eq!(r.n_drawdowns, 1);
        let expected_dd = (110.0_f64 - 99.0) / 110.0;
        assert!((r.sum_squared_drawdowns - expected_dd * expected_dd).abs() < 1e-12);
    }

    #[test]
    fn multiple_drawdowns_summed() {
        // Two distinct DD episodes between new highs.
        let eq = vec![100.0, 110.0, 95.0, 115.0, 100.0, 120.0];
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert_eq!(r.n_drawdowns, 2);
        let dd_1 = (110.0_f64 - 95.0) / 110.0;
        let dd_2 = (115.0_f64 - 100.0) / 115.0;
        let expected = dd_1 * dd_1 + dd_2 * dd_2;
        assert!((r.sum_squared_drawdowns - expected).abs() < 1e-12);
    }

    #[test]
    fn risk_free_offset_subtracted() {
        let eq = vec![100.0, 110.0, 95.0, 120.0];
        let r0 = compute(&eq, 0.0, 252.0).unwrap();
        let r5 = compute(&eq, 0.05, 252.0).unwrap();
        // Higher rf → lower excess → smaller (or more-negative) burke.
        assert!(r5.burke_ratio < r0.burke_ratio);
    }

    #[test]
    fn modified_burke_scales_with_periods_per_year() {
        let eq = vec![100.0, 110.0, 95.0, 120.0];
        let r12 = compute(&eq, 0.0, 12.0).unwrap();
        let r252 = compute(&eq, 0.0, 252.0).unwrap();
        let ratio = r252.modified_burke_ratio / r12.modified_burke_ratio;
        let expected = (252.0_f64 / 12.0).sqrt();
        assert!((ratio - expected).abs() < 1e-6);
    }
}
