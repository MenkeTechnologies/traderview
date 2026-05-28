//! Volume Oscillator (VO) — Pring/Donchian classical formulation.
//!
//! Percent difference between a fast and slow EMA of volume:
//!
//!   fast_t = EMA(volume, fast_period)
//!   slow_t = EMA(volume, slow_period)
//!   VO_t  = (fast_t - slow_t) / slow_t · 100
//!
//! Range: unbounded but typically -50..+150. Used as a
//! breadth-of-volume gauge:
//!
//!   VO > 0 → short-term volume burst above long-term average
//!     (confirms breakouts)
//!   VO < 0 → short-term volume drying up
//!     (trend losing fuel; corrections often near)
//!
//! Distinct from `klinger_volume_oscillator` (KVO), which uses
//! signed volume force rather than raw volume.
//!
//! Pure compute. Defaults: fast = 14, slow = 28. Companion to
//! `on_balance_volume`, `chaikin_money_flow`, `price_volume_trend`.

pub fn compute(volumes: &[f64], fast_period: usize, slow_period: usize) -> Vec<Option<f64>> {
    let n = volumes.len();
    let mut out = vec![None; n];
    if fast_period < 2 || slow_period < 2 || fast_period >= slow_period
        || n < slow_period { return out; }
    if volumes.iter().any(|x| !x.is_finite() || *x < 0.0) { return out; }
    let fast = ema(volumes, fast_period);
    let slow = ema(volumes, slow_period);
    for (i, slot) in out.iter_mut().enumerate() {
        if let (Some(f), Some(s)) = (fast[i], slow[i]) {
            if s > 0.0 {
                *slot = Some((f - s) / s * 100.0);
            }
        }
    }
    out
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let v = vec![1000.0_f64; 50];
        assert!(compute(&v, 1, 28).iter().all(|x| x.is_none()));
        assert!(compute(&v, 28, 14).iter().all(|x| x.is_none()));
        assert!(compute(&v[..5], 14, 28).iter().all(|x| x.is_none()));
    }

    #[test]
    fn negative_or_nan_returns_empty() {
        let v = vec![1000.0_f64; 50];
        let mut v_neg = v.clone();
        v_neg[5] = -100.0;
        assert!(compute(&v_neg, 14, 28).iter().all(|x| x.is_none()));
        let mut v_nan = v;
        v_nan[5] = f64::NAN;
        assert!(compute(&v_nan, 14, 28).iter().all(|x| x.is_none()));
    }

    #[test]
    fn constant_volume_yields_zero_vo() {
        let v = vec![1000.0_f64; 50];
        let r = compute(&v, 14, 28);
        for x in r.iter().flatten() {
            assert!(x.abs() < 1e-9);
        }
    }

    #[test]
    fn surging_volume_yields_positive_vo() {
        let mut v = vec![1000.0_f64; 30];
        v.extend(std::iter::repeat_n(5000.0_f64, 10));
        let r = compute(&v, 14, 28);
        let last = v.len() - 1;
        assert!(r[last].unwrap() > 0.0);
    }

    #[test]
    fn shrinking_volume_yields_negative_vo() {
        let mut v = vec![5000.0_f64; 30];
        v.extend(std::iter::repeat_n(1000.0_f64, 10));
        let r = compute(&v, 14, 28);
        let last = v.len() - 1;
        assert!(r[last].unwrap() < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let v = vec![1000.0_f64; 50];
        assert_eq!(compute(&v, 14, 28).len(), 50);
    }
}
