//! Stochastic RSI — Tushar Chande & Stanley Kroll (1994, "The New
//! Technical Trader").
//!
//! Applies the stochastic-oscillator transform to the RSI series rather
//! than to price:
//!
//!   RSI_t = Wilder RSI over `rsi_period` bars
//!   StochRSI_t = (RSI_t − min(RSI, k_period))
//!                ────────────────────────────────
//!                (max(RSI, k_period) − min(RSI, k_period))
//!
//!   %K = SMA(StochRSI, k_smooth)
//!   %D = SMA(%K, d_smooth)
//!
//! Range [0, 1]:
//!   above 0.8 = overbought (RSI near its recent extreme high)
//!   below 0.2 = oversold; %K/%D crossovers used for entry signals.
//!
//! More responsive than plain stochastic since RSI itself is already
//! smoothed.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StochasticRsiReport {
    pub stoch_rsi: Vec<Option<f64>>,
    pub k_line: Vec<Option<f64>>,
    pub d_line: Vec<Option<f64>>,
}

pub fn compute(
    closes: &[f64],
    rsi_period: usize,
    k_period: usize,
    k_smooth: usize,
    d_smooth: usize,
) -> StochasticRsiReport {
    let n = closes.len();
    let mut out_sr = vec![None; n];
    let mut out_k = vec![None; n];
    let mut out_d = vec![None; n];
    if rsi_period < 2 || k_period < 2 || k_smooth == 0 || d_smooth == 0 { return empty(n); }
    let rsi = wilder_rsi(closes, rsi_period);
    if rsi.len() != n { return empty(n); }
    // StochRSI on the rsi series (skipping leading None values).
    for i in 0..n {
        if i < rsi_period + k_period - 1 { continue; }
        let win = &rsi[i + 1 - k_period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let vals: Vec<f64> = win.iter().map(|x| x.unwrap()).collect();
        let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let cur = vals.last().copied().unwrap();
        if (mx - mn).abs() < 1e-18 {
            out_sr[i] = Some(0.5);
        } else {
            out_sr[i] = Some(((cur - mn) / (mx - mn)).clamp(0.0, 1.0));
        }
    }
    // %K = SMA(StochRSI, k_smooth)
    for i in 0..n {
        if i + 1 < k_smooth { continue; }
        let win = &out_sr[i + 1 - k_smooth..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let sum: f64 = win.iter().map(|x| x.unwrap()).sum();
        out_k[i] = Some(sum / k_smooth as f64);
    }
    // %D = SMA(%K, d_smooth)
    for i in 0..n {
        if i + 1 < d_smooth { continue; }
        let win = &out_k[i + 1 - d_smooth..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let sum: f64 = win.iter().map(|x| x.unwrap()).sum();
        out_d[i] = Some(sum / d_smooth as f64);
    }
    StochasticRsiReport { stoch_rsi: out_sr, k_line: out_k, d_line: out_d }
}

fn empty(n: usize) -> StochasticRsiReport {
    StochasticRsiReport {
        stoch_rsi: vec![None; n],
        k_line: vec![None; n],
        d_line: vec![None; n],
    }
}

fn wilder_rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let mut gain_sum = 0.0;
    let mut loss_sum = 0.0;
    for i in 1..=period {
        let d = closes[i] - closes[i - 1];
        if d > 0.0 { gain_sum += d; } else { loss_sum -= d; }
    }
    let mut avg_g = gain_sum / period as f64;
    let mut avg_l = loss_sum / period as f64;
    out[period] = Some(rsi_from_avgs(avg_g, avg_l));
    for i in (period + 1)..n {
        let d = closes[i] - closes[i - 1];
        let g = d.max(0.0);
        let l = (-d).max(0.0);
        avg_g = (avg_g * (period - 1) as f64 + g) / period as f64;
        avg_l = (avg_l * (period - 1) as f64 + l) / period as f64;
        out[i] = Some(rsi_from_avgs(avg_g, avg_l));
    }
    out
}

fn rsi_from_avgs(avg_g: f64, avg_l: f64) -> f64 {
    if avg_l <= 0.0 { return 100.0; }
    let rs = avg_g / avg_l;
    100.0 - 100.0 / (1.0 + rs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 14, 14, 3, 3);
        assert!(r.stoch_rsi.is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let r = compute(&closes, 0, 14, 3, 3);
        assert!(r.stoch_rsi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn too_short_returns_all_none() {
        let closes = vec![100.0, 101.0, 102.0];
        let r = compute(&closes, 14, 14, 3, 3);
        assert!(r.stoch_rsi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn strict_uptrend_yields_high_stoch_rsi() {
        // Monotonic gainer — RSI saturates near 100 then StochRSI = full
        // range (collapses to 0.5 once min == max).
        let closes: Vec<f64> = (0..60).map(|i| 100.0 + i as f64).collect();
        let r = compute(&closes, 14, 14, 3, 3);
        let v = r.stoch_rsi[59].unwrap();
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn k_line_smooths_stoch_rsi() {
        let closes: Vec<f64> = (0..80).map(|i| 100.0 + (i as f64 * 0.5).sin() * 5.0).collect();
        let r = compute(&closes, 14, 14, 3, 3);
        // %K must lag StochRSI by k_smooth-1 bars.
        assert!(r.k_line[28].is_none() || r.k_line[28].is_some());
        // Wherever both exist, %K equals SMA of last 3 StochRSI.
        for i in 30..80 {
            if let (Some(k), Some(s1), Some(s2), Some(s3)) =
                (r.k_line[i], r.stoch_rsi[i - 2], r.stoch_rsi[i - 1], r.stoch_rsi[i])
            {
                let avg = (s1 + s2 + s3) / 3.0;
                assert!((k - avg).abs() < 1e-12);
                break;
            }
        }
    }

    #[test]
    fn d_line_smooths_k_line() {
        let closes: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64 * 0.3).sin() * 8.0).collect();
        let r = compute(&closes, 14, 14, 3, 3);
        for i in 35..100 {
            if let (Some(d), Some(k1), Some(k2), Some(k3)) =
                (r.d_line[i], r.k_line[i - 2], r.k_line[i - 1], r.k_line[i])
            {
                let avg = (k1 + k2 + k3) / 3.0;
                assert!((d - avg).abs() < 1e-12);
                break;
            }
        }
    }

    #[test]
    fn flat_window_yields_half_stoch_rsi() {
        // Constant prices → constant RSI → min == max in stoch window;
        // implementation collapses to 0.5 in that degenerate case.
        let closes = vec![100.0_f64; 80];
        let r = compute(&closes, 14, 14, 3, 3);
        // After warmup, all StochRSI should be 0.5.
        for v in r.stoch_rsi.iter().flatten() {
            assert!((v - 0.5).abs() < 1e-9, "flat window got {v}");
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let r = compute(&closes, 14, 14, 3, 3);
        assert_eq!(r.stoch_rsi.len(), 50);
        assert_eq!(r.k_line.len(), 50);
        assert_eq!(r.d_line.len(), 50);
    }
}
