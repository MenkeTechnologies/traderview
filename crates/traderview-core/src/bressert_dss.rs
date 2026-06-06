//! Double Smoothed Stochastic — Walter Bressert.
//!
//! Standard stochastic %K passed through two EMA smoothing stages,
//! then re-stochastic'd against its own min/max to give a smoother,
//! lagging oscillator in [0, 100]:
//!
//!   raw_k_t = 100 · (close_t - LL(period)) / (HH(period) - LL(period))
//!   smooth1_t = EMA(raw_k, ema_period)
//!   smooth2_t = EMA(smooth1, ema_period)
//!   final_k_t = 100 · (smooth2_t - LL_s(period)) / (HH_s(period) - LL_s(period))
//!     (where LL_s/HH_s are the period-bar min/max of smooth2)
//!
//! Standard overbought/oversold thresholds: 80 / 20. Centered crossovers
//! of 50 used as trend-change signals.
//!
//! Pure compute. Defaults: stoch_period = 13, ema_period = 8.
//! Companion to `stochastic_rsi`, `stochastic_momentum_index`,
//! `premier_stochastic`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar], stoch_period: usize, ema_period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if stoch_period < 2 || ema_period < 2 || n < stoch_period + 2 * ema_period {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    // Raw %K.
    let mut raw_k = vec![None; n];
    for (i, slot) in raw_k.iter_mut().enumerate().skip(stoch_period - 1) {
        let win = &bars[i + 1 - stoch_period..=i];
        let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let range = hh - ll;
        *slot = Some(if range > 0.0 {
            (bars[i].close - ll) / range * 100.0
        } else {
            50.0
        });
    }
    let s1 = ema_opt(&raw_k, ema_period);
    let s2 = ema_opt(&s1, ema_period);
    // Re-stochastic over period.
    for i in stoch_period - 1..n {
        let win_start = i + 1 - stoch_period;
        let win = &s2[win_start..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        let hh = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ll = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let cur = s2[i].unwrap();
        let range = hh - ll;
        out[i] = Some(if range > 0.0 {
            (cur - ll) / range * 100.0
        } else {
            50.0
        });
    }
    out
}

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 {
        return out;
    }
    let mut seed_end = None;
    let mut seed_sum = 0.0_f64;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => {
                seed_sum += x;
                count += 1;
            }
            None => {
                seed_sum = 0.0;
                count = 0;
            }
        }
        if count == period {
            seed_end = Some(i);
            break;
        }
    }
    let Some(end) = seed_end else {
        return out;
    };
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let mut cur = seed_sum / p_f;
    out[end] = Some(cur);
    for i in (end + 1)..n {
        if let Some(v) = series[i] {
            cur = v * k + cur * (1.0 - k);
            out[i] = Some(cur);
        } else {
            out[i] = Some(cur);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 60];
        assert!(compute(&bars, 1, 8).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 13, 8).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 60];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 13, 8).iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_in_zero_hundred_range() {
        let bars: Vec<_> = (0..200)
            .map(|i| {
                let m = 100.0 + (i as f64 * 0.2).sin() * 5.0;
                b(m + 1.0, m - 1.0, m)
            })
            .collect();
        let r = compute(&bars, 13, 8);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn breakout_from_base_pushes_dss_high() {
        // 60 quiet bars at 100, then 20 rising bars. The post-breakout
        // window still contains low-stochastic readings from the base,
        // so smooth2 varies meaningfully → re-stochastic finds the
        // current bar near the top → DSS > 50.
        let mut bars = vec![b(101.0, 99.0, 100.0); 60];
        for i in 0..20 {
            let m = 100.0 + i as f64;
            bars.push(b(m + 0.5, m - 0.5, m + 0.4));
        }
        let r = compute(&bars, 13, 8);
        let last = bars.len() - 1;
        let v = r[last].unwrap();
        assert!(
            v > 50.0,
            "breakout-from-base should yield DSS > 50, got {v}"
        );
    }

    #[test]
    fn output_within_zero_hundred_in_sin_wave() {
        let bars: Vec<_> = (0..200)
            .map(|i| {
                let m = 100.0 + (i as f64 * 0.1).sin() * 5.0;
                b(m + 1.0, m - 1.0, m)
            })
            .collect();
        let r = compute(&bars, 13, 8);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 60];
        assert_eq!(compute(&bars, 13, 8).len(), 60);
    }
}
