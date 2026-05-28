//! Detrended Volatility Oscillator (DVO) — community-developed
//! oscillator on TradingView / Quantopian.
//!
//! Distinct from `detrended_price_oscillator` and `detrended_synthetic_price`:
//! this measures the rank of the close-to-median ratio against its own
//! recent history, useful as a mean-reversion timing tool.
//!
//!   median_t = (high + low) / 2
//!   ratio_t  = close / SMA(median, mean_period)
//!   dvo_t    = percent_rank(ratio_t over rank_period) · 100
//!
//! Range [0, 100]. Above 80 = overbought relative to recent baseline,
//! below 20 = oversold.
//!
//! Pure compute. Defaults: mean_period = 5, rank_period = 252.
//! Companion to `disparity_index`, `bollinger_percent_b`, `z_score_indicator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(
    bars: &[Bar],
    mean_period: usize,
    rank_period: usize,
) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if mean_period < 2 || rank_period < 2
        || n < mean_period + rank_period { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    let medians: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let p_f = mean_period as f64;
    let mut sma_med = vec![None; n];
    let mut sum: f64 = medians[..mean_period].iter().sum();
    sma_med[mean_period - 1] = Some(sum / p_f);
    for i in mean_period..n {
        sum += medians[i] - medians[i - mean_period];
        sma_med[i] = Some(sum / p_f);
    }
    let mut ratio = vec![None; n];
    for i in 0..n {
        if let Some(m) = sma_med[i] {
            if m > 0.0 { ratio[i] = Some(bars[i].close / m); }
        }
    }
    // Percent-rank of ratio[i] within ratio[i-rank_period+1..=i].
    for i in (mean_period + rank_period - 2)..n {
        let win = &ratio[i + 1 - rank_period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        let cur = vals[vals.len() - 1];
        let mut less_or_eq = 0_usize;
        for v in &vals {
            if *v <= cur { less_or_eq += 1; }
        }
        let pct_rank = less_or_eq as f64 / vals.len() as f64 * 100.0;
        out[i] = Some(pct_rank);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 300];
        assert!(compute(&bars, 1, 252).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..50], 5, 252).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 300];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 5, 252).iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_in_zero_hundred_range() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..400).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            let m = 100.0 + (r - 0.5) * 10.0;
            b(m + 1.0, m - 1.0, m)
        }).collect();
        let r = compute(&bars, 5, 252);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn spike_above_baseline_yields_high_pct_rank() {
        // Mostly flat, then a final close spike well above the SMA of medians.
        let mut bars = vec![b(101.0, 99.0, 100.0); 399];
        bars.push(b(110.0, 99.0, 110.0));
        let r = compute(&bars, 5, 252);
        let last = r[399].unwrap();
        assert!(last >= 99.0,
            "spike above baseline should yield DVO near 100, got {last}");
    }

    #[test]
    fn spike_below_baseline_yields_low_pct_rank() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 399];
        bars.push(b(101.0, 85.0, 85.0));
        let r = compute(&bars, 5, 252);
        let last = r[399].unwrap();
        // The latest ratio is the smallest in window → rank near minimum.
        assert!(last <= 1.0,
            "spike below baseline should yield DVO near 0, got {last}");
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 300];
        assert_eq!(compute(&bars, 5, 252).len(), 300);
    }
}
