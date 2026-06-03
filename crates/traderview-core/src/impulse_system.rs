//! Elder Impulse System — Alexander Elder ("Come Into My Trading Room").
//!
//! Bar-color regime classifier combining a fast EMA's slope with the
//! MACD histogram's direction:
//!
//!   ema_t = EMA(close, fast_period)
//!   macd_hist_t = EMA(close, macd_fast) - EMA(close, macd_slow)
//!                 - EMA(EMA(close, macd_fast) - EMA(close, macd_slow), macd_signal)
//!
//!   if ema_t > ema_{t-1} AND macd_hist_t > macd_hist_{t-1}: Green
//!     (bullish — only longs / hold)
//!   if ema_t < ema_{t-1} AND macd_hist_t < macd_hist_{t-1}: Red
//!     (bearish — only shorts / hold)
//!   else: Blue
//!     (neutral — no new positions; existing positions can be held)
//!
//! Elder's traffic-light usage: trade ONLY in agreement (green for
//! longs, red for shorts). When color flips against the position,
//! exit.
//!
//! Pure compute. Defaults: fast=13, macd=(12,26,9). Companion to
//! `chande_kroll_stop`, `vortex_indicator`, `coppock_curve`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImpulseColor { #[default] Blue, Green, Red }

pub fn compute(
    closes: &[f64],
    fast_period: usize,
    macd_fast: usize,
    macd_slow: usize,
    macd_signal: usize,
) -> Vec<Option<ImpulseColor>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if fast_period < 2 || macd_fast < 2 || macd_slow < 2 || macd_signal < 2
        || macd_fast >= macd_slow
        || n < macd_slow + macd_signal { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let ema_fast_for_trend = ema(closes, fast_period);
    let ema_macd_fast = ema(closes, macd_fast);
    let ema_macd_slow = ema(closes, macd_slow);
    let mut macd_line = vec![None; n];
    for i in 0..n {
        if let (Some(f), Some(s)) = (ema_macd_fast[i], ema_macd_slow[i]) {
            macd_line[i] = Some(f - s);
        }
    }
    let signal_line = ema_opt(&macd_line, macd_signal);
    let mut hist = vec![None; n];
    for i in 0..n {
        if let (Some(m), Some(s)) = (macd_line[i], signal_line[i]) {
            hist[i] = Some(m - s);
        }
    }
    for i in 1..n {
        if let (Some(et), Some(ep), Some(ht), Some(hp))
            = (ema_fast_for_trend[i], ema_fast_for_trend[i - 1], hist[i], hist[i - 1]) {
            out[i] = Some(classify(et > ep, ht > hp, et < ep, ht < hp));
        }
    }
    out
}

