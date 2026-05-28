//! Elder Safe-Zone Stop — Alexander Elder.
//!
//! Adaptive trailing-stop technique. For long positions, computes a
//! Wilder-EMA of recent downside penetrations of the prior bar's low,
//! multiplies by a coefficient (Elder used 3.0), and subtracts from
//! the most recent low. The stop ratchets up only.
//!
//! For shorts, the symmetric computation on upside penetrations of
//! the prior high, ratcheting down only.
//!
//! Per-bar safe-zone stop (long side):
//!
//!   pen_down_t = max(low_{t-1} - low_t, 0)          (downside penetration)
//!   avg_pen_t  = Wilder EMA of pen_down over N bars
//!   raw_stop_t = low_t - K · avg_pen_t
//!   stop_t     = max(raw_stop_t, stop_{t-1})        (ratchet)
//!
//! Pure compute. Returns stops for both long-side and short-side runs.
//! Companion to `chande_kroll_stop` (if shipped) and `parabolic_sar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElderSafeZoneReport {
    pub long_stop: Vec<Option<f64>>,
    pub short_stop: Vec<Option<f64>>,
    pub period: usize,
    pub k_multiplier: f64,
}

pub fn compute(
    bars: &[Bar],
    period: usize,
    k_multiplier: f64,
) -> ElderSafeZoneReport {
    let n = bars.len();
    let mut report = ElderSafeZoneReport {
        long_stop: vec![None; n],
        short_stop: vec![None; n],
        period,
        k_multiplier,
    };
    if period < 2 || !k_multiplier.is_finite() || k_multiplier <= 0.0
        || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()) { return report; }
    let mut pen_down = vec![0.0_f64; n];
    let mut pen_up = vec![0.0_f64; n];
    for i in 1..n {
        pen_down[i] = (bars[i - 1].low - bars[i].low).max(0.0);
        pen_up[i] = (bars[i].high - bars[i - 1].high).max(0.0);
    }
    let avg_down = wilder_ema(&pen_down[1..], period);
    let avg_up = wilder_ema(&pen_up[1..], period);
    // Long-side ratchets up: stop = max(low_t - k·avg_down, prior_stop).
    let mut last_long: Option<f64> = None;
    for i in 1..n {
        if let Some(avg) = avg_down[i - 1] {
            let raw = bars[i].low - k_multiplier * avg;
            let chosen = match last_long { Some(p) => raw.max(p), None => raw };
            last_long = Some(chosen);
            report.long_stop[i] = Some(chosen);
        }
    }
    // Short-side ratchets down: stop = min(high_t + k·avg_up, prior_stop).
    let mut last_short: Option<f64> = None;
    for i in 1..n {
        if let Some(avg) = avg_up[i - 1] {
            let raw = bars[i].high + k_multiplier * avg;
            let chosen = match last_short { Some(p) => raw.min(p), None => raw };
            last_short = Some(chosen);
            report.short_stop[i] = Some(chosen);
        }
    }
    report
}

fn wilder_ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = (cur * (p_f - 1.0) + series[i]) / p_f;
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn invalid_params_return_empty() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 1, 3.0);
        assert!(r.long_stop.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 14, 0.0);
        assert!(r2.long_stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0); 30];
        bars[5] = b(f64::NAN, 99.0);
        let r = compute(&bars, 14, 3.0);
        assert!(r.long_stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_constant_stops() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 14, 3.0);
        // Zero penetration → avg = 0 → stop = low (long) / high (short).
        for v in r.long_stop.iter().skip(15).flatten() {
            assert!((v - 99.0).abs() < 1e-9);
        }
        for v in r.short_stop.iter().skip(15).flatten() {
            assert!((v - 101.0).abs() < 1e-9);
        }
    }

    #[test]
    fn long_stop_ratchets_up_in_uptrend() {
        let bars: Vec<_> = (0..30).map(|i| b(101.0 + i as f64, 99.0 + i as f64)).collect();
        let r = compute(&bars, 14, 3.0);
        let vals: Vec<f64> = r.long_stop.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9);
        }
    }

    #[test]
    fn short_stop_ratchets_down_in_downtrend() {
        let bars: Vec<_> = (0..30).map(|i| b(200.0 - i as f64, 198.0 - i as f64)).collect();
        let r = compute(&bars, 14, 3.0);
        let vals: Vec<f64> = r.short_stop.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] <= w[0] + 1e-9);
        }
    }

    #[test]
    fn long_stop_below_low_short_stop_above_high() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..100).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let mid = 100.0 + ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 4.0;
            b(mid + 1.0, mid - 1.0)
        }).collect();
        let r = compute(&bars, 14, 3.0);
        // raw_stop for long = low - k·avg ≤ low. But ratcheted stop can
        // climb ABOVE the current low if a prior low's stop was higher.
        // The invariant we test is: at the SEED bar (first valid stop),
        // raw_stop ≤ low.
        let seed_idx = r.long_stop.iter().position(|x| x.is_some()).unwrap();
        assert!(r.long_stop[seed_idx].unwrap() <= bars[seed_idx].low + 1e-9);
        let seed_idx_s = r.short_stop.iter().position(|x| x.is_some()).unwrap();
        assert!(r.short_stop[seed_idx_s].unwrap() >= bars[seed_idx_s].high - 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 14, 3.0);
        assert_eq!(r.long_stop.len(), 30);
        assert_eq!(r.short_stop.len(), 30);
    }
}
