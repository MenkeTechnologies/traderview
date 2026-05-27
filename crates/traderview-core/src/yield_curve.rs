//! Yield-curve shape classifier.
//!
//! Given UST yields at standard tenors (3M, 2Y, 5Y, 10Y, 30Y), classify
//! the shape:
//!
//!   - **Normal**: monotonic upward slope (long yields > short yields).
//!     Healthy economy expanding.
//!   - **Flat**: little spread between short and long ends.
//!     Late-cycle / uncertainty.
//!   - **Inverted**: short yields > long yields. Classic recession signal.
//!     Specifically 2Y-10Y inversion is the canonical indicator.
//!   - **Humped**: short end LOW + mid bulges + long end LOW.
//!     Rare; usually transitional.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct YieldCurve {
    pub y3m: f64,
    pub y2y: f64,
    pub y5y: f64,
    pub y10y: f64,
    pub y30y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveShape { Normal, Flat, Inverted, Humped }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveReport {
    pub shape: CurveShape,
    /// 10Y - 2Y spread in bps. Negative = canonical recession signal.
    pub spread_10y_2y_bps: f64,
    /// 10Y - 3M spread (the Fed's preferred measure).
    pub spread_10y_3m_bps: f64,
    pub note: String,
}

pub fn classify(c: &YieldCurve) -> CurveReport {
    let spread_10_2 = (c.y10y - c.y2y) * 10_000.0;
    let spread_10_3m = (c.y10y - c.y3m) * 10_000.0;
    let is_humped = c.y5y > c.y3m && c.y5y > c.y30y && c.y2y < c.y5y && c.y10y < c.y5y;
    let spreads = [c.y2y - c.y3m, c.y5y - c.y2y, c.y10y - c.y5y, c.y30y - c.y10y];
    let shape = if spread_10_2 < 0.0 {
        CurveShape::Inverted
    } else if is_humped {
        CurveShape::Humped
    } else if spreads.iter().all(|s| s.abs() < 0.0025) {
        CurveShape::Flat
    } else if spreads.iter().all(|s| *s >= -0.0001) {
        // Allow tiny noise but require predominantly non-decreasing.
        CurveShape::Normal
    } else {
        // Mixed / non-monotonic but not classically humped → flag flat.
        CurveShape::Flat
    };
    let note = match shape {
        CurveShape::Normal   => "monotonic upward slope — healthy expansion".into(),
        CurveShape::Flat     => "tenors compressed — late-cycle uncertainty".into(),
        CurveShape::Inverted => format!("2Y/10Y inverted by {:.0} bps — recession signal", -spread_10_2),
        CurveShape::Humped   => "5Y peak with both ends lower — rare transitional shape".into(),
    };
    CurveReport {
        shape,
        spread_10y_2y_bps: spread_10_2,
        spread_10y_3m_bps: spread_10_3m,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normal_curve() -> YieldCurve {
        YieldCurve { y3m: 0.03, y2y: 0.035, y5y: 0.04, y10y: 0.045, y30y: 0.05 }
    }

    #[test]
    fn normal_curve_classified_normal() {
        let r = classify(&normal_curve());
        assert_eq!(r.shape, CurveShape::Normal);
        assert!(r.spread_10y_2y_bps > 0.0);
    }

    #[test]
    fn inverted_2y_above_10y_classified_inverted() {
        let c = YieldCurve { y3m: 0.04, y2y: 0.055, y5y: 0.05, y10y: 0.045, y30y: 0.045 };
        let r = classify(&c);
        assert_eq!(r.shape, CurveShape::Inverted);
        assert!(r.spread_10y_2y_bps < 0.0);
    }

    #[test]
    fn flat_curve_all_yields_close_classified_flat() {
        let c = YieldCurve { y3m: 0.04, y2y: 0.041, y5y: 0.042, y10y: 0.0425, y30y: 0.043 };
        let r = classify(&c);
        assert_eq!(r.shape, CurveShape::Flat);
    }

    #[test]
    fn humped_curve_5y_peak_both_ends_lower() {
        let c = YieldCurve { y3m: 0.03, y2y: 0.04, y5y: 0.06, y10y: 0.045, y30y: 0.03 };
        let r = classify(&c);
        assert_eq!(r.shape, CurveShape::Humped);
    }

    #[test]
    fn spread_10y_2y_in_bps() {
        // 4.5% - 3.5% = 1% = 100 bps.
        let r = classify(&normal_curve());
        assert!((r.spread_10y_2y_bps - 100.0).abs() < 1e-9);
    }

    #[test]
    fn spread_10y_3m_in_bps() {
        let r = classify(&normal_curve());
        // 4.5% - 3% = 1.5% = 150 bps.
        assert!((r.spread_10y_3m_bps - 150.0).abs() < 1e-9);
    }

    #[test]
    fn inverted_note_quotes_magnitude() {
        let c = YieldCurve { y3m: 0.04, y2y: 0.055, y5y: 0.05, y10y: 0.045, y30y: 0.045 };
        let r = classify(&c);
        // 5.5% - 4.5% = 1% = 100 bps. Note: "by 100 bps".
        assert!(r.note.contains("100"));
    }

    #[test]
    fn slightly_falling_30y_after_normal_still_normal_with_tiny_noise() {
        // Allow tiny noise tolerance in normal classification.
        let c = YieldCurve { y3m: 0.03, y2y: 0.035, y5y: 0.04, y10y: 0.045, y30y: 0.0449 };
        let r = classify(&c);
        assert_eq!(r.shape, CurveShape::Normal);
    }
}
