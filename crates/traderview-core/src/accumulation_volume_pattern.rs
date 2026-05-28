//! Accumulation Volume Pattern — Donald Cassidy ("It's When You Sell That Counts").
//!
//! Compares N-day average volume in the most recent up-moves vs
//! down-moves to detect institutional accumulation/distribution:
//!
//!   up_vol_avg_t   = mean(volume_i for i in last N bars where close_i > close_{i-1})
//!   down_vol_avg_t = mean(volume_i for i in last N bars where close_i < close_{i-1})
//!   ratio_t        = up_vol_avg_t / down_vol_avg_t  (if down_vol_avg_t > 0)
//!
//! Cassidy's reading:
//!   ratio > 1.5   → strong accumulation (institutions building positions)
//!   1.0 .. 1.5    → mild accumulation
//!   0.67 .. 1.0   → mild distribution
//!   ratio < 0.67  → strong distribution
//!
//! When both up_vol_avg and down_vol_avg can't be computed (e.g. all
//! up days), the indicator returns None.
//!
//! Pure compute. Default lookback = 50.
//! Companion to `accumulation_distribution_line`, `chaikin_money_flow`,
//! `volume_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub close: f64, pub volume: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AvpRegime {
    #[default]
    Neutral,
    StrongAccumulation,
    MildAccumulation,
    MildDistribution,
    StrongDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvpReport {
    pub ratio: Vec<Option<f64>>,
    pub regime: Vec<Option<AvpRegime>>,
    pub lookback: usize,
}

pub fn compute(bars: &[Bar], lookback: usize) -> AvpReport {
    let n = bars.len();
    let mut report = AvpReport {
        ratio: vec![None; n],
        regime: vec![None; n],
        lookback,
    };
    if lookback < 2 || n < lookback + 1 { return report; }
    if bars.iter().any(|b| !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0) {
        return report;
    }
    for i in lookback..n {
        let win_start = i + 1 - lookback;
        let mut up_sum = 0.0_f64;
        let mut up_cnt = 0_usize;
        let mut dn_sum = 0.0_f64;
        let mut dn_cnt = 0_usize;
        for k in win_start..=i {
            if k == 0 { continue; }
            let d = bars[k].close - bars[k - 1].close;
            if d > 0.0 { up_sum += bars[k].volume; up_cnt += 1; }
            else if d < 0.0 { dn_sum += bars[k].volume; dn_cnt += 1; }
        }
        if up_cnt == 0 || dn_cnt == 0 { continue; }
        let up_avg = up_sum / up_cnt as f64;
        let dn_avg = dn_sum / dn_cnt as f64;
        if dn_avg <= 0.0 { continue; }
        let ratio = up_avg / dn_avg;
        report.ratio[i] = Some(ratio);
        report.regime[i] = Some(classify(ratio));
    }
    report
}

fn classify(ratio: f64) -> AvpRegime {
    if ratio > 1.5 { AvpRegime::StrongAccumulation }
    else if ratio > 1.0 { AvpRegime::MildAccumulation }
    else if ratio > 0.67 { AvpRegime::MildDistribution }
    else { AvpRegime::StrongDistribution }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar { Bar { close: c, volume: v } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 1000.0); 60];
        let r = compute(&bars, 1);
        assert!(r.ratio.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..10], 50);
        assert!(r2.ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(100.0, 1000.0); 60];
        bars[5] = b(f64::NAN, 1000.0);
        let r = compute(&bars, 50);
        assert!(r.ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn high_up_volume_classifies_strong_accumulation() {
        // 30 alternating bars with up-volume 5000 vs down-volume 1000.
        let bars: Vec<_> = (0_usize..60).map(|i| {
            if i.is_multiple_of(2) { b(101.0, 5000.0) }
            else { b(100.0, 1000.0) }
        }).collect();
        let r = compute(&bars, 50);
        let last = 59;
        assert!(r.ratio[last].is_some());
        assert!(r.ratio[last].unwrap() > 1.5);
        assert_eq!(r.regime[last].unwrap(), AvpRegime::StrongAccumulation);
    }

    #[test]
    fn high_down_volume_classifies_strong_distribution() {
        let bars: Vec<_> = (0_usize..60).map(|i| {
            if i.is_multiple_of(2) { b(101.0, 1000.0) }
            else { b(100.0, 5000.0) }
        }).collect();
        let r = compute(&bars, 50);
        let last = 59;
        assert!(r.ratio[last].unwrap() < 0.67);
        assert_eq!(r.regime[last].unwrap(), AvpRegime::StrongDistribution);
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(2.0), AvpRegime::StrongAccumulation);
        assert_eq!(classify(1.2), AvpRegime::MildAccumulation);
        assert_eq!(classify(0.8), AvpRegime::MildDistribution);
        assert_eq!(classify(0.5), AvpRegime::StrongDistribution);
    }

    #[test]
    fn all_up_or_all_down_returns_none() {
        let bars: Vec<_> = (0..60).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        let r = compute(&bars, 50);
        // All up days, no down → None.
        let last = 59;
        assert!(r.ratio[last].is_none());
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100.0, 1000.0); 60];
        let r = compute(&bars, 50);
        assert_eq!(r.ratio.len(), 60);
        assert_eq!(r.regime.len(), 60);
    }
}
