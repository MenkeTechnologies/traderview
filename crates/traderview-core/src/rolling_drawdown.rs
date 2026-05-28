//! Rolling Max Drawdown.
//!
//! For each bar i, compute the maximum drawdown observed in the
//! trailing window of length `window`:
//!
//!   running_max_t = max(equity_{t-w+1..t})
//!   dd_t = (running_max_t − equity_t) / running_max_t
//!   max_dd_t = max(dd over window)
//!
//! Outputs both the per-bar drawdown (relative to the window's high)
//! and the rolling max drawdown over the window.
//!
//! Use cases:
//!   - Track strategy drawdown for risk limits
//:   - Visualize drawdown band on equity curves
//!   - Trigger position reduction when rolling DD exceeds threshold
//!
//! Pure compute. Companion to `expected_drawdown`, `ulcer_index`,
//! `pain_index`, `conditional_drawdown`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollingDrawdownReport {
    pub rolling_max_drawdown: Vec<Option<f64>>,
    pub rolling_high_water_mark: Vec<Option<f64>>,
    pub per_bar_drawdown: Vec<Option<f64>>,
}

pub fn compute(equity: &[f64], window: usize) -> Option<RollingDrawdownReport> {
    let n = equity.len();
    if n < 2 || window < 2 { return None; }
    if equity.iter().any(|x| !x.is_finite() || *x <= 0.0) { return None; }
    let mut rolling_max_dd = vec![None; n];
    let mut rolling_hwm = vec![None; n];
    let mut per_bar_dd = vec![None; n];
    for i in 0..n {
        let lo = i.saturating_sub(window - 1);
        let win = &equity[lo..=i];
        let mut hwm = win[0];
        let mut max_dd = 0.0_f64;
        for &v in win {
            if v > hwm { hwm = v; }
            let dd = (hwm - v) / hwm;
            if dd > max_dd { max_dd = dd; }
        }
        rolling_hwm[i] = Some(hwm);
        rolling_max_dd[i] = Some(max_dd);
        per_bar_dd[i] = Some((hwm - equity[i]) / hwm);
    }
    Some(RollingDrawdownReport {
        rolling_max_drawdown: rolling_max_dd,
        rolling_high_water_mark: rolling_hwm,
        per_bar_drawdown: per_bar_dd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_or_invalid_window_returns_none() {
        assert!(compute(&[100.0], 20).is_none());
        assert!(compute(&[100.0, 101.0], 1).is_none());
    }

    #[test]
    fn nan_or_nonpositive_returns_none() {
        assert!(compute(&[100.0, f64::NAN, 102.0], 3).is_none());
        assert!(compute(&[100.0, 0.0, 102.0], 3).is_none());
        assert!(compute(&[100.0, -1.0, 102.0], 3).is_none());
    }

    #[test]
    fn monotone_uptrend_zero_drawdown() {
        let eq: Vec<f64> = (1..=20).map(|i| 100.0 + i as f64).collect();
        let r = compute(&eq, 10).unwrap();
        for dd in r.rolling_max_drawdown.iter().flatten() {
            assert!(dd.abs() < 1e-12);
        }
    }

    #[test]
    fn known_drawdown_detected() {
        // Peak 110, trough 88, drawdown = 0.20.
        let eq = vec![100.0, 105.0, 110.0, 95.0, 88.0, 92.0, 95.0];
        let r = compute(&eq, 7).unwrap();
        let last_dd = r.rolling_max_drawdown[6].unwrap();
        assert!((last_dd - 0.20).abs() < 1e-9,
            "expected DD = 20%, got {}", last_dd);
    }

    #[test]
    fn per_bar_dd_zero_at_high_water_mark() {
        let eq = vec![100.0, 110.0, 105.0, 120.0];
        let r = compute(&eq, 4).unwrap();
        // At i=3, current = 120 = HWM → per-bar DD = 0.
        assert!(r.per_bar_drawdown[3].unwrap().abs() < 1e-12);
    }

    #[test]
    fn rolling_window_forgets_old_drawdowns() {
        // Big drawdown in bars 1-3, recovery, then quiet — small window
        // should not see the old drawdown.
        let mut eq = vec![100.0, 50.0, 100.0];
        eq.extend(vec![100.0, 101.0, 102.0, 103.0, 104.0]);
        let r = compute(&eq, 3).unwrap();
        // Bar 4: window = [100, 101, 102] → DD = 0.
        let dd_4 = r.rolling_max_drawdown[4].unwrap();
        assert!(dd_4.abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let eq: Vec<f64> = (1..=30).map(|i| 100.0 + (i as f64).sin()).collect();
        let r = compute(&eq, 10).unwrap();
        assert_eq!(r.rolling_max_drawdown.len(), 30);
        assert_eq!(r.rolling_high_water_mark.len(), 30);
        assert_eq!(r.per_bar_drawdown.len(), 30);
    }
}
