//! Chande Momentum Oscillator (CMO) — Tushar Chande.
//!
//! Sum of up-day price changes minus sum of down-day price changes,
//! normalized by the sum of absolute changes over the lookback:
//!
//!   sum_up   = sum of (close_t − close_{t-1}) for up days in last `period`
//!   sum_down = sum of |close_t − close_{t-1}| for down days
//!   CMO = 100 × (sum_up − sum_down) / (sum_up + sum_down)
//!
//! Range −100..=+100. Unlike RSI (which always lives 0..100), CMO is
//! signed. Convention: >+50 strong up, <−50 strong down.
//!
//! Pure compute. Standard period = 14.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period.saturating_add(1) {
        return out;
    }
    let mut up = vec![0.0_f64; n];
    let mut down = vec![0.0_f64; n];
    for i in 1..n {
        let d = closes[i] - closes[i - 1];
        if d > 0.0 {
            up[i] = d;
        } else if d < 0.0 {
            down[i] = -d;
        }
    }
    for i in period..n {
        let sum_up: f64 = up[i + 1 - period..=i].iter().sum();
        let sum_dn: f64 = down[i + 1 - period..=i].iter().sum();
        let denom = sum_up + sum_dn;
        if denom > 0.0 {
            let v = 100.0 * (sum_up - sum_dn) / denom;
            if v.is_finite() {
                out[i] = Some(v);
            }
        }
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
    fn period_zero_returns_all_none() {
        let v = vec![100.0; 30];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_cmo_undefined() {
        // No up or down days → denom = 0 → None.
        let v = vec![100.0; 30];
        let out = compute(&v, 14);
        for v in &out {
            assert!(v.is_none());
        }
    }

    #[test]
    fn all_up_yields_cmo_100() {
        let v: Vec<f64> = (1..=30).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 14);
        let last = out[29].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn all_down_yields_cmo_minus_100() {
        let v: Vec<f64> = (1..=30).map(|i| 200.0 - i as f64).collect();
        let out = compute(&v, 14);
        let last = out[29].expect("populated");
        assert!((last + 100.0).abs() < 1e-9);
    }

    #[test]
    fn balanced_changes_cmo_near_zero() {
        // Alternating +1/-1 → sum_up ≈ sum_down.
        let v: Vec<f64> = (0..30)
            .map(|i| if i % 2 == 0 { 100.0 } else { 101.0 })
            .collect();
        let out = compute(&v, 14);
        let last = out[29].expect("populated");
        assert!(last.abs() < 10.0, "got {last}");
    }

    #[test]
    fn output_in_range_minus_100_to_100() {
        let v: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.3).sin() * 10.0)
            .collect();
        let out = compute(&v, 14);
        for x in out.iter().flatten() {
            assert!((-100.0..=100.0).contains(x), "CMO out of range: {x}");
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
