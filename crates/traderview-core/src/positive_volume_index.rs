//! Positive Volume Index (PVI) — Paul Dysart (1936), popularized by Norman Fosback.
//!
//! Cumulative index that ONLY updates on days where volume increases
//! versus the prior day. Fosback's research: high-volume days reflect
//! "uninformed crowd" activity, so PVI tracks the crowd; NVI tracks
//! smart money.
//!
//!   PVI_0 = 1000
//!   If volume_t > volume_{t-1}:
//!     PVI_t = PVI_{t-1} · (1 + (close_t - close_{t-1}) / close_{t-1})
//!   Else:
//!     PVI_t = PVI_{t-1}
//!
//! Per Fosback: PVI above its 255-bar EMA → bullish 67% of the time;
//! PVI below → bearish only 53% of the time (PVI is the weaker signal
//! versus its NVI partner).
//!
//! Pure compute. Companion to `negative_volume_index`, `on_balance_volume`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub close: f64, pub volume: f64 }

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 { return out; }
    if bars.iter().any(|b| !b.close.is_finite() || !b.volume.is_finite()) { return out; }
    let mut pvi = 1000.0_f64;
    out[0] = Some(pvi);
    for i in 1..n {
        if bars[i].volume > bars[i - 1].volume {
            let prev = bars[i - 1].close;
            if prev != 0.0 {
                pvi *= 1.0 + (bars[i].close - prev) / prev;
            }
        }
        out[i] = Some(pvi);
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
        let bars = vec![b(100.0, 1000.0), b(f64::NAN, 2000.0)];
        assert!(compute(&bars).iter().all(|x| x.is_none()));
    }

    #[test]
    fn seed_is_one_thousand() {
        let bars = vec![b(100.0, 1000.0)];
        assert!((compute(&bars)[0].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn high_volume_up_day_lifts_pvi() {
        let bars = vec![b(100.0, 1000.0), b(110.0, 2000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 1100.0).abs() < 1e-9);
    }

    #[test]
    fn low_volume_day_does_not_update_pvi() {
        let bars = vec![b(100.0, 2000.0), b(110.0, 1000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn equal_volume_day_does_not_update_pvi() {
        let bars = vec![b(100.0, 1000.0), b(110.0, 1000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn high_volume_down_day_drops_pvi() {
        let bars = vec![b(100.0, 1000.0), b(90.0, 2000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 900.0).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 1000.0); 30];
        assert_eq!(compute(&bars).len(), 30);
    }
}
