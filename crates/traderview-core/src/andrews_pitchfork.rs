//! Andrews' Pitchfork — three-pivot trend channel.
//!
//! Given three pivots P1, P2, P3 (typically a major pivot followed by
//! an opposing pivot pair forming the "handle width"):
//!
//!   median_line slope     = (mid(P2, P3) − P1) / Δx
//!   parallel upper slope  = same slope, intercept passed through P2
//!   parallel lower slope  = same slope, intercept passed through P3
//!
//! At any future bar x, returns (lower_band, median, upper_band) by
//! linear extrapolation from the chosen anchor (P1).
//!
//! Pure compute. Used in technical analysis to draw the canonical
//! pitchfork — trades fade extensions back to the median line.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pivot {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PitchforkLines {
    pub median_at_x: f64,
    pub upper_at_x: f64,
    pub lower_at_x: f64,
}

pub fn fit(p1: Pivot, p2: Pivot, p3: Pivot) -> Option<(f64, f64, f64, f64)> {
    // Returns (slope, median_intercept, upper_intercept, lower_intercept)
    // where y = slope · x + intercept.
    if [p1.x, p1.y, p2.x, p2.y, p3.x, p3.y]
        .iter()
        .any(|v| !v.is_finite())
    {
        return None;
    }
    let mid_x = (p2.x + p3.x) / 2.0;
    let mid_y = (p2.y + p3.y) / 2.0;
    let dx = mid_x - p1.x;
    if dx == 0.0 {
        return None;
    }
    let slope = (mid_y - p1.y) / dx;
    let median_intercept = p1.y - slope * p1.x;
    let upper_intercept = p2.y - slope * p2.x;
    let lower_intercept = p3.y - slope * p3.x;
    Some((slope, median_intercept, upper_intercept, lower_intercept))
}

pub fn project(p1: Pivot, p2: Pivot, p3: Pivot, x: f64) -> Option<PitchforkLines> {
    if !x.is_finite() {
        return None;
    }
    let (slope, m, u, l) = fit(p1, p2, p3)?;
    // Caller decides which of u/l is the "upper" — we report them as the
    // value that's larger at x as upper, smaller as lower.
    let median = slope * x + m;
    let u_at_x = slope * x + u;
    let l_at_x = slope * x + l;
    let (upper, lower) = if u_at_x >= l_at_x {
        (u_at_x, l_at_x)
    } else {
        (l_at_x, u_at_x)
    };
    Some(PitchforkLines {
        median_at_x: median,
        upper_at_x: upper,
        lower_at_x: lower,
    })
}

/// Project the pitchfork over a vector of x values.
pub fn series(p1: Pivot, p2: Pivot, p3: Pivot, xs: &[f64]) -> Vec<Option<PitchforkLines>> {
    xs.iter().map(|x| project(p1, p2, p3, *x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Pivot {
        Pivot { x, y }
    }

    #[test]
    fn collinear_p1_to_midpoint_undefined() {
        let p1 = p(0.0, 0.0);
        let p2 = p(0.0, 1.0); // mid_x = 0 → dx = 0
        let p3 = p(0.0, -1.0);
        assert!(project(p1, p2, p3, 5.0).is_none());
    }

    #[test]
    fn nan_pivots_return_none() {
        let p1 = p(0.0, f64::NAN);
        let p2 = p(1.0, 1.0);
        let p3 = p(1.0, -1.0);
        assert!(project(p1, p2, p3, 5.0).is_none());
    }

    #[test]
    fn classic_pitchfork_at_pivots_satisfies_construction() {
        // P1 at (0, 0), P2 at (4, 10), P3 at (4, 0).
        // mid(P2, P3) = (4, 5). Median: y = 5/4 · x.
        // Upper parallel through P2: y = 5/4 · x + (10 − 5) = 5/4 · x + 5.
        // Lower parallel through P3: y = 5/4 · x + (0 − 5) = 5/4 · x − 5.
        let p1 = p(0.0, 0.0);
        let p2 = p(4.0, 10.0);
        let p3 = p(4.0, 0.0);
        let lines = project(p1, p2, p3, 8.0).expect("populated");
        // At x=8: median = 5/4 · 8 = 10. upper = 10 + 5 = 15. lower = 10 − 5 = 5.
        assert!((lines.median_at_x - 10.0).abs() < 1e-9);
        assert!((lines.upper_at_x - 15.0).abs() < 1e-9);
        assert!((lines.lower_at_x - 5.0).abs() < 1e-9);
    }

    #[test]
    fn median_passes_through_p1_and_midpoint() {
        let p1 = p(0.0, 100.0);
        let p2 = p(10.0, 120.0);
        let p3 = p(10.0, 80.0);
        let at_p1 = project(p1, p2, p3, p1.x).unwrap();
        assert!((at_p1.median_at_x - p1.y).abs() < 1e-9);
        let at_mid = project(p1, p2, p3, (p2.x + p3.x) / 2.0).unwrap();
        let expected_mid_y = (p2.y + p3.y) / 2.0;
        assert!((at_mid.median_at_x - expected_mid_y).abs() < 1e-9);
    }

    #[test]
    fn parallel_lines_separated_by_constant_distance() {
        let p1 = p(0.0, 0.0);
        let p2 = p(4.0, 10.0);
        let p3 = p(4.0, 0.0);
        let xs = vec![1.0, 5.0, 10.0, 20.0, 50.0];
        let series = series(p1, p2, p3, &xs);
        for s in series.iter().flatten() {
            // distance to median should be 5 above and below at all x.
            assert!((s.upper_at_x - s.median_at_x - 5.0).abs() < 1e-9);
            assert!((s.median_at_x - s.lower_at_x - 5.0).abs() < 1e-9);
        }
    }

    #[test]
    fn nan_x_in_series_returns_none_entry() {
        let p1 = p(0.0, 0.0);
        let p2 = p(4.0, 10.0);
        let p3 = p(4.0, 0.0);
        let xs = vec![1.0, f64::NAN, 5.0];
        let s = series(p1, p2, p3, &xs);
        assert!(s[1].is_none());
        assert!(s[0].is_some() && s[2].is_some());
    }
}
