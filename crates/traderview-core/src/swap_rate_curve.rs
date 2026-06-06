//! Swap Rate Curve — fixed-leg par-swap rates by tenor + slope +
//! curvature summary statistics.
//!
//! Caller supplies (tenor_years, swap_rate_pct) pairs. Returns the
//! sorted curve plus:
//!   - slope (long minus short tenor rate)
//!   - butterfly (mid - average of endpoints) — concavity measure
//!   - 2s10s, 5s30s common spread shortcuts (when available)
//!
//! Pure compute. Companion to `swap_valuation`, `yield_curve_bootstrap`,
//! `nelson_siegel_svensson`, `cross_currency_basis`,
//! `term_spread`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SwapPoint {
    pub tenor_years: f64,
    pub rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwapRateCurveReport {
    pub curve: Vec<SwapPoint>,
    pub slope_bps: Option<f64>,
    pub butterfly_bps: Option<f64>,
    pub spread_2s10s_bps: Option<f64>,
    pub spread_5s30s_bps: Option<f64>,
}

pub fn compute(points: &[SwapPoint]) -> Option<SwapRateCurveReport> {
    if points.is_empty() {
        return None;
    }
    if points
        .iter()
        .any(|p| !p.tenor_years.is_finite() || p.tenor_years <= 0.0 || !p.rate_pct.is_finite())
    {
        return None;
    }
    let mut curve: Vec<SwapPoint> = points.to_vec();
    curve.sort_by(|a, b| {
        a.tenor_years
            .partial_cmp(&b.tenor_years)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let slope = if curve.len() >= 2 {
        Some((curve.last().unwrap().rate_pct - curve[0].rate_pct) * 100.0)
    } else {
        None
    };
    let butterfly = if curve.len() >= 3 {
        let mid = curve[curve.len() / 2];
        let avg_ends = (curve[0].rate_pct + curve.last().unwrap().rate_pct) / 2.0;
        Some((mid.rate_pct - avg_ends) * 100.0)
    } else {
        None
    };
    let find_rate = |target: f64| -> Option<f64> {
        curve
            .iter()
            .find(|p| (p.tenor_years - target).abs() < 0.01)
            .map(|p| p.rate_pct)
    };
    let spread_2s10s = match (find_rate(2.0), find_rate(10.0)) {
        (Some(s2), Some(s10)) => Some((s10 - s2) * 100.0),
        _ => None,
    };
    let spread_5s30s = match (find_rate(5.0), find_rate(30.0)) {
        (Some(s5), Some(s30)) => Some((s30 - s5) * 100.0),
        _ => None,
    };
    Some(SwapRateCurveReport {
        curve,
        slope_bps: slope,
        butterfly_bps: butterfly,
        spread_2s10s_bps: spread_2s10s,
        spread_5s30s_bps: spread_5s30s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(tenor: f64, rate: f64) -> SwapPoint {
        SwapPoint {
            tenor_years: tenor,
            rate_pct: rate,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
    }

    #[test]
    fn invalid_returns_none() {
        assert!(compute(&[p(0.0, 4.0)]).is_none());
        assert!(compute(&[p(1.0, f64::NAN)]).is_none());
    }

    #[test]
    fn curve_sorted_by_tenor() {
        let pts = vec![p(10.0, 4.5), p(2.0, 4.0), p(5.0, 4.25)];
        let r = compute(&pts).unwrap();
        for w in r.curve.windows(2) {
            assert!(w[0].tenor_years <= w[1].tenor_years);
        }
    }

    #[test]
    fn upward_curve_yields_positive_slope() {
        let pts = vec![p(1.0, 3.0), p(10.0, 5.0)];
        let r = compute(&pts).unwrap();
        assert!((r.slope_bps.unwrap() - 200.0).abs() < 1e-6);
    }

    #[test]
    fn humped_curve_yields_positive_butterfly() {
        // Mid above average of endpoints → positive butterfly.
        let pts = vec![p(1.0, 3.0), p(5.0, 5.0), p(10.0, 4.0)];
        let r = compute(&pts).unwrap();
        // Avg ends = 3.5, mid = 5.0 → butterfly = 1.5pp = 150 bps.
        assert!((r.butterfly_bps.unwrap() - 150.0).abs() < 1e-6);
    }

    #[test]
    fn standard_spreads_computed_when_available() {
        let pts = vec![p(2.0, 3.5), p(5.0, 4.0), p(10.0, 4.5), p(30.0, 5.0)];
        let r = compute(&pts).unwrap();
        assert!((r.spread_2s10s_bps.unwrap() - 100.0).abs() < 1e-6);
        assert!((r.spread_5s30s_bps.unwrap() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn missing_tenors_yield_none_spreads() {
        let pts = vec![p(1.0, 3.0), p(7.0, 4.0)];
        let r = compute(&pts).unwrap();
        assert!(r.spread_2s10s_bps.is_none());
        assert!(r.spread_5s30s_bps.is_none());
    }
}
