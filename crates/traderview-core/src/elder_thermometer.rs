//! Elder Market Thermometer — Alexander Elder ("Come Into My Trading Room", 2002).
//!
//! Per-bar measure of the larger of the high-to-prior-high and
//! prior-low-to-low excursions, smoothed with an EMA:
//!
//!   temp_t = max(|high_t - high_{t-1}|, |low_{t-1} - low_t|)
//!   avg_t  = EMA(temp, period)
//!
//! Elder's heuristics:
//!   - temp < 0.5 · avg → quiet, no trade
//!   - temp >= 0.5 · avg AND temp < avg → normal activity
//!   - temp >= avg AND temp < 1.5 · avg → hot, prepare to enter
//!   - temp >= 1.5 · avg → eruption, EXIT positions
//!
//! Output includes both raw temp + smoothed avg + per-bar regime tag.
//!
//! Pure compute. Companion to `atr_cone`, `keltner_squeeze`,
//! `volatility_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThermometerRegime {
    #[default]
    Quiet,
    Normal,
    Hot,
    Eruption,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElderThermometerReport {
    pub temperature: Vec<Option<f64>>,
    pub average: Vec<Option<f64>>,
    pub regime: Vec<Option<ThermometerRegime>>,
    pub period: usize,
}

pub fn compute(bars: &[Bar], period: usize) -> ElderThermometerReport {
    let n = bars.len();
    let mut report = ElderThermometerReport {
        temperature: vec![None; n],
        average: vec![None; n],
        regime: vec![None; n],
        period,
    };
    if period < 2 || n < period + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite())
    {
        return report;
    }
    for i in 1..n {
        let up = (bars[i].high - bars[i - 1].high).abs();
        let dn = (bars[i - 1].low - bars[i].low).abs();
        report.temperature[i] = Some(up.max(dn));
    }
    // EMA over the temperature series starting from index 1.
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let mut sum = 0.0_f64;
    for i in 1..=period {
        sum += report.temperature[i].unwrap();
    }
    let seed = sum / p_f;
    report.average[period] = Some(seed);
    let mut cur = seed;
    for i in (period + 1)..n {
        let t = report.temperature[i].unwrap();
        cur = t * k + cur * (1.0 - k);
        report.average[i] = Some(cur);
    }
    for i in 0..n {
        if let (Some(t), Some(a)) = (report.temperature[i], report.average[i]) {
            report.regime[i] = Some(classify(t, a));
        }
    }
    report
}

fn classify(temp: f64, avg: f64) -> ThermometerRegime {
    if temp < 0.5 * avg {
        ThermometerRegime::Quiet
    } else if temp < avg {
        ThermometerRegime::Normal
    } else if temp < 1.5 * avg {
        ThermometerRegime::Hot
    } else {
        ThermometerRegime::Eruption
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 1);
        assert!(r.average.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..5], 22);
        assert!(r2.average.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0); 30];
        bars[5] = b(f64::NAN, 99.0);
        let r = compute(&bars, 14);
        assert!(r.temperature.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_temp() {
        let bars = vec![b(101.0, 99.0); 50];
        let r = compute(&bars, 14);
        for v in r.temperature.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
        for v in r.average.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn classify_eruption_when_temp_exceeds_one_and_half_avg() {
        assert_eq!(classify(1.6, 1.0), ThermometerRegime::Eruption);
        assert_eq!(classify(1.5, 1.0), ThermometerRegime::Eruption);
    }

    #[test]
    fn classify_hot_for_temp_at_avg() {
        assert_eq!(classify(1.0, 1.0), ThermometerRegime::Hot);
        assert_eq!(classify(1.4, 1.0), ThermometerRegime::Hot);
    }

    #[test]
    fn classify_normal_for_half_to_full_avg() {
        assert_eq!(classify(0.5, 1.0), ThermometerRegime::Normal);
        assert_eq!(classify(0.9, 1.0), ThermometerRegime::Normal);
    }

    #[test]
    fn classify_quiet_below_half_avg() {
        assert_eq!(classify(0.0, 1.0), ThermometerRegime::Quiet);
        assert_eq!(classify(0.49, 1.0), ThermometerRegime::Quiet);
    }

    #[test]
    fn expanding_volatility_triggers_eruption() {
        // Constant range for 30 bars, then a 10x bar excursion.
        let mut bars = vec![b(101.0, 99.0); 40];
        bars[35] = b(120.0, 99.0);
        let r = compute(&bars, 14);
        // Index 35: temp = |120 - 101| = 19, avg ≈ 0+small.
        assert_eq!(r.regime[35].unwrap(), ThermometerRegime::Eruption);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0); 50];
        let r = compute(&bars, 14);
        assert_eq!(r.temperature.len(), 50);
        assert_eq!(r.average.len(), 50);
        assert_eq!(r.regime.len(), 50);
    }
}
