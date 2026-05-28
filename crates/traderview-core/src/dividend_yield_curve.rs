//! Dividend Yield Curve — implied annualized dividend yield by maturity
//! from dividend-swap or dividend-future prices.
//!
//! Caller supplies (tenor_years, dividend_amount, underlying_spot)
//! tuples. Computes:
//!
//!   yield_t = dividend_amount / (underlying_spot · tenor_years) · 100
//!
//! Returns the term-structure sorted by tenor plus slope (long minus
//! short yield) and concavity (mid - linear-interpolation of endpoints).
//!
//! Pure compute. Companion to `yield_curve_bootstrap`,
//! `nelson_siegel`, `nelson_siegel_svensson`, `breakeven_inflation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DividendObservation {
    pub tenor_years: f64,
    pub dividend_amount: f64,
    pub underlying_spot: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct YieldPoint {
    pub tenor_years: f64,
    pub yield_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendYieldCurveReport {
    pub curve: Vec<YieldPoint>,
    pub slope: Option<f64>,
    pub curvature: Option<f64>,
}

pub fn compute(observations: &[DividendObservation]) -> Option<DividendYieldCurveReport> {
    if observations.is_empty() { return None; }
    if observations.iter().any(|o| !o.tenor_years.is_finite() || o.tenor_years <= 0.0
        || !o.dividend_amount.is_finite() || o.dividend_amount < 0.0
        || !o.underlying_spot.is_finite() || o.underlying_spot <= 0.0) {
        return None;
    }
    let mut curve: Vec<YieldPoint> = observations.iter().map(|o| {
        YieldPoint {
            tenor_years: o.tenor_years,
            yield_pct: o.dividend_amount / (o.underlying_spot * o.tenor_years) * 100.0,
        }
    }).collect();
    curve.sort_by(|a, b| a.tenor_years.partial_cmp(&b.tenor_years).unwrap_or(std::cmp::Ordering::Equal));
    let slope = if curve.len() >= 2 {
        Some(curve.last().unwrap().yield_pct - curve[0].yield_pct)
    } else { None };
    let curvature = if curve.len() >= 3 {
        let lo = curve[0];
        let hi = curve.last().unwrap();
        let mid_idx = curve.len() / 2;
        let mid = curve[mid_idx];
        let t = (mid.tenor_years - lo.tenor_years) / (hi.tenor_years - lo.tenor_years);
        let linear_interp = lo.yield_pct + t * (hi.yield_pct - lo.yield_pct);
        Some(mid.yield_pct - linear_interp)
    } else { None };
    Some(DividendYieldCurveReport { curve, slope, curvature })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(tenor: f64, div: f64, spot: f64) -> DividendObservation {
        DividendObservation { tenor_years: tenor, dividend_amount: div, underlying_spot: spot }
    }

    #[test]
    fn empty_returns_none() { assert!(compute(&[]).is_none()); }

    #[test]
    fn invalid_observation_returns_none() {
        assert!(compute(&[d(0.0, 1.0, 100.0)]).is_none());
        assert!(compute(&[d(1.0, f64::NAN, 100.0)]).is_none());
        assert!(compute(&[d(1.0, 1.0, 0.0)]).is_none());
    }

    #[test]
    fn single_point_yields_no_slope_or_curvature() {
        let r = compute(&[d(1.0, 2.0, 100.0)]).unwrap();
        assert_eq!(r.curve.len(), 1);
        assert!((r.curve[0].yield_pct - 2.0).abs() < 1e-9);
        assert!(r.slope.is_none());
        assert!(r.curvature.is_none());
    }

    #[test]
    fn curve_sorted_by_tenor() {
        let obs = vec![
            d(5.0, 10.0, 100.0),
            d(1.0, 2.0, 100.0),
            d(3.0, 6.0, 100.0),
        ];
        let r = compute(&obs).unwrap();
        for w in r.curve.windows(2) {
            assert!(w[0].tenor_years <= w[1].tenor_years);
        }
    }

    #[test]
    fn upward_sloping_curve_yields_positive_slope() {
        let obs = vec![
            d(1.0, 1.0, 100.0),    // 1% yield
            d(5.0, 15.0, 100.0),   // 3% yield
        ];
        let r = compute(&obs).unwrap();
        assert!(r.slope.unwrap() > 0.0);
    }

    #[test]
    fn yield_pct_computed_correctly() {
        let r = compute(&[d(2.0, 4.0, 100.0)]).unwrap();
        // yield = 4 / (100 · 2) · 100 = 2.0%.
        assert!((r.curve[0].yield_pct - 2.0).abs() < 1e-9);
    }
}
