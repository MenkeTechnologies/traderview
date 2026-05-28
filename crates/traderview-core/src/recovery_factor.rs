//! Recovery Factor — total cumulative return divided by max drawdown.
//!
//!   Recovery = total_return / max_drawdown_pct
//!
//! Higher is better. A recovery factor > 1 means the strategy made
//! back more than the worst drawdown over its life.
//!
//! Companion metrics:
//!   - **MAR Ratio** = annualized_return / max_drawdown_pct
//!   - both rest on the same denominator
//!
//! Pure compute. Companion to `calmar_ratio`, `sterling_ratio`,
//! `burke_ratio`, `pain_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryFactorReport {
    pub recovery_factor: f64,
    pub mar_ratio: f64,
    pub total_return: f64,
    pub annualized_return: f64,
    pub max_drawdown_pct: f64,
}

pub fn compute(
    equity: &[f64],
    periods_per_year: f64,
) -> Option<RecoveryFactorReport> {
    let n = equity.len();
    if n < 2 || !periods_per_year.is_finite() || periods_per_year <= 0.0 {
        return None;
    }
    if equity.iter().any(|x| !x.is_finite() || *x <= 0.0) { return None; }
    let start = equity[0];
    let end = *equity.last().unwrap();
    let total_return = end / start - 1.0;
    let years = (n - 1) as f64 / periods_per_year;
    if years <= 0.0 { return None; }
    let ann_return = (1.0 + total_return).powf(1.0 / years) - 1.0;
    let mut hwm = start;
    let mut max_dd = 0.0_f64;
    for &v in &equity[1..] {
        if v > hwm { hwm = v; }
        let dd = (hwm - v) / hwm;
        if dd > max_dd { max_dd = dd; }
    }
    let recovery = if max_dd > 0.0 { total_return / max_dd } else { f64::INFINITY };
    let mar = if max_dd > 0.0 { ann_return / max_dd } else { f64::INFINITY };
    Some(RecoveryFactorReport {
        recovery_factor: recovery,
        mar_ratio: mar,
        total_return,
        annualized_return: ann_return,
        max_drawdown_pct: max_dd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[100.0], 252.0).is_none());
        assert!(compute(&[100.0, 110.0], 0.0).is_none());
        assert!(compute(&[100.0, f64::NAN], 252.0).is_none());
        assert!(compute(&[100.0, 0.0], 252.0).is_none());
    }

    #[test]
    fn monotone_uptrend_infinite_recovery() {
        let eq: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let r = compute(&eq, 252.0).unwrap();
        assert!(r.recovery_factor.is_infinite());
        assert!(r.mar_ratio.is_infinite());
        assert_eq!(r.max_drawdown_pct, 0.0);
    }

    #[test]
    fn known_recovery_factor() {
        // Total return = 20% (100 → 120). Max DD = 10% (peak 110 → trough 99).
        // Recovery = 0.20 / 0.10 ≈ 2.0... wait but DD computed as (peak-trough)/peak.
        // peak = 110, trough = 99, DD = 11/110 = 0.1.
        let eq = vec![100.0, 110.0, 99.0, 120.0];
        let r = compute(&eq, 252.0).unwrap();
        assert!((r.total_return - 0.20).abs() < 1e-9);
        assert!((r.max_drawdown_pct - 0.10).abs() < 1e-9);
        assert!((r.recovery_factor - 2.0).abs() < 1e-9);
    }

    #[test]
    fn negative_return_negative_recovery() {
        let eq = vec![100.0, 90.0, 80.0];
        let r = compute(&eq, 252.0).unwrap();
        assert!(r.total_return < 0.0);
        assert!(r.recovery_factor < 0.0);
    }

    #[test]
    fn mar_and_recovery_share_denominator() {
        let eq = vec![100.0, 105.0, 95.0, 110.0];
        let r = compute(&eq, 252.0).unwrap();
        // mar_ratio / recovery_factor should equal ann_return / total_return.
        if r.recovery_factor.is_finite() && r.recovery_factor.abs() > 0.0 {
            let ratio = r.mar_ratio / r.recovery_factor;
            let expected = r.annualized_return / r.total_return;
            assert!((ratio - expected).abs() < 1e-9);
        }
    }
}
