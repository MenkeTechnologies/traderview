//! Weis Wave Volume — David Weis (Trades About to Happen, 2013).
//!
//! Cumulative volume per "wave" defined by ZigZag-style reversals
//! larger than a configurable threshold. Each wave's volume = sum of
//! per-bar volumes while price is moving in the wave's direction; on
//! reversal, a new wave begins.
//!
//! Per-bar output: the cumulative volume of the wave that bar belongs
//! to, signed positive for up-waves, negative for down-waves.
//!
//! Two-pass algorithm:
//!   1. Identify wave segments via close-based ZigZag with `reversal_pct`
//!   2. For each bar, attribute it to its containing wave and emit the
//!      signed cumulative wave volume up through that bar.
//!
//! Useful for Wyckoff-style volume confirmation: a smaller up-wave volume
//! than the previous up-wave despite a higher price = bearish divergence.
//!
//! Pure compute. Companion to `volume_burst`, `accumulation_distribution_line`,
//! `vsa`, `wyckoff`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub close: f64, pub volume: f64 }

pub fn compute(bars: &[Bar], reversal_pct: f64) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 || !reversal_pct.is_finite() || reversal_pct <= 0.0 { return out; }
    if bars.iter().any(|b| !b.close.is_finite() || !b.volume.is_finite()) { return out; }
    // direction: +1 = up wave, -1 = down wave, 0 = undetermined.
    let mut direction = 0_i32;
    let mut wave_anchor_close = bars[0].close;
    let mut wave_cum_vol = bars[0].volume;
    let threshold = reversal_pct / 100.0;
    out[0] = Some(wave_cum_vol);
    for i in 1..n {
        let close = bars[i].close;
        match direction {
            0 => {
                // Establish initial direction once we see a meaningful move.
                let change = (close - wave_anchor_close) / wave_anchor_close.abs().max(1e-12);
                if change.abs() >= threshold {
                    direction = if change > 0.0 { 1 } else { -1 };
                }
                wave_cum_vol += bars[i].volume;
                out[i] = Some(signed(wave_cum_vol, direction));
            }
            1 => {
                if close > wave_anchor_close {
                    // Still up; extend wave.
                    wave_anchor_close = close;
                    wave_cum_vol += bars[i].volume;
                    out[i] = Some(signed(wave_cum_vol, direction));
                } else {
                    let drop_pct = (wave_anchor_close - close) / wave_anchor_close.abs().max(1e-12);
                    if drop_pct >= threshold {
                        direction = -1;
                        wave_anchor_close = close;
                        wave_cum_vol = bars[i].volume;
                        out[i] = Some(signed(wave_cum_vol, direction));
                    } else {
                        wave_cum_vol += bars[i].volume;
                        out[i] = Some(signed(wave_cum_vol, direction));
                    }
                }
            }
            -1 => {
                if close < wave_anchor_close {
                    wave_anchor_close = close;
                    wave_cum_vol += bars[i].volume;
                    out[i] = Some(signed(wave_cum_vol, direction));
                } else {
                    let rally_pct = (close - wave_anchor_close) / wave_anchor_close.abs().max(1e-12);
                    if rally_pct >= threshold {
                        direction = 1;
                        wave_anchor_close = close;
                        wave_cum_vol = bars[i].volume;
                        out[i] = Some(signed(wave_cum_vol, direction));
                    } else {
                        wave_cum_vol += bars[i].volume;
                        out[i] = Some(signed(wave_cum_vol, direction));
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    out
}

fn signed(vol: f64, direction: i32) -> f64 {
    match direction {
        1 => vol,
        -1 => -vol,
        _ => vol,    // pre-direction: report positive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar { Bar { close: c, volume: v } }

    #[test]
    fn empty_returns_empty() { assert!(compute(&[], 1.0).is_empty()); }

    #[test]
    fn invalid_reversal_returns_all_none() {
        let bars = vec![b(100.0, 1000.0); 10];
        assert!(compute(&bars, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&bars, f64::NAN).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let bars = vec![b(100.0, 1000.0), b(f64::NAN, 1000.0)];
        assert!(compute(&bars, 1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn pure_uptrend_accumulates_positive_volume() {
        let bars: Vec<_> = (0..20).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        let r = compute(&bars, 1.0);
        // After direction is set to +1, cumulative volume keeps growing.
        let last = r[19].unwrap();
        assert!(last > 0.0);
        // Total volume bound: 20·1000 = 20000.
        assert!(last <= 20_000.0);
    }

    #[test]
    fn pure_downtrend_accumulates_negative_volume() {
        let bars: Vec<_> = (0..20).map(|i| b(100.0 - i as f64, 1000.0)).collect();
        let r = compute(&bars, 1.0);
        let last = r[19].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn reversal_resets_wave_volume() {
        // Up 10 bars (close 100 → 110), then sharp drop to 95.
        let mut bars: Vec<Bar> = (0..10).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        bars.push(b(95.0, 5000.0));
        let r = compute(&bars, 1.0);
        // Bar 10 is start of new down wave → cum vol = 5000 (signed negative).
        assert!(r[10].unwrap() < 0.0);
        assert!((r[10].unwrap() + 5000.0).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 1000.0); 30];
        assert_eq!(compute(&bars, 1.0).len(), 30);
    }
}
