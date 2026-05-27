//! Rate of Change momentum indicator.
//!
//!   ROC_t = (close_t - close_{t-N}) / close_{t-N} × 100
//!
//! Simple momentum: percent change over N periods. Crossing zero =
//! trend change. >0 = positive momentum, <0 = negative.
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<f64> {
    let n = closes.len();
    let mut out = vec![0.0; n];
    if n <= period || period == 0 { return out; }
    for i in period..n {
        let prior = closes[i - period];
        if prior > 0.0 {
            out[i] = (closes[i] - prior) / prior * 100.0;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10).is_empty());
    }

    #[test]
    fn under_period_zeros() {
        let out = compute(&[100.0, 105.0], 10);
        for v in &out { assert_eq!(*v, 0.0); }
    }

    #[test]
    fn doubling_yields_100pct_roc() {
        let out = compute(&[100.0, 100.0, 200.0], 2);
        assert_eq!(out[2], 100.0);
    }

    #[test]
    fn halving_yields_minus_50pct_roc() {
        let out = compute(&[100.0, 100.0, 50.0], 2);
        assert_eq!(out[2], -50.0);
    }

    #[test]
    fn flat_yields_zero_roc() {
        let out = compute(&[100.0; 10], 5);
        for v in &out[5..] { assert_eq!(*v, 0.0); }
    }

    #[test]
    fn zero_prior_close_returns_zero() {
        let out = compute(&[0.0, 100.0, 200.0], 2);
        assert_eq!(out[2], 0.0);
    }
}
