//! Chande Trend Index (CTI) — Tushar Chande.
//!
//! Measures the linearity (trend-strength) of price over a window via
//! the correlation between the closing-price series and a perfect
//! linear ramp:
//!
//!   CTI = corr(closes[t-N+1..=t], 1..=N)
//!
//! Range [−1, +1]:
//!   +1   = perfect uptrend (closes monotone increasing)
//!   −1   = perfect downtrend
//!    0   = no trend / random walk
//!
//! Default period N = 14.
//!
//! Distinct from CMO (Chande Momentum Oscillator) and the choppiness
//! index. Specifically a TREND-STRENGTH metric, not momentum.
//!
//! Pure compute. Companion to `aroon_indicator`, `chande_momentum_oscillator`,
//! `choppiness`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let n_f = period as f64;
    // x = 1..N, sum = N(N+1)/2, sum_sq = N(N+1)(2N+1)/6.
    let sx: f64 = (1..=period).map(|i| i as f64).sum();
    let sx2: f64 = (1..=period).map(|i| (i * i) as f64).sum();
    let xbar = sx / n_f;
    let sxx = sx2 - n_f * xbar * xbar;
    if sxx <= 0.0 {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        let ybar: f64 = win.iter().sum::<f64>() / n_f;
        let mut sxy = 0.0_f64;
        let mut syy = 0.0_f64;
        for (k, y) in win.iter().enumerate() {
            let dx = (k + 1) as f64 - xbar;
            let dy = y - ybar;
            sxy += dx * dy;
            syy += dy * dy;
        }
        if syy > 0.0 {
            *slot = Some((sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0));
        } else {
            *slot = Some(0.0);
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
    fn invalid_period_returns_all_none() {
        let c = vec![100.0_f64; 30];
        assert!(compute(&c, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let c = vec![100.0, f64::NAN, 101.0, 102.0, 103.0];
        assert!(compute(&c, 3).iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfect_uptrend_yields_cti_one() {
        let c: Vec<f64> = (1..=30).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 14);
        let last = r[29].unwrap();
        assert!((last - 1.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_downtrend_yields_cti_minus_one() {
        let c: Vec<f64> = (0..30).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 14);
        let last = r[29].unwrap();
        assert!((last + 1.0).abs() < 1e-9);
    }

    #[test]
    fn flat_market_yields_zero_cti() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 14);
        let last = r[29].unwrap();
        assert_eq!(last, 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let c: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0)
            .collect();
        let r = compute(&c, 14);
        assert_eq!(r.len(), 50);
        assert!(r[12].is_none());
        assert!(r[13].is_some());
    }
}
