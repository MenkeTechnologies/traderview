//! FRAMA — Fractal Adaptive Moving Average (John Ehlers).
//!
//! Adaptive EMA where the smoothing factor scales with the local
//! fractal dimension of the price series. Trending markets have low
//! fractal dimension (≈ 1) → fast smoothing; choppy markets have high
//! dimension (≈ 2) → slow smoothing.
//!
//! Formula (Ehlers 2005):
//!   period must be EVEN; split window into two halves.
//!   N1 = (highest_first_half − lowest_first_half) / (period / 2)
//!   N2 = (highest_second_half − lowest_second_half) / (period / 2)
//!   N3 = (highest_full − lowest_full) / period
//!   D = (ln(N1 + N2) − ln(N3)) / ln(2)
//!   alpha = exp(−4.6 · (D − 1))      ∈ (0, 1]
//!   FRAMA_t = alpha · close_t + (1 − alpha) · FRAMA_{t−1}
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
    // Period must be even and ≥ 2.
    if period < 2 || !period.is_multiple_of(2) || n < period {
        return out;
    }
    let half = period / 2;
    let half_f = half as f64;
    let period_f = period as f64;
    let mut prev: Option<f64> = None;
    for i in (period - 1)..n {
        let first = &bars[i + 1 - period..=(i + 1 - period + half - 1)];
        let second = &bars[(i + 1 - half)..=i];
        let full = &bars[i + 1 - period..=i];
        let n1 = (high(first) - low(first)) / half_f;
        let n2 = (high(second) - low(second)) / half_f;
        let n3 = (high(full) - low(full)) / period_f;
        // Flat window (range = 0) or non-finite ranges: alpha defaults to
        // 1.0 (FRAMA follows price exactly, which on a flat series means
        // hold the constant). Earlier code `continue`'d before the
        // seed/recurrence below, leaving the entire series None.
        let sum = if n1.is_finite() && n2.is_finite() { n1 + n2 } else { 0.0 };
        let alpha = if !n3.is_finite() || n3 <= 0.0 || sum <= 0.0 {
            1.0
        } else {
            let d = (sum.ln() - n3.ln()) / 2.0_f64.ln();
            let d = d.clamp(1.0, 2.0);
            (-4.6 * (d - 1.0)).exp().clamp(0.01, 1.0)
        };
        let c = bars[i].close;
        if !c.is_finite() {
            if let Some(p) = prev { out[i] = Some(p); }
            continue;
        }
        let new = match prev {
            None => c,
            Some(p) => alpha * c + (1.0 - alpha) * p,
        };
        if new.is_finite() {
            prev = Some(new);
            out[i] = prev;
        }
    }
    out
}

fn high(bars: &[Bar]) -> f64 {
    let mut hi = f64::NEG_INFINITY;
    for b in bars {
        if b.high.is_finite() && b.high > hi { hi = b.high; }
    }
    hi
}

fn low(bars: &[Bar]) -> f64 {
    let mut lo = f64::INFINITY;
    for b in bars {
        if b.low.is_finite() && b.low < lo { lo = b.low; }
    }
    lo
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar { high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 16).is_empty());
    }

    #[test]
    fn odd_or_small_period_returns_all_none() {
        let v = vec![b(101.0, 99.0, 100.0); 50];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
        assert!(compute(&v, 1).iter().all(|x| x.is_none()));
        assert!(compute(&v, 15).iter().all(|x| x.is_none()), "odd period rejected");
    }

    #[test]
    fn flat_series_frama_equals_close() {
        // Flat → N1 = N2 = N3 = 0 → guard hits → carry prior; first
        // populated value seeds at close.
        let v = vec![b(100.0, 100.0, 100.0); 50];
        let out = compute(&v, 16);
        let last = out.last().copied().flatten().expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn rising_series_frama_tracks() {
        let v: Vec<Bar> = (1..=80).map(|i| {
            let c = 100.0 + i as f64;
            b(c + 0.5, c - 0.5, c)
        }).collect();
        let out = compute(&v, 16);
        let last = out[79].expect("populated");
        // On a clean trend D ≈ 1 → alpha ≈ 1 → FRAMA hugs current close.
        assert!((last - v[79].close).abs() < 5.0);
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![b(101.0, 99.0, 100.0); 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
