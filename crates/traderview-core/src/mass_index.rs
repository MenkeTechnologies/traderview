//! Mass Index — Donald Dorsey (1992).
//!
//!   ema1 = 9-period EMA of (high - low)
//!   ema2 = 9-period EMA of ema1
//!   ratio = ema1 / ema2
//!   mass_index = 25-period sum of ratio
//!
//! Dorsey's "reversal bulge": when MI rises above 27 then drops below
//! 26.5, a trend reversal is statistically more likely (regardless of
//! direction). Range-expansion signal — does not predict direction.
//!
//! Pure compute.

pub fn compute(highs: &[f64], lows: &[f64], ema_period: usize, sum_period: usize) -> Vec<f64> {
    let n = highs.len();
    let mut out = vec![0.0; n];
    // `sum_period == 0` would underflow `sum_period - 1` (debug panic) and
    // also yields a meaningless 0-period sum. `ema_period == 0` similarly
    // breaks the ema() helper's smoothing constant. Both are JSON-supplied
    // via the chart route so guard explicitly here.
    if highs.len() != lows.len() || sum_period == 0 || ema_period == 0 || n < sum_period {
        return out;
    }
    let ranges: Vec<f64> = highs.iter().zip(lows).map(|(h, l)| h - l).collect();
    let ema1 = ema(&ranges, ema_period);
    let ema2 = ema(&ema1, ema_period);
    let ratio: Vec<f64> = ema1
        .iter()
        .zip(&ema2)
        .map(|(a, b)| if *b > 0.0 { a / b } else { 0.0 })
        .collect();
    for i in (sum_period - 1)..n {
        out[i] = ratio[(i + 1 - sum_period)..=i].iter().sum();
    }
    out
}

fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    if n == 0 || period == 0 {
        return vec![];
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut out = Vec::with_capacity(n);
    let mut prev = values[0];
    out.push(prev);
    for v in &values[1..n] {
        let e = k * v + (1.0 - k) * prev;
        out.push(e);
        prev = e;
    }
    out
}

/// True when the indicator forms Dorsey's "reversal bulge" at index `i`:
/// rose above `high_threshold` (default 27) recently AND dropped below
/// `low_threshold` (default 26.5) at the current bar.
pub fn detect_reversal_bulge(
    mi: &[f64],
    high_threshold: f64,
    low_threshold: f64,
    lookback: usize,
) -> Vec<bool> {
    let mut out = vec![false; mi.len()];
    for i in 1..mi.len() {
        let start = i.saturating_sub(lookback);
        let max_in_window = mi[start..i]
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        if max_in_window > high_threshold && mi[i] < low_threshold {
            out[i] = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 9, 25).is_empty());
    }

    #[test]
    fn under_sum_period_zeros() {
        let highs = vec![10.0; 10];
        let lows = vec![9.0; 10];
        let out = compute(&highs, &lows, 9, 25);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn length_mismatch_returns_zeros() {
        let highs = vec![10.0; 30];
        let lows = vec![9.0; 10];
        let out = compute(&highs, &lows, 9, 25);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn stable_range_mi_near_sum_period() {
        // Constant range → ema1 = ema2 = range → ratio = 1.0 each bar →
        // 25-sum ratio = 25.
        let highs = vec![10.0; 30];
        let lows = vec![9.0; 30];
        let out = compute(&highs, &lows, 9, 25);
        assert!((out[29] - 25.0).abs() < 0.5);
    }

    #[test]
    fn expanding_range_mi_above_25() {
        // Ranges growing → ema1 outpaces ema2 → ratio > 1 → sum > 25.
        let highs: Vec<f64> = (1..=40).map(|i| 100.0 + i as f64).collect();
        let lows: Vec<f64> = (1..=40).map(|i| 100.0 - i as f64).collect();
        let out = compute(&highs, &lows, 9, 25);
        assert!(out[39] > 25.0);
    }

    // ─── reversal bulge ────────────────────────────────────────────────

    #[test]
    fn no_bulge_detected_when_mi_stays_low() {
        let mi = vec![20.0; 30];
        let out = detect_reversal_bulge(&mi, 27.0, 26.5, 5);
        for b in &out {
            assert!(!*b);
        }
    }

    #[test]
    fn bulge_detected_after_spike_above_27_then_drop_below_26_5() {
        let mut mi = vec![25.0; 25];
        mi.push(28.0); // spike above 27
        mi.push(26.0); // drop below 26.5 → bulge!
        let out = detect_reversal_bulge(&mi, 27.0, 26.5, 5);
        assert!(out[26]);
    }

    #[test]
    fn no_bulge_when_high_threshold_not_breached() {
        let mut mi = vec![25.0; 25];
        mi.push(26.8); // below 27
        mi.push(26.0);
        let out = detect_reversal_bulge(&mi, 27.0, 26.5, 5);
        assert!(!out[26]);
    }
}
