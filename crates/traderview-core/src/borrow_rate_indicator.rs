//! Borrow Rate Indicator — annualized hard-to-borrow stress gauge.
//!
//! Tracks the per-bar securities-lending fee paid to short a stock,
//! plus its rate of change, to detect when short demand is spiking
//! (often a leading indicator of squeeze risk).
//!
//!   change_pct_t  = (rate_t - rate_{t-period}) / rate_{t-period} · 100
//!   stress_level  = classify(rate, change_pct):
//!     LowAvailable  : rate < 1%
//!     Normal        : 1 ≤ rate < 10
//!     Tight         : 10 ≤ rate < 50
//!     HardToBorrow  : 50 ≤ rate < 200
//!     ExtremeSqueeze: rate ≥ 200 OR change_pct ≥ 100
//!
//! Pure compute. Default period = 5.
//! Companion to `short_interest_scanner`, `unusual_options_activity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BorrowStress {
    #[default]
    LowAvailable,
    Normal,
    Tight,
    HardToBorrow,
    ExtremeSqueeze,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BorrowRateReport {
    pub change_pct: Vec<Option<f64>>,
    pub stress: Vec<Option<BorrowStress>>,
    pub period: usize,
}

pub fn compute(rates_pct: &[f64], period: usize) -> BorrowRateReport {
    let n = rates_pct.len();
    let mut report = BorrowRateReport {
        change_pct: vec![None; n],
        stress: vec![None; n],
        period,
    };
    if period < 1 || n < period + 1 {
        return report;
    }
    if rates_pct.iter().any(|x| !x.is_finite() || *x < 0.0) {
        return report;
    }
    for i in 0..n {
        let cur = rates_pct[i];
        let change_pct = if i >= period {
            let prev = rates_pct[i - period];
            if prev > 0.0 {
                Some((cur - prev) / prev * 100.0)
            } else {
                None
            }
        } else {
            None
        };
        report.change_pct[i] = change_pct;
        let stress = classify(cur, change_pct.unwrap_or(0.0));
        report.stress[i] = Some(stress);
    }
    report
}

fn classify(rate: f64, change_pct: f64) -> BorrowStress {
    if rate >= 200.0 || change_pct >= 100.0 {
        BorrowStress::ExtremeSqueeze
    } else if rate >= 50.0 {
        BorrowStress::HardToBorrow
    } else if rate >= 10.0 {
        BorrowStress::Tight
    } else if rate >= 1.0 {
        BorrowStress::Normal
    } else {
        BorrowStress::LowAvailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let r = vec![0.5_f64; 10];
        let rep = compute(&r, 0);
        assert!(rep.change_pct.iter().all(|x| x.is_none()));
        let rep2 = compute(&r[..2], 5);
        assert!(rep2.change_pct.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut r = vec![0.5_f64; 10];
        r[3] = f64::NAN;
        let rep = compute(&r, 5);
        assert!(rep.change_pct.iter().all(|x| x.is_none()));
        let mut r2 = vec![0.5_f64; 10];
        r2[3] = -1.0;
        let rep2 = compute(&r2, 5);
        assert!(rep2.change_pct.iter().all(|x| x.is_none()));
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(0.5, 0.0), BorrowStress::LowAvailable);
        assert_eq!(classify(5.0, 0.0), BorrowStress::Normal);
        assert_eq!(classify(25.0, 0.0), BorrowStress::Tight);
        assert_eq!(classify(100.0, 0.0), BorrowStress::HardToBorrow);
        assert_eq!(classify(300.0, 0.0), BorrowStress::ExtremeSqueeze);
        assert_eq!(classify(5.0, 200.0), BorrowStress::ExtremeSqueeze);
    }

    #[test]
    fn spike_triggers_extreme_squeeze() {
        // Rate stable at 5% then jumps to 12% (140% change) → extreme.
        let mut r = vec![5.0_f64; 10];
        r.push(12.0);
        let rep = compute(&r, 5);
        assert_eq!(rep.stress[10].unwrap(), BorrowStress::ExtremeSqueeze);
    }

    #[test]
    fn high_rate_alone_triggers_squeeze() {
        let r = vec![250.0_f64; 10];
        let rep = compute(&r, 5);
        for s in rep.stress.iter().skip(5).flatten() {
            assert_eq!(*s, BorrowStress::ExtremeSqueeze);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let r = vec![5.0_f64; 10];
        let rep = compute(&r, 5);
        assert_eq!(rep.change_pct.len(), 10);
        assert_eq!(rep.stress.len(), 10);
    }
}
