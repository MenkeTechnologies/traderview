//! McClellan Oscillator — market-breadth momentum.
//!
//! Definition (Sherman & Marian McClellan, 1969):
//!   - **Net advances** = `advancing_issues − declining_issues`
//!   - **19-day EMA** of net advances
//!   - **39-day EMA** of net advances
//!   - **Oscillator** = 19EMA − 39EMA
//!
//! The "summation index" is the running cumulative sum of the oscillator
//! and is also reported.
//!
//! Reading:
//!   - Oscillator > 0  → bullish breadth momentum
//!   - Oscillator < 0  → bearish breadth momentum
//!   - Extremes (|osc| > 100) → overbought / oversold breadth
//!   - Summation crossing zero from negative → major buy signal
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BreadthBar {
    pub advancing_issues: i64,
    pub declining_issues: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum McClellanRegime {
    OverboughtBreadth,
    Bullish,
    #[default]
    Neutral,
    Bearish,
    OversoldBreadth,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McClellanReport {
    pub oscillator: Vec<Option<f64>>,
    pub summation_index: Vec<Option<f64>>,
    pub latest_osc: Option<f64>,
    pub latest_summation: Option<f64>,
    pub regime: McClellanRegime,
}

fn ema_smoothing(period: usize) -> f64 {
    2.0 / (period as f64 + 1.0)
}

pub fn compute(bars: &[BreadthBar]) -> McClellanReport {
    let n = bars.len();
    // Need at least one bar to seed both EMAs and one more for the EMA to actually evolve.
    if n == 0 {
        return McClellanReport::default();
    }
    let k_short = ema_smoothing(19);
    let k_long = ema_smoothing(39);
    // Compute net advances in f64 to avoid an i64 subtraction overflow on
    // pathological JSON inputs (e.g. advancing_issues=i64::MIN paired with
    // a positive declining_issues panics in debug, wraps in release).
    // Real exchange data is tiny so this is purely a hardening fix.
    let net: Vec<f64> = bars
        .iter()
        .map(|b| b.advancing_issues as f64 - b.declining_issues as f64)
        .collect();
    // Seed both EMAs with the first net-advances value.
    let mut ema_short = net[0];
    let mut ema_long = net[0];
    let mut osc: Vec<Option<f64>> = Vec::with_capacity(n);
    let mut sum_idx: Vec<Option<f64>> = Vec::with_capacity(n);
    let mut running = 0.0_f64;
    // First bar — both EMAs equal the seed; oscillator is 0 by definition.
    osc.push(Some(0.0));
    sum_idx.push(Some(0.0));
    for &v in net.iter().skip(1) {
        ema_short = v * k_short + ema_short * (1.0 - k_short);
        ema_long = v * k_long + ema_long * (1.0 - k_long);
        let o = ema_short - ema_long;
        running += o;
        osc.push(Some(o));
        sum_idx.push(Some(running));
    }
    let latest_osc = osc.last().copied().flatten();
    let latest_summation = sum_idx.last().copied().flatten();
    let regime = match latest_osc {
        Some(v) if v > 100.0 => McClellanRegime::OverboughtBreadth,
        Some(v) if v > 0.0 => McClellanRegime::Bullish,
        Some(v) if v < -100.0 => McClellanRegime::OversoldBreadth,
        Some(v) if v < 0.0 => McClellanRegime::Bearish,
        _ => McClellanRegime::Neutral,
    };
    McClellanReport {
        oscillator: osc,
        summation_index: sum_idx,
        latest_osc,
        latest_summation,
        regime,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(ai: i64, di: i64) -> BreadthBar {
        BreadthBar {
            advancing_issues: ai,
            declining_issues: di,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[]);
        assert!(r.latest_osc.is_none());
    }

    #[test]
    fn first_bar_oscillator_seeded_zero() {
        let bars = vec![bar(2000, 1000)];
        let r = compute(&bars);
        assert!((r.latest_osc.unwrap() - 0.0).abs() < 1e-9);
        assert!((r.latest_summation.unwrap() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn sustained_positive_breadth_drives_osc_positive() {
        // Start from balanced (seed 0) then jump to strongly positive breadth.
        // Short-19 EMA rises faster than long-39 EMA on the step, so the
        // oscillator is positive throughout the ramp.
        let mut bars = vec![bar(1500, 1500); 5]; // seed near zero net
        bars.extend(vec![bar(2000, 1000); 40]); // sustained +1000 net
        let r = compute(&bars);
        let v = r.latest_osc.unwrap();
        assert!(
            v > 0.0,
            "rising-breadth oscillator should be positive, got {v}"
        );
        assert!(r.latest_summation.unwrap() > 0.0);
        assert!(matches!(
            r.regime,
            McClellanRegime::Bullish | McClellanRegime::OverboughtBreadth
        ));
    }

    #[test]
    fn breadth_flip_inverts_oscillator() {
        // Run 30 bars positive then 30 bars negative — oscillator should end up negative.
        let mut bars = vec![bar(2000, 1000); 30];
        bars.extend(vec![bar(1000, 2000); 30]);
        let r = compute(&bars);
        let v = r.latest_osc.unwrap();
        assert!(v < 0.0, "post-flip oscillator should be negative, got {v}");
        assert!(matches!(
            r.regime,
            McClellanRegime::Bearish | McClellanRegime::OversoldBreadth
        ));
    }

    #[test]
    fn summation_index_is_cumulative() {
        let bars = vec![
            bar(2000, 1000),
            bar(2000, 1000),
            bar(2000, 1000),
            bar(1000, 2000),
            bar(1000, 2000),
        ];
        let r = compute(&bars);
        // Manually verify summation = running sum of oscillator.
        let mut running = 0.0;
        for i in 0..bars.len() {
            running += r.oscillator[i].unwrap();
            assert!((r.summation_index[i].unwrap() - running).abs() < 1e-9);
        }
    }

    #[test]
    fn extreme_overbought_classified() {
        // Force a strong positive impulse to push oscillator > 100.
        // 1-bar large net-advance starting from balanced seed.
        let mut bars = vec![bar(1500, 1500); 5]; // seed
        bars.push(bar(10_000, 100)); // big up-spike
        let r = compute(&bars);
        let v = r.latest_osc.unwrap();
        assert!(v > 100.0, "expected overbought breadth (>100), got {v}");
        assert!(matches!(r.regime, McClellanRegime::OverboughtBreadth));
    }
}
