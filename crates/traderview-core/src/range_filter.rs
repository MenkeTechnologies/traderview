//! Range Filter — Donovan Wall (TradingView "Range Filter Buy and Sell").
//!
//! Adaptive band that follows price only when the close moves more
//! than a smoothed-range threshold from the prior filter value:
//!
//!   ema_range = EMA(|close_t - close_{t-1}|, n_range)
//!   smoothed_range = EMA(ema_range, 2·n_range - 1) · multiplier
//!   filter_t = if close_t > filter_{t-1} + smoothed_range_t:
//!                  close_t - smoothed_range_t
//!              elif close_t < filter_{t-1} - smoothed_range_t:
//!                  close_t + smoothed_range_t
//!              else:
//!                  filter_{t-1}
//!
//! Trend tag: +1 when filter increases bar-over-bar, -1 when it
//! decreases, 0 unchanged.
//!
//! Pure compute. Defaults: n_range = 20, multiplier = 3.5.
//! Companion to `donchian_channels`, `keltner_squeeze`, `supertrend`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RangeFilterReport {
    pub filter: Vec<Option<f64>>,
    pub upper_band: Vec<Option<f64>>,
    pub lower_band: Vec<Option<f64>>,
    pub trend: Vec<Option<i32>>,
    pub n_range: usize,
    pub multiplier: f64,
}

pub fn compute(
    closes: &[f64],
    n_range: usize,
    multiplier: f64,
) -> RangeFilterReport {
    let n = closes.len();
    let mut report = RangeFilterReport {
        filter: vec![None; n],
        upper_band: vec![None; n],
        lower_band: vec![None; n],
        trend: vec![None; n],
        n_range,
        multiplier,
    };
    if n_range < 2 || !multiplier.is_finite() || multiplier <= 0.0
        || n < 2 * n_range { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    let abs_diff: Vec<f64> = (0..n).map(|i| {
        if i == 0 { 0.0 } else { (closes[i] - closes[i - 1]).abs() }
    }).collect();
    let ema_range = ema(&abs_diff, n_range);
    let mut ema_range_arr = vec![0.0_f64; n];
    let mut have_inner = vec![false; n];
    for i in 0..n {
        if let Some(v) = ema_range[i] {
            ema_range_arr[i] = v;
            have_inner[i] = true;
        }
    }
    let second_period = 2 * n_range - 1;
    let smoothed_range = ema_after_index(&ema_range_arr, &have_inner, second_period);
    // Filter recursion: seed with first close at index where smoothed_range is available.
    let mut last_filter = None;
    for i in 0..n {
        if let Some(sr) = smoothed_range[i] {
            let sr_eff = sr * multiplier;
            let new_filter = match last_filter {
                None => closes[i],
                Some(prev) => {
                    if closes[i] > prev + sr_eff {
                        closes[i] - sr_eff
                    } else if closes[i] < prev - sr_eff {
                        closes[i] + sr_eff
                    } else {
                        prev
                    }
                }
            };
            let trend = match last_filter {
                Some(prev) => {
                    if new_filter > prev { 1 }
                    else if new_filter < prev { -1 }
                    else { 0 }
                }
                None => 0,
            };
            report.filter[i] = Some(new_filter);
            report.upper_band[i] = Some(new_filter + sr_eff);
            report.lower_band[i] = Some(new_filter - sr_eff);
            report.trend[i] = Some(trend);
            last_filter = Some(new_filter);
        }
    }
    report
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

fn ema_after_index(values: &[f64], have: &[bool], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    let start = have.iter().position(|x| *x);
    let Some(seed_start) = start else { return out; };
    if seed_start + period > n { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = values[seed_start..seed_start + period].iter().sum::<f64>() / p_f;
    let seed_idx = seed_start + period - 1;
    out[seed_idx] = Some(seed);
    let mut cur = seed;
    for i in (seed_idx + 1)..n {
        cur = values[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 1, 3.5);
        assert!(r.filter.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 20, 0.0);
        assert!(r2.filter.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        let r = compute(&c, 20, 3.5);
        assert!(r.filter.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_filter_constant() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 20, 3.5);
        // smoothed_range = 0 → any close move (none in flat) triggers no update.
        // Filter seeded at first valid bar = 100; stays 100 thereafter.
        for v in r.filter.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn uptrend_filter_rises_with_trend_plus_one() {
        let c: Vec<f64> = (0..150).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 20, 3.5);
        // Steady linear uptrend should eventually drive filter up with trend +1.
        let any_up = r.trend.iter().flatten().any(|t| *t == 1);
        assert!(any_up, "should observe at least one trend = +1 bar");
    }

    #[test]
    fn downtrend_filter_falls_with_trend_minus_one() {
        let c: Vec<f64> = (0..150).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 20, 3.5);
        let any_dn = r.trend.iter().flatten().any(|t| *t == -1);
        assert!(any_dn);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 20, 3.5);
        assert_eq!(r.filter.len(), 100);
        assert_eq!(r.upper_band.len(), 100);
        assert_eq!(r.trend.len(), 100);
    }
}
