//! Schaff Trend Cycle — Doug Schaff (1999).
//!
//! Combines MACD's trend identification with Stochastic's cycle-timing:
//!   macd = EMA(close, 23) - EMA(close, 50)
//!   k = stoch_pct_k(macd, cycle_period)
//!   d = stoch_pct_k(k, cycle_period)    // double-smoothed
//!
//! Output: 0..100 oscillator. Standard thresholds 25/75 for entry/exit.
//! Pure compute.

fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    if n == 0 || period == 0 { return vec![]; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut out = Vec::with_capacity(n);
    let mut prev = values[0];
    out.push(prev);
    for i in 1..n {
        let e = k * values[i] + (1.0 - k) * prev;
        out.push(e);
        prev = e;
    }
    out
}

fn stoch_pct_k(values: &[f64], cycle: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![0.0; n];
    if n < cycle || cycle == 0 { return out; }
    for i in (cycle - 1)..n {
        let window = &values[(i + 1 - cycle)..=i];
        let low  = window.iter().cloned().fold(f64::INFINITY, f64::min);
        let high = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = high - low;
        out[i] = if range > 0.0 {
            (values[i] - low) / range * 100.0
        } else { 50.0 };
    }
    out
}

pub fn compute(closes: &[f64], short: usize, long: usize, cycle: usize) -> Vec<f64> {
    let n = closes.len();
    if n == 0 || short == 0 || long == 0 || cycle == 0 { return vec![0.0; n]; }
    let e_short = ema(closes, short);
    let e_long  = ema(closes, long);
    let macd: Vec<f64> = e_short.iter().zip(&e_long).map(|(a, b)| a - b).collect();
    let k = stoch_pct_k(&macd, cycle);
    // Double-smooth.
    stoch_pct_k(&k, cycle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 23, 50, 10).is_empty());
    }

    #[test]
    fn accelerating_uptrend_stc_at_or_above_50() {
        // Linear uptrend makes MACD constant after EMAs settle → stoch
        // collapses to 50 (constant input, range=0 → fallback). For STC
        // to differentiate, MACD must continue moving. Use quadratic
        // acceleration so MACD keeps growing.
        let closes: Vec<f64> = (1..=80).map(|i| {
            100.0 + (i as f64).powi(2) * 0.5
        }).collect();
        let out = compute(&closes, 23, 50, 10);
        assert!(out[79] >= 50.0, "accelerating uptrend should yield STC ≥ 50, got {}", out[79]);
    }

    #[test]
    fn accelerating_downtrend_stc_at_or_below_50() {
        let closes: Vec<f64> = (1..=80).map(|i| {
            2000.0 - (i as f64).powi(2) * 0.5
        }).collect();
        let out = compute(&closes, 23, 50, 10);
        assert!(out[79] <= 50.0);
    }

    #[test]
    fn flat_series_stc_50() {
        let closes = vec![100.0; 80];
        let out = compute(&closes, 23, 50, 10);
        // Constant input → all EMAs equal → MACD = 0 → stoch flat → 50.
        // After full warmup.
        assert!((out[79] - 50.0).abs() < 0.01,
            "flat series should produce STC ≈ 50, got {}", out[79]);
    }

    #[test]
    fn output_bounded_zero_to_hundred() {
        let closes: Vec<f64> = (1..=80).map(|i| {
            100.0 + 10.0 * (i as f64 * 0.5).sin()
        }).collect();
        let out = compute(&closes, 23, 50, 10);
        for v in &out {
            assert!(*v >= 0.0 && *v <= 100.0,
                "STC must be in [0, 100], got {}", v);
        }
    }

    #[test]
    fn zero_periods_return_empty_like_array() {
        let closes = vec![100.0, 105.0, 110.0];
        // Note: zero short/long is invalid — should not panic.
        let out = compute(&closes, 0, 0, 0);
        // Should return zeros or empty.
        assert!(out.iter().all(|v| *v == 0.0));
    }
}
