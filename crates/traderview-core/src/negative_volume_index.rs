//! Negative Volume Index (NVI) — Paul Dysart (1936), popularized by Norman Fosback.
//!
//! Cumulative index that ONLY updates on days where volume decreases
//! versus the prior day. Fosback's research found that low-volume
//! days reflect "smart-money" activity (institutions accumulating
//! quietly), while high-volume days reflect "crowd" activity.
//!
//!   NVI_0 = 1000
//!   If volume_t < volume_{t-1}:
//!     NVI_t = NVI_{t-1} · (1 + (close_t - close_{t-1}) / close_{t-1})
//!   Else:
//!     NVI_t = NVI_{t-1}
//!
//! Interpretation: NVI > its 1-year (255-bar) EMA is bullish per
//! Fosback. Most useful with a long EMA overlay.
//!
//! Pure compute. Companion to `positive_volume_index`, `on_balance_volume`,
//! `price_volume_trend`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub close: f64, pub volume: f64 }

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 { return out; }
    if bars.iter().any(|b| !b.close.is_finite() || !b.volume.is_finite()) { return out; }
    let mut nvi = 1000.0_f64;
    out[0] = Some(nvi);
    for i in 1..n {
        if bars[i].volume < bars[i - 1].volume {
            let prev = bars[i - 1].close;
            if prev != 0.0 {
                nvi *= 1.0 + (bars[i].close - prev) / prev;
            }
        }
        out[i] = Some(nvi);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar { Bar { close: c, volume: v } }

    #[test]
    fn empty_returns_empty() { assert!(compute(&[]).is_empty()); }

    #[test]
    fn nan_returns_all_none() {
        let bars = vec![b(100.0, 1000.0), b(101.0, f64::NAN)];
        assert!(compute(&bars).iter().all(|x| x.is_none()));
    }

    #[test]
    fn seed_is_one_thousand() {
        let bars = vec![b(100.0, 1000.0)];
        assert!((compute(&bars)[0].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn low_volume_up_day_lifts_nvi() {
        // vol drops 1000 → 500 with price 100 → 110 (+10%).
        let bars = vec![b(100.0, 1000.0), b(110.0, 500.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 1100.0).abs() < 1e-9);
    }

    #[test]
    fn high_volume_day_does_not_update_nvi() {
        // vol rises 1000 → 2000 → NVI unchanged.
        let bars = vec![b(100.0, 1000.0), b(110.0, 2000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn equal_volume_day_does_not_update_nvi() {
        let bars = vec![b(100.0, 1000.0), b(110.0, 1000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 1000.0); 30];
        assert_eq!(compute(&bars).len(), 30);
    }

    #[test]
    fn nvi_remains_positive_across_long_drawdowns() {
        // Down trend on declining volume → NVI multiplied by (1+r) < 1
        // each day but cannot go negative.
        let bars: Vec<_> = (0..50).map(|i| {
            b(100.0 * 0.99_f64.powi(i), 1000.0 - i as f64 * 5.0)
        }).collect();
        let r = compute(&bars);
        for v in r.iter().flatten() { assert!(*v > 0.0); }
    }
}
