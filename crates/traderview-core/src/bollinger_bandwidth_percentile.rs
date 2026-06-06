//! Bollinger Bandwidth Percentile (BBWP) — community indicator.
//!
//! Percent rank of the current Bollinger Band width within its own
//! rolling history. Whereas `bollinger_band_width` reports the raw
//! width and `bollinger_squeeze` flags binary squeeze/no-squeeze,
//! BBWP gives a continuous reading of where current volatility sits
//! relative to recent history:
//!
//!   sma_t   = SMA(close, bb_period)
//!   stdev_t = sample stdev of close over bb_period
//!   width_t = (2 · n_stdev · stdev) / sma · 100   (% of midline)
//!   bbwp_t  = percent_rank(width_t, lookback)
//!
//! Range [0, 100]:
//!   BBWP > 90 → volatility in top 10% of recent history (expansion)
//!   BBWP < 10 → volatility in bottom 10% (compression/squeeze)
//!
//! Pure compute. Defaults: bb_period = 20, n_stdev = 2.0, lookback = 252.
//! Companion to `bollinger_band_width`, `bollinger_squeeze`,
//! `bollinger_percent_b`.

pub fn compute(
    closes: &[f64],
    bb_period: usize,
    n_stdev: f64,
    lookback: usize,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if bb_period < 2
        || lookback < bb_period
        || !n_stdev.is_finite()
        || n_stdev <= 0.0
        || n < lookback
    {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let p_f = bb_period as f64;
    let mut width = vec![None; n];
    for i in (bb_period - 1)..n {
        let win = &closes[i + 1 - bb_period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        if mean.abs() > 0.0 {
            width[i] = Some(2.0 * n_stdev * std / mean.abs() * 100.0);
        }
    }
    for (i, slot) in out.iter_mut().enumerate().skip(lookback - 1) {
        let win = &width[i + 1 - lookback..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        let cur = vals[vals.len() - 1];
        let less_or_eq = vals.iter().filter(|v| **v <= cur).count();
        *slot = Some(less_or_eq as f64 / vals.len() as f64 * 100.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 300];
        assert!(compute(&c, 1, 2.0, 252).iter().all(|x| x.is_none()));
        assert!(compute(&c, 20, 0.0, 252).iter().all(|x| x.is_none()));
        assert!(compute(&c, 20, 2.0, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 300];
        c[5] = f64::NAN;
        assert!(compute(&c, 20, 2.0, 252).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_high_rank() {
        // Flat market → constant zero width → every reading equals min/max
        // → rank = 100 (all values ≤ current).
        let c = vec![100.0_f64; 300];
        let r = compute(&c, 20, 2.0, 252);
        for v in r.iter().skip(260).flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn volatility_spike_yields_high_rank() {
        let mut state: u64 = 42;
        let mut c = vec![100.0_f64; 252];
        for _ in 0..50 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            c.push(100.0 + (r - 0.5) * 50.0);
        }
        let r = compute(&c, 20, 2.0, 252);
        // Final bars in surge region should sit high in the percentile.
        let last_few: Vec<f64> = r.iter().rev().take(20).filter_map(|x| *x).collect();
        let max_p = last_few.iter().cloned().fold(0.0_f64, f64::max);
        assert!(
            max_p > 70.0,
            "volatility surge should rank high, got max {max_p}"
        );
    }

    #[test]
    fn output_in_zero_hundred_range() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..400)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
                100.0 + (r - 0.5) * 5.0
            })
            .collect();
        let r = compute(&c, 20, 2.0, 252);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 300];
        assert_eq!(compute(&c, 20, 2.0, 252).len(), 300);
    }
}
