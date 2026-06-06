//! MAMA / FAMA — MESA Adaptive Moving Average + Following Adaptive MA
//! (John Ehlers, TASC 2001).
//!
//! Adaptive low-pass that adjusts its smoothing constant to the
//! market's dominant cycle period via a Hilbert-Transform homodyne
//! discriminator. The trigger line FAMA = EMA(MAMA, alpha/2.0)
//! provides a slower companion for crossover signals.
//!
//! This implementation uses a simplified but well-known public form
//! of Ehlers' algorithm (TASC, "MESA Adaptive Moving Averages"):
//!
//!   smooth = (4·x_t + 3·x_{t-1} + 2·x_{t-2} + x_{t-3}) / 10
//!   detrender = (0.0962·smooth_t + 0.5769·smooth_{t-2}
//!                - 0.5769·smooth_{t-4} - 0.0962·smooth_{t-6}) · (0.075·period_{t-1} + 0.54)
//!   I1, Q1: quadrature and in-phase components
//!   jI, jQ: 90° shifted versions
//!   I2 = I1 - jQ;  Q2 = Q1 + jI                         smoothed
//!   re = I2·I2_prev + Q2·Q2_prev;  im = I2·Q2_prev - Q2·I2_prev
//!   period = constrained 6..50 via atan2(im, re)
//!   alpha = max(min(fast_limit, 2 / (smooth_period + 1)), slow_limit)
//!   MAMA = α·x_t + (1 − α)·MAMA_{t-1}
//!   FAMA = 0.5·α·MAMA + (1 − 0.5·α)·FAMA_{t-1}
//!
//! Defaults: fast_limit = 0.5, slow_limit = 0.05.
//! Pure compute. Companion to `jurik_ma`, `kama`, `vidya`,
//! `roofing_filter`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MamaFamaReport {
    pub mama: Vec<Option<f64>>,
    pub fama: Vec<Option<f64>>,
    pub period: Vec<Option<f64>>,
    pub fast_limit: f64,
    pub slow_limit: f64,
}

pub fn compute(series: &[f64], fast_limit: f64, slow_limit: f64) -> MamaFamaReport {
    let n = series.len();
    let mut report = MamaFamaReport {
        mama: vec![None; n],
        fama: vec![None; n],
        period: vec![None; n],
        fast_limit,
        slow_limit,
    };
    if n < 7
        || !fast_limit.is_finite()
        || !slow_limit.is_finite()
        || fast_limit <= 0.0
        || slow_limit <= 0.0
        || slow_limit > fast_limit
    {
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
    let mut mama = vec![0.0_f64; n];
    let mut fama = vec![0.0_f64; n];
    // Seed pure-passthrough for first 6 bars (need ≥ index 6 to evaluate detrender).
    for i in 0..n.min(7) {
        mama[i] = series[i];
        fama[i] = series[i];
        report.mama[i] = Some(series[i]);
        report.fama[i] = Some(series[i]);
        report.period[i] = Some(0.0);
    }
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
        // Smooth I2, Q2.
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
        // Constrain period changes.
        if p > 1.5 * period[i - 1] {
            p = 1.5 * period[i - 1];
        }
        if p < 0.67 * period[i - 1] {
            p = 0.67 * period[i - 1];
        }
        p = p.clamp(6.0, 50.0);
        period[i] = 0.2 * p + 0.8 * period[i - 1];
        smooth_period[i] = 0.33 * period[i] + 0.67 * smooth_period[i - 1];
        let alpha = (fast_limit.min(2.0 / (smooth_period[i] + 1.0))).max(slow_limit);
        mama[i] = alpha * series[i] + (1.0 - alpha) * mama[i - 1];
        fama[i] = 0.5 * alpha * mama[i] + (1.0 - 0.5 * alpha) * fama[i - 1];
        report.mama[i] = Some(mama[i]);
        report.fama[i] = Some(fama[i]);
        report.period[i] = Some(period[i]);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s, 0.0, 0.05);
        assert!(r.mama.iter().all(|x| x.is_none()));
        let r2 = compute(&s, 0.05, 0.5); // slow > fast
        assert!(r2.mama.iter().all(|x| x.is_none()));
        let r3 = compute(&s[..3], 0.5, 0.05);
        assert!(r3.mama.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        let r = compute(&s, 0.5, 0.05);
        assert!(r.mama.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_signal_settles_to_constant() {
        let s = vec![100.0_f64; 100];
        let r = compute(&s, 0.5, 0.05);
        for v in r.mama.iter().skip(20).flatten() {
            assert!((v - 100.0).abs() < 1e-6);
        }
        for v in r.fama.iter().skip(20).flatten() {
            assert!((v - 100.0).abs() < 1e-6);
        }
    }

    #[test]
    fn linear_trend_tracks_input() {
        let s: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&s, 0.5, 0.05);
        let last_mama = r.mama[199].unwrap();
        // Lag bounded by 1/slow_limit ≈ 20 bars worst case.
        assert!((s[199] - last_mama).abs() < 25.0);
    }

    #[test]
    fn mama_leads_fama_in_uptrend() {
        let s: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&s, 0.5, 0.05);
        let last = 199;
        let m = r.mama[last].unwrap();
        let f = r.fama[last].unwrap();
        assert!(
            m >= f - 1e-9,
            "MAMA {m} should lead or equal FAMA {f} in steady uptrend"
        );
    }

    #[test]
    fn output_lengths_match_input() {
        let s = vec![100.0_f64; 100];
        let r = compute(&s, 0.5, 0.05);
        assert_eq!(r.mama.len(), 100);
        assert_eq!(r.fama.len(), 100);
        assert_eq!(r.period.len(), 100);
    }
}
