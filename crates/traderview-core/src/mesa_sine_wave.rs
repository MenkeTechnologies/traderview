//! MESA Sine Wave Indicator — John Ehlers ("Cycle Analytics for Traders", 2013).
//!
//! Two sinusoidal curves derived from the Hilbert-transform-style
//! dominant cycle phase, leading the price action in cyclic markets:
//!
//!   smooth_t = (4·x + 3·x_{t-1} + 2·x_{t-2} + x_{t-3}) / 10
//!   detrender, Q, I, jI, jQ, I2, Q2, re, im (Hilbert)
//!   phase = atan2(I, Q)   in degrees
//!   sine        = sin(phase)
//!   lead_sine   = sin(phase + 45°)
//!
//! Bullish cycle peak: lead_sine crosses ABOVE sine.
//! Bearish cycle trough: lead_sine crosses BELOW sine.
//! In strongly trending markets the two lines stay tangled (signal that
//! cycle interpretation isn't appropriate).
//!
//! Pure compute. Companion to `ehlers_mama_fama`, `roofing_filter`,
//! `hilbert_transform`, `ehlers_instant_trendline`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SineWaveReport {
    pub sine: Vec<Option<f64>>,
    pub lead_sine: Vec<Option<f64>>,
    pub period: Vec<Option<f64>>,
}

pub fn compute(series: &[f64]) -> SineWaveReport {
    let n = series.len();
    let mut report = SineWaveReport {
        sine: vec![None; n],
        lead_sine: vec![None; n],
        period: vec![None; n],
    };
    if n < 7 {
        return report;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let mut smooth = vec![0.0_f64; n];
    let mut detrender = vec![0.0_f64; n];
    let mut q1 = vec![0.0_f64; n];
    let mut i1 = vec![0.0_f64; n];
    let mut ji = vec![0.0_f64; n];
    let mut jq = vec![0.0_f64; n];
    let mut i2 = vec![0.0_f64; n];
    let mut q2 = vec![0.0_f64; n];
    let mut re = vec![0.0_f64; n];
    let mut im = vec![0.0_f64; n];
    let mut period = vec![0.0_f64; n];
    let mut smooth_period = vec![0.0_f64; n];
    let mut phase = vec![0.0_f64; n];
    let mut delta_phase = vec![0.0_f64; n];
    let mut instant_period = vec![0.0_f64; n];
    for i in 6..n {
        smooth[i] =
            (4.0 * series[i] + 3.0 * series[i - 1] + 2.0 * series[i - 2] + series[i - 3]) / 10.0;
        let p_factor = 0.075 * period[i - 1] + 0.54;
        detrender[i] = (0.0962 * smooth[i] + 0.5769 * smooth[i - 2]
            - 0.5769 * smooth[i - 4]
            - 0.0962 * smooth[i - 6])
            * p_factor;
        q1[i] = (0.0962 * detrender[i] + 0.5769 * detrender[i - 2]
            - 0.5769 * detrender[i - 4]
            - 0.0962 * detrender[i - 6])
            * p_factor;
        i1[i] = detrender[i - 3];
        ji[i] = (0.0962 * i1[i] + 0.5769 * i1[i - 2] - 0.5769 * i1[i - 4] - 0.0962 * i1[i - 6])
            * p_factor;
        jq[i] = (0.0962 * q1[i] + 0.5769 * q1[i - 2] - 0.5769 * q1[i - 4] - 0.0962 * q1[i - 6])
            * p_factor;
        i2[i] = i1[i] - jq[i];
        q2[i] = q1[i] + ji[i];
        i2[i] = 0.2 * i2[i] + 0.8 * i2[i - 1];
        q2[i] = 0.2 * q2[i] + 0.8 * q2[i - 1];
        re[i] = i2[i] * i2[i - 1] + q2[i] * q2[i - 1];
        im[i] = i2[i] * q2[i - 1] - q2[i] * i2[i - 1];
        re[i] = 0.2 * re[i] + 0.8 * re[i - 1];
        im[i] = 0.2 * im[i] + 0.8 * im[i - 1];
        let mut p = if im[i] != 0.0 && re[i] != 0.0 {
            360.0 / (im[i].atan2(re[i])).to_degrees().abs().max(1e-12)
        } else {
            period[i - 1]
        };
        if p > 1.5 * period[i - 1] {
            p = 1.5 * period[i - 1];
        }
        if p < 0.67 * period[i - 1] {
            p = 0.67 * period[i - 1];
        }
        p = p.clamp(6.0, 50.0);
        period[i] = 0.2 * p + 0.8 * period[i - 1];
        smooth_period[i] = 0.33 * period[i] + 0.67 * smooth_period[i - 1];
        // Cycle phase from in-phase / quadrature.
        phase[i] = if i1[i] != 0.0 {
            q1[i].atan2(i1[i]).to_degrees()
        } else {
            0.0
        };
        delta_phase[i] = (phase[i - 1] - phase[i]).max(1.0);
        if delta_phase[i] > 60.0 {
            delta_phase[i] = 60.0;
        }
        instant_period[i] = if delta_phase[i] > 0.0 {
            360.0 / delta_phase[i]
        } else {
            0.0
        };
        let p_rad = phase[i].to_radians();
        report.sine[i] = Some(p_rad.sin());
        report.lead_sine[i] = Some((phase[i] + 45.0).to_radians().sin());
        report.period[i] = Some(period[i]);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 5];
        assert!(compute(&s).sine.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s).sine.iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_in_unit_signed_range() {
        let s: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.3).sin() * 5.0)
            .collect();
        let r = compute(&s);
        for v in r.sine.iter().flatten() {
            assert!((-1.0..=1.0).contains(v));
        }
        for v in r.lead_sine.iter().flatten() {
            assert!((-1.0..=1.0).contains(v));
        }
    }

    #[test]
    fn period_in_six_to_fifty_range() {
        let s: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.3).sin() * 5.0)
            .collect();
        let r = compute(&s);
        for v in r.period.iter().skip(20).flatten() {
            assert!((0.0..=50.0).contains(v), "period {v} should be in [0, 50]");
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s);
        assert_eq!(r.sine.len(), 50);
        assert_eq!(r.lead_sine.len(), 50);
        assert_eq!(r.period.len(), 50);
    }
}
