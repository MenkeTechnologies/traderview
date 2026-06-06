//! Momentum Persistence Index — rolling fraction of consecutive same-
//! direction bars.
//!
//!   over the last `period` bars, fraction of bars where:
//!     close_t > close_{t-1}  → up_count
//!     close_t < close_{t-1}  → down_count
//!
//!   persistence_t = max(up_count, down_count) / period
//!     (range [0.5, 1.0]; closer to 1.0 = stronger one-sided momentum)
//!
//! Signed variant:
//!   signed_t = (up_count - down_count) / period
//!     (range [-1.0, +1.0]; sign indicates direction)
//!
//! Pure compute. Default period = 20.
//! Companion to `efficiency_ratio`, `chande_trend_index`,
//! `choppy_market_index`, `ehlers_correlation_trend_indicator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MomentumPersistenceReport {
    pub persistence: Vec<Option<f64>>,
    pub signed_persistence: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(closes: &[f64], period: usize) -> MomentumPersistenceReport {
    let n = closes.len();
    let mut report = MomentumPersistenceReport {
        persistence: vec![None; n],
        signed_persistence: vec![None; n],
        period,
    };
    if period < 2 || n < period + 1 {
        return report;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let p_f = period as f64;
    for i in period..n {
        let win = &closes[i + 1 - period..=i];
        let mut up = 0_usize;
        let mut dn = 0_usize;
        for w in win.windows(2) {
            if w[1] > w[0] {
                up += 1;
            } else if w[1] < w[0] {
                dn += 1;
            }
        }
        // Include the first bar of the window vs the bar BEFORE the window
        // (closes[i + 1 - period - 1] = closes[i - period]).
        let first_in_win = win[0];
        let bar_before = closes[i - period];
        if first_in_win > bar_before {
            up += 1;
        } else if first_in_win < bar_before {
            dn += 1;
        }
        let max_count = up.max(dn) as f64;
        report.persistence[i] = Some(max_count / p_f);
        report.signed_persistence[i] = Some((up as f64 - dn as f64) / p_f);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 1);
        assert!(r.persistence.iter().all(|x| x.is_none()));
        let r2 = compute(&c[..5], 20);
        assert!(r2.persistence.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 30];
        c[5] = f64::NAN;
        let r = compute(&c, 20);
        assert!(r.persistence.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_persistence() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 20);
        // No directional movement → both up and dn are 0 → persistence = 0.
        for v in r.persistence.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
        for v in r.signed_persistence.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn pure_uptrend_yields_unit_persistence() {
        let c: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 20);
        let last = 29;
        assert!((r.persistence[last].unwrap() - 1.0).abs() < 1e-9);
        assert!((r.signed_persistence[last].unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pure_downtrend_yields_negative_unit_signed() {
        let c: Vec<f64> = (0..30).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 20);
        let last = 29;
        assert!((r.persistence[last].unwrap() - 1.0).abs() < 1e-9);
        assert!((r.signed_persistence[last].unwrap() + 1.0).abs() < 1e-9);
    }

    #[test]
    fn alternating_yields_half_persistence() {
        // Alternating up/down → up and dn each = 10 of 20 → persistence = 0.5.
        let c: Vec<f64> = (0_usize..30)
            .map(|i| if i.is_multiple_of(2) { 100.0 } else { 101.0 })
            .collect();
        let r = compute(&c, 20);
        let last = 29;
        assert!((r.persistence[last].unwrap() - 0.5).abs() < 1e-9);
        assert!(r.signed_persistence[last].unwrap().abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 20);
        assert_eq!(r.persistence.len(), 30);
        assert_eq!(r.signed_persistence.len(), 30);
    }
}
