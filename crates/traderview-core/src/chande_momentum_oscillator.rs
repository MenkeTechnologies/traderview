//! Chande Momentum Oscillator (CMO) — Tushar Chande (1994).
//!
//! Modified RSI that uses unsmoothed sums and unfolds the asymmetry
//! between up- and down-moves over a fixed window:
//!
//!   SoU_t = Σ_{i=t−n+1..t} max(close_i − close_{i−1}, 0)
//!   SoD_t = Σ_{i=t−n+1..t} max(close_{i−1} − close_i, 0)
//!   CMO_t = 100 · (SoU − SoD) / (SoU + SoD)
//!
//! Range [−100, +100]:
//!   above +50 = strong upside momentum (overbought)
//!   below −50 = strong downside momentum (oversold)
//!   Zero-line crossovers used as trend-change signals.
//!
//! Distinct from RSI (which uses Wilder smoothing) and Stoch RSI
//! (which applies a stochastic transform). CMO responds faster than
//! RSI to single large bars.
//!
//! Default period n = 14 (Chande's original).
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(period) {
        let mut sou = 0.0_f64;
        let mut sod = 0.0_f64;
        for k in (i - period + 1)..=i {
            let d = closes[k] - closes[k - 1];
            if d > 0.0 {
                sou += d;
            } else {
                sod -= d;
            }
        }
        let denom = sou + sod;
        *slot = if denom > 0.0 {
            Some(100.0 * (sou - sod) / denom)
        } else {
            Some(0.0)
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn too_short_returns_all_none() {
        let closes = vec![100.0_f64; 5];
        let out = compute(&closes, 14);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 1);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn strict_uptrend_yields_cmo_100() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 14);
        let v = out[49].unwrap();
        assert!(
            (v - 100.0).abs() < 1e-9,
            "uptrend: CMO should be 100, got {v}"
        );
    }

    #[test]
    fn strict_downtrend_yields_cmo_negative_100() {
        let closes: Vec<f64> = (0..50).map(|i| 200.0 - i as f64).collect();
        let out = compute(&closes, 14);
        let v = out[49].unwrap();
        assert!(
            (v + 100.0).abs() < 1e-9,
            "downtrend: CMO should be -100, got {v}"
        );
    }

    #[test]
    fn flat_series_yields_zero_cmo() {
        let closes = vec![100.0_f64; 50];
        let out = compute(&closes, 14);
        // SoU = SoD = 0 → CMO = 0 (per implementation: denom == 0 → 0).
        assert_eq!(out[49].unwrap(), 0.0);
    }

    #[test]
    fn symmetric_alternation_yields_zero_cmo() {
        // Up/down/up/down with equal magnitudes → SoU = SoD → CMO = 0.
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + ((i % 2) as f64)).collect();
        let out = compute(&closes, 14);
        let v = out[49].unwrap();
        assert!(v.abs() < 1e-9, "symmetric: CMO should be 0, got {v}");
    }

    #[test]
    fn cmo_bounded_in_minus_100_to_100() {
        let mut state: u64 = 7;
        let closes: Vec<f64> = (0..200)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                100.0 + ((state >> 32) as f64 / u32::MAX as f64) * 10.0 - 5.0
            })
            .collect();
        let out = compute(&closes, 14);
        for v in out.iter().flatten() {
            assert!((-100.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_length_matches_input() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 0.1).collect();
        let out = compute(&closes, 14);
        assert_eq!(out.len(), 50);
        assert!(out[13].is_none());
        assert!(out[14].is_some());
    }
}
