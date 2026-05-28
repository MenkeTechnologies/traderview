//! Recursive Moving Average (RMA) — Roy Larsen.
//!
//! Larsen's recursive form is essentially the Wilder smoothing (also
//! known as RMA in Pine Script / Wilder's smoothed moving average):
//!
//!   alpha = 1 / period
//!   RMA_t = alpha · x_t + (1 - alpha) · RMA_{t-1}
//!
//! Equivalent to an EMA with smoothing factor k = 1/N (vs standard
//! EMA's k = 2/(N+1)). Used as the smoothing engine in many Wilder
//! indicators: RSI, ATR, ADX, DI±.
//!
//! Seed: SMA of first `period` values.
//!
//! Pure compute. Companion to `jurik_ma`, `triangular_ma`, `kama`,
//! `vidya`, `tema`, `dema`.

pub fn compute(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period < 2 || n < period { return out; }
    if series.iter().any(|x| !x.is_finite()) { return out; }
    let p_f = period as f64;
    let alpha = 1.0 / p_f;
    let one_minus = 1.0 - alpha;
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = alpha * series[i] + one_minus * cur;
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 50];
        assert!(compute(&s, 1).iter().all(|x| x.is_none()));
        assert!(compute(&s[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_constant_rma() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s, 14);
        for v in r.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn linear_trend_settles_with_known_lag() {
        // For a linear trend with slope k, Wilder RMA lags input by
        // (period - 1) units in steady state (vs (period-1)/2 for SMA).
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 14);
        let last = r[199].unwrap();
        // Steady-state offset ≈ (period - 1) below input.
        let expected_lag = 13.0;
        assert!((s[199] - last - expected_lag).abs() < 0.5,
            "RMA lag for slope-1 linear trend should be ~13, got {}", s[199] - last);
    }

    #[test]
    fn rma_smoother_than_input() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..400).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            100.0 + (r - 0.5) * 10.0
        }).collect();
        let r = compute(&s, 14);
        let vals: Vec<f64> = r.iter().flatten().copied().collect();
        let mean_in: f64 = s.iter().sum::<f64>() / s.len() as f64;
        let var_in: f64 = s.iter().map(|x| (x - mean_in).powi(2)).sum::<f64>() / s.len() as f64;
        let mean_rma: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        let var_rma: f64 = vals.iter().map(|x| (x - mean_rma).powi(2)).sum::<f64>() / vals.len() as f64;
        assert!(var_rma < var_in);
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![100.0_f64; 50];
        assert_eq!(compute(&s, 14).len(), 50);
    }
}
