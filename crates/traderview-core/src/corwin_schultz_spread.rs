//! Corwin-Schultz High-Low Effective Spread Estimator (2012).
//!
//! Estimates the effective bid-ask spread from daily high-low ranges
//! WITHOUT requiring quote-level data. Based on the insight that the
//! ratio of two-day to one-day high-low ranges has predictable
//! components of price variance vs spread:
//!
//!   β = (ln(H_t / L_t))² + (ln(H_{t+1} / L_{t+1}))²
//!   γ = (ln(H_{t,t+1} / L_{t,t+1}))²
//!
//!   α = (√(2β) − √β) / (3 − 2√2) − √(γ / (3 − 2√2))
//!
//!   S = 2·(e^α − 1) / (1 + e^α)     (proportional effective spread)
//!
//! where H_{t,t+1}, L_{t,t+1} are the max-high and min-low across the
//! two-day window. Negative S estimates are floored to 0.
//!
//! Aggregate spread over a window of N two-day pairs:
//!   - mean S
//!   - median S
//!   - fraction of negative-α pairs (a noise diagnostic)
//!
//! Pure compute. Companion to `roll_spread`, `effective_spread`,
//! `amihud_illiquidity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorwinSchultzReport {
    /// Per-pair spread estimate (length = bars.len() - 1).
    pub per_pair_spread: Vec<f64>,
    pub mean_spread: f64,
    pub median_spread: f64,
    pub n_pairs: usize,
    pub negative_alpha_fraction: f64,
}

pub fn compute(bars: &[Bar]) -> Option<CorwinSchultzReport> {
    let n = bars.len();
    if n < 2 {
        return None;
    }
    if bars.iter().any(|b| {
        !b.high.is_finite() || !b.low.is_finite() || b.high <= 0.0 || b.low <= 0.0 || b.low > b.high
    }) {
        return None;
    }
    let three_minus_two_sqrt2 = 3.0 - 2.0 * 2.0_f64.sqrt();
    let mut per_pair = Vec::with_capacity(n - 1);
    let mut neg_alpha_count = 0_usize;
    for i in 0..(n - 1) {
        let h1 = bars[i].high;
        let l1 = bars[i].low;
        let h2 = bars[i + 1].high;
        let l2 = bars[i + 1].low;
        let h12 = h1.max(h2);
        let l12 = l1.min(l2);
        if l12 <= 0.0 {
            per_pair.push(0.0);
            continue;
        }
        let log_hl1 = (h1 / l1).ln();
        let log_hl2 = (h2 / l2).ln();
        let beta = log_hl1.powi(2) + log_hl2.powi(2);
        let gamma = (h12 / l12).ln().powi(2);
        let alpha = ((2.0 * beta).sqrt() - beta.sqrt()) / three_minus_two_sqrt2
            - (gamma / three_minus_two_sqrt2).sqrt();
        if alpha < 0.0 {
            neg_alpha_count += 1;
        }
        let s = 2.0 * (alpha.exp() - 1.0) / (1.0 + alpha.exp());
        per_pair.push(s.max(0.0));
    }
    if per_pair.is_empty() {
        return None;
    }
    let mean: f64 = per_pair.iter().sum::<f64>() / per_pair.len() as f64;
    let mut sorted = per_pair.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.len().is_multiple_of(2_usize) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    let neg_frac = neg_alpha_count as f64 / per_pair.len() as f64;
    Some(CorwinSchultzReport {
        n_pairs: per_pair.len(),
        per_pair_spread: per_pair,
        mean_spread: mean,
        median_spread: median,
        negative_alpha_fraction: neg_frac,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[b(101.0, 99.0)]).is_none());
    }

    #[test]
    fn nan_or_invalid_bars_return_none() {
        assert!(compute(&[b(f64::NAN, 99.0), b(101.0, 99.0)]).is_none());
        assert!(compute(&[b(99.0, 101.0), b(101.0, 99.0)]).is_none()); // low > high
        assert!(compute(&[b(-1.0, -2.0), b(101.0, 99.0)]).is_none());
    }

    #[test]
    fn zero_range_bars_yield_zero_spread() {
        let bars = vec![b(100.0, 100.0); 30];
        let r = compute(&bars).unwrap();
        for s in &r.per_pair_spread {
            assert_eq!(*s, 0.0);
        }
    }

    #[test]
    fn per_pair_length_matches_n_minus_1() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.1).sin();
                b(mid + 0.5, mid - 0.5)
            })
            .collect();
        let r = compute(&bars).unwrap();
        assert_eq!(r.per_pair_spread.len(), 49);
        assert_eq!(r.n_pairs, 49);
    }

    #[test]
    fn mean_spread_positive_for_realistic_bars() {
        // Simulate slightly noisy bars; should produce a non-negative mean spread.
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..200)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let noise = ((state >> 32) as f64 / u32::MAX as f64) * 2.0;
                let mid = 100.0 + noise;
                b(mid + 0.10, mid - 0.10)
            })
            .collect();
        let r = compute(&bars).unwrap();
        assert!(r.mean_spread >= 0.0);
    }

    #[test]
    fn negative_alpha_fraction_in_unit_range() {
        let mut state: u64 = 7;
        let bars: Vec<_> = (0..100)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let noise = ((state >> 32) as f64 / u32::MAX as f64) * 2.0;
                let mid = 100.0 + noise;
                b(mid + 0.05, mid - 0.05)
            })
            .collect();
        let r = compute(&bars).unwrap();
        assert!((0.0..=1.0).contains(&r.negative_alpha_fraction));
    }

    #[test]
    fn median_is_in_per_pair_range() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.1).sin();
                b(mid + 0.5, mid - 0.5)
            })
            .collect();
        let r = compute(&bars).unwrap();
        let min = r
            .per_pair_spread
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max = r
            .per_pair_spread
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(r.median_spread >= min && r.median_spread <= max);
    }
}
