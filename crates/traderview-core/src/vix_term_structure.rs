//! VIX term-structure analyzer.
//!
//! VIX9D / VIX / VIX3M / VIX6M / VIX1Y comprise the implied-vol term
//! structure. Normal CONTANGO = short-dated VIX < long-dated (fear
//! priced further out). BACKWARDATION = short > long (immediate
//! fear). Backwardation is a contrarian buy signal at extremes,
//! confirming downtrend pressure when persistent.
//!
//! Computes:
//!   - VIX/VIX3M ratio (the canonical backwardation gauge — > 1.0 = bear)
//!   - Slope of term structure (sum of consecutive differences)
//!   - Backwardation flag at standard thresholds
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VixTermStructure {
    pub vix9d: f64,
    pub vix:   f64,    // 30-day
    pub vix3m: f64,    // 90-day
    pub vix6m: f64,    // 180-day
    pub vix1y: f64,    // 365-day (VXM1Y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveState {
    SteepContango,    // vix/vix3m < 0.80
    Contango,         // 0.80 ≤ vix/vix3m < 1.00
    Flat,             // 1.00 ≤ vix/vix3m < 1.05
    Backwardation,    // 1.05 ≤ vix/vix3m < 1.20
    SevereBackwardation,    // ≥ 1.20
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TermStructureReport {
    pub vix_to_vix3m_ratio: f64,
    pub slope: f64,
    pub state: CurveState,
    pub note: String,
}

impl Default for CurveState {
    fn default() -> Self { CurveState::Flat }
}

pub fn analyze(ts: &VixTermStructure) -> TermStructureReport {
    let mut report = TermStructureReport::default();
    if ts.vix3m <= 0.0 { return report; }
    let ratio = ts.vix / ts.vix3m;
    report.vix_to_vix3m_ratio = ratio;
    report.slope = (ts.vix - ts.vix9d) + (ts.vix3m - ts.vix) + (ts.vix6m - ts.vix3m)
        + (ts.vix1y - ts.vix6m);
    report.state = if ratio < 0.80 { CurveState::SteepContango }
        else if ratio < 1.00 { CurveState::Contango }
        else if ratio < 1.05 { CurveState::Flat }
        else if ratio < 1.20 { CurveState::Backwardation }
        else { CurveState::SevereBackwardation };
    report.note = match report.state {
        CurveState::SteepContango       => "very calm market — short vol favored".into(),
        CurveState::Contango            => "normal market structure".into(),
        CurveState::Flat                => "neutral — front end aligning with mid term".into(),
        CurveState::Backwardation       => "near-term fear — caution on long exposure".into(),
        CurveState::SevereBackwardation => "extreme fear — historically a contrarian buy signal but in the moment indicates active stress".into(),
    };
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(vix9d: f64, vix: f64, vix3m: f64, vix6m: f64, vix1y: f64) -> VixTermStructure {
        VixTermStructure { vix9d, vix, vix3m, vix6m, vix1y }
    }

    #[test]
    fn normal_contango_classified() {
        // VIX 15, VIX3M 18 → ratio 0.833 → Contango.
        let r = analyze(&ts(13.0, 15.0, 18.0, 19.0, 20.0));
        assert_eq!(r.state, CurveState::Contango);
    }

    #[test]
    fn steep_contango_when_ratio_under_80() {
        let r = analyze(&ts(10.0, 12.0, 18.0, 19.0, 20.0));
        // 12/18 = 0.667 < 0.80.
        assert_eq!(r.state, CurveState::SteepContango);
    }

    #[test]
    fn flat_when_vix_approximates_vix3m() {
        // VIX 20, VIX3M 20 → ratio 1.00 → Flat.
        let r = analyze(&ts(20.0, 20.0, 20.0, 20.0, 20.0));
        assert_eq!(r.state, CurveState::Flat);
    }

    #[test]
    fn backwardation_when_vix_exceeds_vix3m_by_5pct() {
        // VIX 25, VIX3M 23 → ratio 1.087 → Backwardation.
        let r = analyze(&ts(28.0, 25.0, 23.0, 22.0, 22.0));
        assert_eq!(r.state, CurveState::Backwardation);
    }

    #[test]
    fn severe_backwardation_at_20pct_inversion() {
        // VIX 40, VIX3M 30 → ratio 1.333 → severe.
        let r = analyze(&ts(45.0, 40.0, 30.0, 28.0, 26.0));
        assert_eq!(r.state, CurveState::SevereBackwardation);
    }

    #[test]
    fn slope_positive_in_normal_contango() {
        let r = analyze(&ts(13.0, 15.0, 18.0, 19.0, 20.0));
        assert!(r.slope > 0.0, "contango = positive slope through tenors");
    }

    #[test]
    fn slope_negative_in_severe_backwardation() {
        let r = analyze(&ts(45.0, 40.0, 30.0, 28.0, 26.0));
        assert!(r.slope < 0.0, "backwardation = negative slope");
    }

    #[test]
    fn zero_vix3m_returns_default() {
        let r = analyze(&ts(15.0, 18.0, 0.0, 0.0, 0.0));
        assert_eq!(r.vix_to_vix3m_ratio, 0.0);
    }

    #[test]
    fn note_explains_state() {
        let r = analyze(&ts(45.0, 40.0, 30.0, 28.0, 26.0));
        assert!(r.note.contains("extreme") || r.note.contains("severe") || r.note.contains("stress"));
    }
}
