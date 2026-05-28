//! Arms High-Low Index — Richard Arms.
//!
//! Smoothed market-breadth ratio:
//!
//!   raw_t      = new_highs_t / (new_highs_t + new_lows_t) · 100
//!   ahli_t     = EMA(raw, period)
//!
//! Range [0, 100]:
//!   AHLI > 70 → strong bull market (most issues making new highs)
//!   AHLI < 30 → strong bear market (most issues making new lows)
//!   AHLI ≈ 50 → balanced / transition
//!
//! Distinct from arms_index (TRIN), which uses up-vs-down volume.
//! AHLI uses 52-week new highs and new lows.
//!
//! Pure compute. Default period = 10.
//! Companion to `arms_index`, `mcclellan_oscillator`, `breadth_lines`,
//! `fifty_two_week_high_low_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyBreadth {
    pub new_highs: u64,
    pub new_lows: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArmsHighLowReport {
    pub raw_ratio: Vec<Option<f64>>,
    pub ahli: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(breadth: &[DailyBreadth], period: usize) -> ArmsHighLowReport {
    let n = breadth.len();
    let mut report = ArmsHighLowReport {
        raw_ratio: vec![None; n],
        ahli: vec![None; n],
        period,
    };
    if period < 2 || n < period { return report; }
    let raw: Vec<f64> = breadth.iter().map(|d| {
        let denom = d.new_highs + d.new_lows;
        if denom > 0 {
            d.new_highs as f64 / denom as f64 * 100.0
        } else {
            50.0
        }
    }).collect();
    for (i, &v) in raw.iter().enumerate() {
        report.raw_ratio[i] = Some(v);
    }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = raw[..period].iter().sum::<f64>() / p_f;
    report.ahli[period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in raw.iter().enumerate().skip(period) {
        cur = v * k + cur * (1.0 - k);
        report.ahli[i] = Some(cur);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(highs: u64, lows: u64) -> DailyBreadth {
        DailyBreadth { new_highs: highs, new_lows: lows }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let breadth = vec![d(50, 50); 30];
        let r = compute(&breadth, 1);
        assert!(r.ahli.iter().all(|x| x.is_none()));
        let r2 = compute(&breadth[..5], 10);
        assert!(r2.ahli.iter().all(|x| x.is_none()));
    }

    #[test]
    fn zero_zero_yields_fifty_neutral() {
        let breadth = vec![d(0, 0); 30];
        let r = compute(&breadth, 10);
        for v in r.ahli.iter().flatten() {
            assert!((v - 50.0).abs() < 1e-9);
        }
    }

    #[test]
    fn all_new_highs_yields_hundred() {
        let breadth = vec![d(100, 0); 30];
        let r = compute(&breadth, 10);
        let last = r.ahli[29].unwrap();
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn all_new_lows_yields_zero() {
        let breadth = vec![d(0, 100); 30];
        let r = compute(&breadth, 10);
        let last = r.ahli[29].unwrap();
        assert!(last.abs() < 1e-9);
    }

    #[test]
    fn balanced_breadth_near_fifty() {
        let breadth = vec![d(50, 50); 30];
        let r = compute(&breadth, 10);
        for v in r.ahli.iter().flatten() {
            assert!((v - 50.0).abs() < 1e-9);
        }
    }

    #[test]
    fn output_in_zero_hundred_range() {
        let mut state: u64 = 42;
        let breadth: Vec<_> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            let total = 100;
            let highs = (r * total as f64) as u64;
            d(highs, total - highs)
        }).collect();
        let r = compute(&breadth, 10);
        for v in r.ahli.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let breadth = vec![d(50, 50); 30];
        let r = compute(&breadth, 10);
        assert_eq!(r.raw_ratio.len(), 30);
        assert_eq!(r.ahli.len(), 30);
    }
}
