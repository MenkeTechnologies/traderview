//! DeMarker Oscillator — Tom DeMark.
//!
//! Bounded oscillator [0, 1] measuring buying vs selling pressure
//! from per-bar high/low extension:
//!
//!   demax_t = max(high_t - high_{t-1}, 0)
//!   demin_t = max(low_{t-1} - low_t, 0)
//!   sma_max = SMA(demax, N)
//!   sma_min = SMA(demin, N)
//!   DeMarker = sma_max / (sma_max + sma_min)
//!
//! Interpretation: `> 0.7` = overbought, `< 0.3` = oversold.
//! Crossovers used for entry/exit signals.
//!
//! Default period N = 14.
//!
//! Pure compute. Companion to `td_sequential`, `chande_momentum_oscillator`,
//! `stochastic_rsi`.

pub fn compute(highs: &[f64], lows: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = highs.len();
    let mut out = vec![None; n];
    if period < 2 || lows.len() != n || n < period + 1 {
        return out;
    }
    if highs.iter().any(|x| !x.is_finite()) || lows.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let mut demax = vec![0.0_f64; n];
    let mut demin = vec![0.0_f64; n];
    for i in 1..n {
        demax[i] = (highs[i] - highs[i - 1]).max(0.0);
        demin[i] = (lows[i - 1] - lows[i]).max(0.0);
    }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period) {
        let sma_max: f64 = demax[i + 1 - period..=i].iter().sum::<f64>() / p_f;
        let sma_min: f64 = demin[i + 1 - period..=i].iter().sum::<f64>() / p_f;
        let denom = sma_max + sma_min;
        *slot = if denom > 0.0 {
            Some(sma_max / denom)
        } else {
            Some(0.5)
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_all_none() {
        let h = vec![101.0_f64; 30];
        let l = vec![99.0_f64; 30];
        assert!(compute(&h, &l, 1).iter().all(|x| x.is_none()));
        assert!(compute(&h[..5], &l, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let h = vec![f64::NAN; 30];
        let l = vec![99.0_f64; 30];
        assert!(compute(&h, &l, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_in_unit_range() {
        let mut state: u64 = 42;
        let h: Vec<f64> = (0..100)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                101.0 + ((state >> 32) as f64 / u32::MAX as f64) * 2.0
            })
            .collect();
        let l: Vec<f64> = h.iter().map(|x| x - 2.0).collect();
        let r = compute(&h, &l, 14);
        for v in r.iter().flatten() {
            assert!((0.0..=1.0).contains(v));
        }
    }

    #[test]
    fn all_rising_highs_yields_demarker_one() {
        // Each high > prior → demax > 0, demin = 0 → DeMarker = 1.
        let h: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let l: Vec<f64> = (0..30).map(|i| 99.0 + i as f64).collect();
        let r = compute(&h, &l, 14);
        assert!((r[29].unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn all_falling_lows_yields_demarker_zero() {
        let h: Vec<f64> = (0..30).map(|i| 100.0 - i as f64 * 0.1).collect();
        let l: Vec<f64> = (0..30).map(|i| 99.0 - i as f64).collect();
        let r = compute(&h, &l, 14);
        assert!(
            r[29].unwrap() < 0.05,
            "falling lows should yield DeMarker ≈ 0, got {}",
            r[29].unwrap()
        );
    }

    #[test]
    fn flat_market_yields_one_half() {
        let h = vec![101.0_f64; 30];
        let l = vec![99.0_f64; 30];
        let r = compute(&h, &l, 14);
        // Both sums = 0 → demoter = 0.5 (per convention).
        assert!((r[29].unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let h: Vec<f64> = (0..50).map(|i| 101.0 + (i as f64).sin()).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 2.0).collect();
        let r = compute(&h, &l, 14);
        assert_eq!(r.len(), 50);
    }
}