fn classify(ema_up: bool, hist_up: bool, ema_dn: bool, hist_dn: bool) -> ImpulseColor {
    if ema_up && hist_up { ImpulseColor::Green }
    else if ema_dn && hist_dn { ImpulseColor::Red }
    else { ImpulseColor::Blue }
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

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 { return out; }
    let mut seed_end = None;
    let mut seed_sum = 0.0_f64;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => { seed_sum += x; count += 1; }
            None => { seed_sum = 0.0; count = 0; }
        }
        if count == period { seed_end = Some(i); break; }
    }
    let Some(end) = seed_end else { return out; };
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
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 100];
        assert!(compute(&c, 1, 12, 26, 9).iter().all(|x| x.is_none()));
        assert!(compute(&c, 13, 26, 12, 9).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 13, 12, 26, 9).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        assert!(compute(&c, 13, 12, 26, 9).iter().all(|x| x.is_none()));
    }

    #[test]
    fn accelerating_uptrend_yields_green() {
        // Steady linear trend → constant MACD → histogram → 0 → Blue.
        // Strongly accelerating uptrend widens the EMA-fast / EMA-slow
        // lag-gap monotonically → MACD line rising → histogram rising
        // alongside the EMA → Green.
        let c: Vec<f64> = (0..200).map(|i| 100.0 + (i as f64).powi(2) * 0.01).collect();
        let r = compute(&c, 13, 12, 26, 9);
        let any_green = r.iter().skip(80).flatten().any(|x| *x == ImpulseColor::Green);
        assert!(any_green,
            "accelerating uptrend should produce at least one Green bar");
    }

    #[test]
    fn accelerating_downtrend_yields_red() {
        let c: Vec<f64> = (0..200).map(|i| {
            let k = i as f64;
            1000.0 - k * k * 0.01
        }).collect();
        let r = compute(&c, 13, 12, 26, 9);
        let any_red = r.iter().skip(80).flatten().any(|x| *x == ImpulseColor::Red);
        assert!(any_red);
    }

    #[test]
    fn flat_market_yields_blue() {
        let c = vec![100.0_f64; 200];
        let r = compute(&c, 13, 12, 26, 9);
        // No directional movement → Blue.
        for v in r.iter().skip(50).flatten() {
            assert_eq!(*v, ImpulseColor::Blue);
        }
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(true, true, false, false), ImpulseColor::Green);
        assert_eq!(classify(false, false, true, true), ImpulseColor::Red);
        assert_eq!(classify(true, false, false, false), ImpulseColor::Blue);
        assert_eq!(classify(false, true, false, false), ImpulseColor::Blue);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 100];
        assert_eq!(compute(&c, 13, 12, 26, 9).len(), 100);
    }

    // ─── classify exhaustive truth-table + symmetry pins ─────────────
    //
    // Elder's Impulse System: GREEN iff EMA+hist both rising,
    // RED iff both falling, BLUE everywhere else. A bug in the
    // boolean fall-through silently turns a GREEN-flagged bar BLUE
    // and changes every consumer-side entry rule.

    #[test]
    fn classify_red_requires_both_ema_and_hist_down() {
        // Only ema_dn alone → BLUE, not RED.
        assert_eq!(classify(false, false, true, false), ImpulseColor::Blue);
        // Only hist_dn alone → BLUE.
        assert_eq!(classify(false, false, false, true), ImpulseColor::Blue);
        // Both → RED.
        assert_eq!(classify(false, false, true, true), ImpulseColor::Red);
    }

    #[test]
    fn classify_green_dominates_red_when_both_directions_set() {
        // Defensive: when caller passes contradictory signals (up AND
        // down on both axes), GREEN wins because the `ema_up &&
        // hist_up` branch is checked first. Pin this so a refactor
        // doesn't flip priority and silently re-classify edge bars.
        assert_eq!(classify(true, true, true, true), ImpulseColor::Green);
    }

    #[test]
    fn classify_all_false_is_blue() {
        // Neutral / sideways bar: nothing fired → BLUE.
        assert_eq!(classify(false, false, false, false), ImpulseColor::Blue);
    }

    // ─── compute edge: zero / undersized input ───────────────────────

    #[test]
    fn compute_empty_input_returns_empty_output() {
        let out = compute(&[], 13, 12, 26, 9);
        assert!(out.is_empty(), "empty input must yield empty output");
    }

    #[test]
    fn compute_input_shorter_than_period_still_returns_aligned_length() {
        // Output length should always equal input length, even if
        // the period is longer (early bars carry Blue or undefined).
        let c = vec![100.0_f64; 5];
        let out = compute(&c, 13, 12, 26, 9);
        assert_eq!(
            out.len(),
            5,
            "output length must always match input length, even when shorter than period"
        );
    }

    // ─── ema/ema_opt seed + smoothing pins ───────────────────────────
    //
    // The EMA seed is a simple mean over the first `period` bars; the
    // smoothing factor k = 2/(p+1). Drift in either silently shifts
    // every downstream MACD / impulse signal off by ~half a bar.

    #[test]
    fn ema_period_zero_yields_all_none() {
        let s = vec![1.0, 2.0, 3.0, 4.0];
        let out = ema(&s, 0);
        assert_eq!(out.len(), 4);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn ema_seed_is_simple_mean_of_first_period_bars() {
        let s = vec![10.0_f64, 20.0, 30.0, 40.0, 50.0];
        let out = ema(&s, 3);
        // First two bars carry None; bar at index 2 (period-1) carries
        // the simple mean (10+20+30)/3 = 20.
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert_eq!(out[2], Some(20.0));
    }

    #[test]
    fn ema_constant_series_converges_immediately_to_the_constant() {
        // EMA of a constant series equals that constant from the seed
        // bar onward (k * c + (1-k) * c = c).
        let s = vec![42.0_f64; 10];
        let out = ema(&s, 4);
        for (i, slot) in out.iter().enumerate().skip(3) {
            let v = slot.unwrap();
            let diff = (v - 42.0).abs();
            assert!(
                diff < 1e-9,
                "constant-series EMA must equal the constant at idx {i}; got {v}"
            );
        }
    }

    #[test]
    fn ema_output_length_equals_input_length_for_short_series() {
        let s = vec![1.0_f64; 2];
        // Series shorter than period; output is still len=2 with all None.
        let out = ema(&s, 5);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn ema_opt_resets_seed_window_on_None_gap() {
        // When the input contains a None gap, the seed window has to
        // restart from the next Some — period bars of contiguous data
        // are needed before the first emit. Pin that behavior.
        let s = vec![
            Some(1.0_f64),
            Some(2.0),
            None,            // gap resets count
            Some(3.0),
            Some(4.0),
            Some(5.0),
        ];
        let out = ema_opt(&s, 3);
        // Indices 0, 1 cannot seed (only 2 bars before gap).
        // Index 2 is the gap itself.
        // Indices 3, 4, 5 form a contiguous 3-bar window — first emit
        // lands at index 5 (= seed_end).
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert!(out[2].is_none());
        assert!(out[3].is_none());
        assert!(out[4].is_none());
        assert_eq!(out[5], Some(4.0)); // seed = (3+4+5)/3 = 4
    }
}
