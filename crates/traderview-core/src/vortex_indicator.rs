//! Vortex Indicator — Etienne Botes & Douglas Siepman (2010).
//!
//! Two oscillators capturing the strength of positive (+VM) and negative
//! (−VM) directional vortex motion across a window. Crossovers between
//! VI⁺ and VI⁻ signal trend changes.
//!
//!   +VM_t = |high_t − low_{t−1}|
//!   −VM_t = |low_t  − high_{t−1}|
//!   TR_t  = max(high_t − low_t, |high_t − close_{t−1}|, |low_t − close_{t−1}|)
//!
//!   VI⁺_t = Σ_{n} +VM / Σ_{n} TR
//!   VI⁻_t = Σ_{n} −VM / Σ_{n} TR
//!
//! Interpretation:
//!   VI⁺ > VI⁻  = uptrend
//!   VI⁻ > VI⁺  = downtrend
//!   Crossovers often precede confirmed trend changes.
//!
//! Default period n = 14 (Botes-Siepman recommendation).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VortexReport {
    pub vi_plus: Vec<Option<f64>>,
    pub vi_minus: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> VortexReport {
    let n = bars.len();
    let mut vi_plus = vec![None; n];
    let mut vi_minus = vec![None; n];
    if period < 2 || n < period + 1 {
        return VortexReport { vi_plus, vi_minus };
    }
    // Per-bar +VM, −VM, TR; require previous bar so first index unfilled.
    let mut pvm = vec![0.0_f64; n];
    let mut nvm = vec![0.0_f64; n];
    let mut tr = vec![0.0_f64; n];
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        if !cur.high.is_finite()
            || !cur.low.is_finite()
            || !cur.close.is_finite()
            || !prev.high.is_finite()
            || !prev.low.is_finite()
            || !prev.close.is_finite()
        {
            continue;
        }
        pvm[i] = (cur.high - prev.low).abs();
        nvm[i] = (cur.low - prev.high).abs();
        let hl = cur.high - cur.low;
        let hc = (cur.high - prev.close).abs();
        let lc = (cur.low - prev.close).abs();
        tr[i] = hl.max(hc).max(lc);
    }
    // Rolling window of length `period`, anchored at i.
    for i in period..n {
        let sum_pvm: f64 = pvm[i + 1 - period..=i].iter().sum();
        let sum_nvm: f64 = nvm[i + 1 - period..=i].iter().sum();
        let sum_tr: f64 = tr[i + 1 - period..=i].iter().sum();
        if sum_tr <= 0.0 {
            continue;
        }
        vi_plus[i] = Some(sum_pvm / sum_tr);
        vi_minus[i] = Some(sum_nvm / sum_tr);
    }
    VortexReport { vi_plus, vi_minus }
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
        let r = compute(&[], 14);
        assert!(r.vi_plus.is_empty() && r.vi_minus.is_empty());
    }

    #[test]
    fn too_short_returns_all_none() {
        let bars: Vec<_> = (0..5).map(|_| b(101.0, 99.0, 100.0)).collect();
        let r = compute(&bars, 14);
        assert!(r.vi_plus.iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let bars: Vec<_> = (0..30).map(|_| b(101.0, 99.0, 100.0)).collect();
        let r = compute(&bars, 1);
        assert!(r.vi_plus.iter().all(|x| x.is_none()));
    }

    #[test]
    fn uptrend_yields_vi_plus_above_vi_minus() {
        // Each bar higher than the last → +VM > −VM consistently.
        let bars: Vec<_> = (0..40)
            .map(|i| {
                let mid = 100.0 + i as f64;
                b(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        let r = compute(&bars, 14);
        let last_plus = r.vi_plus[39].unwrap();
        let last_minus = r.vi_minus[39].unwrap();
        assert!(
            last_plus > last_minus,
            "uptrend: +VI {last_plus} should exceed -VI {last_minus}"
        );
    }

    #[test]
    fn downtrend_yields_vi_minus_above_vi_plus() {
        let bars: Vec<_> = (0..40)
            .map(|i| {
                let mid = 100.0 - i as f64;
                b(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        let r = compute(&bars, 14);
        let last_plus = r.vi_plus[39].unwrap();
        let last_minus = r.vi_minus[39].unwrap();
        assert!(
            last_minus > last_plus,
            "downtrend: -VI {last_minus} should exceed +VI {last_plus}"
        );
    }

    #[test]
    fn flat_market_vi_indicators_balanced() {
        let bars: Vec<_> = (0..40).map(|_| b(101.0, 99.0, 100.0)).collect();
        let r = compute(&bars, 14);
        let p = r.vi_plus[39].unwrap();
        let m = r.vi_minus[39].unwrap();
        // High-Low overlap on flat data → |high - prev_low| = 2, |low - prev_high| = 2.
        assert!(
            (p - m).abs() < 1e-9,
            "flat market: VI+/- should match, got {p} vs {m}"
        );
    }

    #[test]
    fn output_lengths_match_input() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                b(
                    101.0 + i as f64 * 0.1,
                    99.0 + i as f64 * 0.1,
                    100.0 + i as f64 * 0.1,
                )
            })
            .collect();
        let r = compute(&bars, 14);
        assert_eq!(r.vi_plus.len(), 30);
        assert_eq!(r.vi_minus.len(), 30);
    }
}
