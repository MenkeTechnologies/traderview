//! McClellan Summation Index — long-term running sum of the McClellan
//! oscillator (which itself is `EMA-19(adv−dec) − EMA-39(adv−dec)`).
//!
//! Cumulative integrator of the existing `mcclellan_oscillator`. Reading:
//!   - SI rising and > 0 = secular uptrend in breadth
//!   - SI falling and < 0 = secular downtrend
//!   - SI cross of its 10-day SMA = intermediate-term signal
//!   - SI > +3000 / < −3000 = breadth thrust / crash levels
//!
//! Caller supplies the per-day McClellan oscillator series (use
//! `mcclellan_oscillator::compute` to build it). Pure compute.

pub fn compute(mcclellan: &[Option<f64>]) -> Vec<Option<f64>> {
    let n = mcclellan.len();
    let mut out = vec![None; n];
    if n == 0 {
        return out;
    }
    let mut acc: Option<f64> = None;
    for (i, v) in mcclellan.iter().enumerate() {
        if let Some(x) = v {
            if x.is_finite() {
                acc = Some(acc.unwrap_or(0.0) + x);
            }
        }
        if let Some(a) = acc {
            if a.is_finite() {
                out[i] = Some(a);
            } else {
                acc = Some(0.0);
                out[i] = Some(0.0);
            }
        }
    }
    out
}

/// Rolling SMA over a populated Option series — convenience companion
/// so callers can compute the canonical "SI vs 10-day SMA" cross signal
/// without re-implementing windowing.
pub fn sma(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        let mut sum = 0.0;
        let mut ok = true;
        for v in win {
            match v {
                Some(x) if x.is_finite() => sum += x,
                _ => { ok = false; break; }
            }
        }
        if ok {
            out[i] = Some(sum / period as f64);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn all_none_input_yields_all_none_output() {
        let v = vec![None; 20];
        let out = compute(&v);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn cumulative_accumulates_after_first_value() {
        let v = vec![None, Some(10.0), Some(20.0), Some(-5.0)];
        let out = compute(&v);
        assert!(out[0].is_none());
        assert_eq!(out[1], Some(10.0));
        assert_eq!(out[2], Some(30.0));
        assert_eq!(out[3], Some(25.0));
    }

    #[test]
    fn flat_zero_input_yields_flat_zero_output() {
        let v: Vec<Option<f64>> = (0..30).map(|_| Some(0.0)).collect();
        let out = compute(&v);
        for x in out.iter().flatten() {
            assert_eq!(*x, 0.0);
        }
    }

    #[test]
    fn sma_populated_after_warmup() {
        let v: Vec<Option<f64>> = (1..=20).map(|i| Some(i as f64)).collect();
        let out = sma(&v, 5);
        assert!(out[3].is_none());
        let r = out[4].expect("populated");
        // SMA of 1..5 = 3.
        assert!((r - 3.0).abs() < 1e-9);
    }

    #[test]
    fn sma_period_zero_returns_all_none() {
        let v: Vec<Option<f64>> = (0..10).map(|i| Some(i as f64)).collect();
        assert!(sma(&v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn sma_huge_period_no_panic() {
        let v: Vec<Option<f64>> = (0..5).map(|i| Some(i as f64)).collect();
        assert!(sma(&v, usize::MAX).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nonfinite_intermediate_does_not_corrupt_subsequent() {
        let v = vec![Some(10.0), Some(f64::NAN), Some(5.0)];
        let out = compute(&v);
        // First: 10. NaN skipped (acc stays 10). Third: 15.
        assert_eq!(out[0], Some(10.0));
        assert_eq!(out[1], Some(10.0));
        assert_eq!(out[2], Some(15.0));
    }
}
