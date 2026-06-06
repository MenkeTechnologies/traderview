//! Fractal Adaptive Moving Average (FRAMA) — John F. Ehlers (2005).
//!
//! Adaptive EMA whose smoothing constant tracks the local fractal
//! dimension of price. In choppy / high-dimension markets, FRAMA
//! smooths aggressively (low α); in trending / low-dimension markets,
//! FRAMA tracks price closely (high α).
//!
//! For a `period`-bar window (period must be even):
//!
//!   split window in half (N1 = first half, N2 = second half)
//!   H1, L1 = max/min over N1; H2, L2 = max/min over N2
//!   H, L   = max/min over full window
//!
//!   N1_norm = (H1 − L1) / (period/2)
//!   N2_norm = (H2 − L2) / (period/2)
//!   N_norm  = (H − L) / period
//!
//!   D = (log(N1_norm + N2_norm) − log(N_norm)) / log(2)
//!     ∈ [1, 2]; 1 = pure trend, 2 = pure noise
//!
//!   α = exp(−4.6 · (D − 1))
//!     clamped to [0.01, 1.0]
//!
//!   FRAMA_t = α · close_t + (1 − α) · FRAMA_{t−1}
//!
//! Default period = 16.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 4 || !period.is_multiple_of(2_usize) || n < period {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    let half = period / 2;
    let mut prev: Option<f64> = None;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &bars[i + 1 - period..=i];
        let n1 = &win[..half];
        let n2 = &win[half..];
        let h1 = n1.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let l1 = n1.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let h2 = n2.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let l2 = n2.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let h = h1.max(h2);
        let l = l1.min(l2);
        let n1_norm = (h1 - l1) / half as f64;
        let n2_norm = (h2 - l2) / half as f64;
        let n_norm = (h - l) / period as f64;
        let alpha = if n1_norm + n2_norm > 0.0 && n_norm > 0.0 {
            let d = ((n1_norm + n2_norm).ln() - n_norm.ln()) / 2.0_f64.ln();
            let a = (-4.6 * (d - 1.0)).exp();
            a.clamp(0.01, 1.0)
        } else {
            // Degenerate flat window: use full smoothing (α=0.01).
            0.01
        };
        let frama = match prev {
            Some(p) => alpha * bars[i].close + (1.0 - alpha) * p,
            None => bars[i].close,
        };
        *slot = Some(frama);
        prev = Some(frama);
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
    fn empty_returns_empty() {
        assert!(compute(&[], 16).is_empty());
    }

    #[test]
    fn invalid_period_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars, 2).iter().all(|x| x.is_none())); // < 4
        assert!(compute(&bars, 15).iter().all(|x| x.is_none())); // odd
    }

    #[test]
    fn shorter_than_period_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 5];
        assert!(compute(&bars, 16).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_flat_frama() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let out = compute(&bars, 16);
        for x in out.iter().skip(15).flatten() {
            assert!((x - 100.0).abs() < 1e-12, "got {x}");
        }
    }

    #[test]
    fn strong_uptrend_yields_low_dimension_high_alpha() {
        // Strong trend → fractal dimension near 1 → high α → FRAMA close to price.
        let bars: Vec<_> = (0..60)
            .map(|i| {
                let mid = 100.0 + i as f64;
                b(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        let out = compute(&bars, 16);
        let last = out[59].unwrap();
        let last_close = bars[59].close;
        // Should lag only by a small amount.
        assert!(
            (last_close - last).abs() < 5.0,
            "FRAMA {} too far from close {}",
            last,
            last_close
        );
    }

    #[test]
    fn output_length_matches_input() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                b(mid + 1.0, mid - 1.0, mid)
            })
            .collect();
        let out = compute(&bars, 16);
        assert_eq!(out.len(), 50);
        assert!(out[14].is_none());
        assert!(out[15].is_some());
    }
}
