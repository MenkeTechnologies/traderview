//! Arnaud Legoux Moving Average (ALMA) — Arnaud Legoux & Dimitris
//! Kouzis-Loukas (2009).
//!
//! FIR filter with a Gaussian-shaped weighting kernel offset from the
//! center of the window. Lower lag than SMA/EMA, sharper turning-point
//! response than triangular MAs.
//!
//! For a window of length `period`:
//!
//!   m = floor(offset · (period − 1))
//!   s = period / sigma
//!   w_i = exp(−(i − m)² / (2·s²))            for i = 0..period−1
//!   ALMA_t = Σ w_i · close_{t − (period − 1) + i}  /  Σ w_i
//!
//! Default parameters per Legoux's original paper:
//!   period = 9, offset = 0.85, sigma = 6.0
//!
//! - `offset` = 1.0 → kernel peak at most recent bar (highest
//!   responsiveness, max lag reduction; close to EMA in limit)
//! - `offset` = 0.0 → peak at oldest bar (just-in-past smoothing)
//! - `offset` = 0.5 → centered Gaussian (lowest lag-reduction but most
//!   noise rejection)
//! - `sigma` → sharper kernel = lower noise rejection
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize, offset: f64, sigma: f64) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2
        || n < period
        || !offset.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
        || !(0.0..=1.0).contains(&offset)
    {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    // Pre-compute kernel.
    let m = (offset * (period - 1) as f64).floor();
    let s = period as f64 / sigma;
    let denom_inv = 1.0 / (2.0 * s * s);
    let mut w = vec![0.0_f64; period];
    let mut w_sum = 0.0_f64;
    for (i, slot) in w.iter_mut().enumerate() {
        let d = i as f64 - m;
        *slot = (-d * d * denom_inv).exp();
        w_sum += *slot;
    }
    if w_sum <= 0.0 {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let mut acc = 0.0_f64;
        for k in 0..period {
            acc += w[k] * closes[i + 1 - period + k];
        }
        *slot = Some(acc / w_sum);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 9, 0.85, 6.0).is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let closes = vec![100.0_f64; 30];
        assert!(compute(&closes, 1, 0.85, 6.0).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 9, -0.1, 6.0).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 9, 1.1, 6.0).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 9, 0.85, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 9, 0.85, -1.0).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 9, f64::NAN, 6.0)
            .iter()
            .all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_period_returns_all_none() {
        let closes = vec![100.0_f64; 5];
        assert!(compute(&closes, 9, 0.85, 6.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_flat_alma() {
        let closes = vec![100.0_f64; 30];
        let out = compute(&closes, 9, 0.85, 6.0);
        for v in out.iter().skip(8) {
            assert!((v.unwrap() - 100.0).abs() < 1e-12);
        }
    }

    #[test]
    fn uptrend_alma_below_current_price() {
        // ALMA lags price (offset < 1) so on uptrend it sits below close.
        let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 9, 0.85, 6.0);
        for i in 8..30 {
            let v = out[i].unwrap();
            assert!(
                v < closes[i],
                "ALMA {} should lag close {} in uptrend",
                v,
                closes[i]
            );
        }
    }

    #[test]
    fn outputs_align_to_input_length() {
        let closes: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * 0.3).sin() * 5.0)
            .collect();
        let out = compute(&closes, 9, 0.85, 6.0);
        assert_eq!(out.len(), 50);
        assert!(out[7].is_none());
        assert!(out[8].is_some());
    }

    #[test]
    fn higher_offset_responds_faster() {
        // Two ALMAs on the same step-function input. Higher offset places
        // kernel peak nearer the most recent bar → closer to the new level
        // after a step.
        let mut closes = vec![100.0_f64; 20];
        closes.extend(vec![110.0_f64; 20]);
        let low_off = compute(&closes, 9, 0.10, 6.0);
        let high_off = compute(&closes, 9, 0.95, 6.0);
        let i = 25; // ~5 bars after the step
        let low_v = low_off[i].unwrap();
        let high_v = high_off[i].unwrap();
        assert!(
            high_v > low_v,
            "high-offset ALMA ({high_v}) should react faster than low-offset ({low_v})"
        );
    }
}
