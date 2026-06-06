//! IV Term-Structure Slope Analyzer.
//!
//! Given the ATM implied volatility for a sequence of expiries on the
//! same underlying, characterizes:
//!
//!   - **Slope** = β_1 in OLS regression of IV on time-to-expiry T
//!     (negative β_1 = front-month IV > back-month IV = backwardation,
//!     typical pre-event)
//!   - **Contango / Backwardation**: classification by slope sign.
//!   - **Front-back spread** = IV(shortest) − IV(longest)
//!   - **Convexity** = β_2 in IV ~ β_0 + β_1 · T + β_2 · T²
//!     (positive = U-shaped term structure)
//!
//! Used to detect:
//!   - Event-driven term-structure inversions (earnings, FDA, macro)
//!   - Vol-of-vol regime shifts
//!   - Calendar spread / diagonal opportunities
//!
//! Pure compute. Companion to `iv_skew_scanner`, `iv_rank_scanner`,
//! `iv_solver`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExpiryIv {
    pub time_to_expiry_years: f64,
    pub atm_iv: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TermShape {
    Contango,
    Backwardation,
    #[default]
    Flat,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IvTermStructureReport {
    pub slope: f64,
    pub intercept: f64,
    pub convexity_beta_2: Option<f64>,
    pub front_back_spread: f64,
    pub shape: TermShape,
    pub front_iv: f64,
    pub back_iv: f64,
    pub n_expiries: usize,
}

pub fn compute(expiries: &[ExpiryIv]) -> Option<IvTermStructureReport> {
    if expiries.len() < 2 {
        return None;
    }
    if expiries.iter().any(|e| {
        !e.time_to_expiry_years.is_finite()
            || !e.atm_iv.is_finite()
            || e.time_to_expiry_years <= 0.0
            || e.atm_iv <= 0.0
    }) {
        return None;
    }
    // Sort by tenor ascending.
    let mut sorted = expiries.to_vec();
    sorted.sort_by(|a, b| {
        a.time_to_expiry_years
            .partial_cmp(&b.time_to_expiry_years)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let front = sorted.first().unwrap();
    let back = sorted.last().unwrap();
    // Linear OLS of IV on T.
    let n = sorted.len() as f64;
    let t_mean: f64 = sorted.iter().map(|e| e.time_to_expiry_years).sum::<f64>() / n;
    let iv_mean: f64 = sorted.iter().map(|e| e.atm_iv).sum::<f64>() / n;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for e in &sorted {
        let dx = e.time_to_expiry_years - t_mean;
        sxx += dx * dx;
        sxy += dx * (e.atm_iv - iv_mean);
    }
    if sxx <= 0.0 {
        return None;
    }
    let slope = sxy / sxx;
    let intercept = iv_mean - slope * t_mean;
    // Quadratic convexity if ≥ 3 expiries.
    let convexity = if sorted.len() >= 3 {
        ols_quadratic(&sorted).map(|c| c.beta_2)
    } else {
        None
    };
    let front_back_spread = front.atm_iv - back.atm_iv;
    let shape = if slope > 0.001 {
        TermShape::Contango
    } else if slope < -0.001 {
        TermShape::Backwardation
    } else {
        TermShape::Flat
    };
    Some(IvTermStructureReport {
        slope,
        intercept,
        convexity_beta_2: convexity,
        front_back_spread,
        shape,
        front_iv: front.atm_iv,
        back_iv: back.atm_iv,
        n_expiries: sorted.len(),
    })
}

struct Quadratic {
    #[allow(dead_code)]
    beta_0: f64,
    #[allow(dead_code)]
    beta_1: f64,
    beta_2: f64,
}

fn ols_quadratic(points: &[ExpiryIv]) -> Option<Quadratic> {
    let n = points.len();
    if n < 3 {
        return None;
    }
    let mut xtx = [[0.0_f64; 3]; 3];
    let mut xty = [0.0_f64; 3];
    for e in points {
        let row = [1.0, e.time_to_expiry_years, e.time_to_expiry_years.powi(2)];
        let y = e.atm_iv;
        for j in 0..3 {
            xty[j] += row[j] * y;
            for k in 0..3 {
                xtx[j][k] += row[j] * row[k];
            }
        }
    }
    let det = xtx[0][0] * (xtx[1][1] * xtx[2][2] - xtx[1][2] * xtx[2][1])
        - xtx[0][1] * (xtx[1][0] * xtx[2][2] - xtx[1][2] * xtx[2][0])
        + xtx[0][2] * (xtx[1][0] * xtx[2][1] - xtx[1][1] * xtx[2][0]);
    if det.abs() < 1e-18 {
        return None;
    }
    let inv00 = (xtx[1][1] * xtx[2][2] - xtx[1][2] * xtx[2][1]) / det;
    let inv01 = -(xtx[0][1] * xtx[2][2] - xtx[0][2] * xtx[2][1]) / det;
    let inv02 = (xtx[0][1] * xtx[1][2] - xtx[0][2] * xtx[1][1]) / det;
    let inv10 = -(xtx[1][0] * xtx[2][2] - xtx[1][2] * xtx[2][0]) / det;
    let inv11 = (xtx[0][0] * xtx[2][2] - xtx[0][2] * xtx[2][0]) / det;
    let inv12 = -(xtx[0][0] * xtx[1][2] - xtx[0][2] * xtx[1][0]) / det;
    let inv20 = (xtx[1][0] * xtx[2][1] - xtx[1][1] * xtx[2][0]) / det;
    let inv21 = -(xtx[0][0] * xtx[2][1] - xtx[0][1] * xtx[2][0]) / det;
    let inv22 = (xtx[0][0] * xtx[1][1] - xtx[0][1] * xtx[1][0]) / det;
    let beta_0 = inv00 * xty[0] + inv01 * xty[1] + inv02 * xty[2];
    let beta_1 = inv10 * xty[0] + inv11 * xty[1] + inv12 * xty[2];
    let beta_2 = inv20 * xty[0] + inv21 * xty[1] + inv22 * xty[2];
    Some(Quadratic {
        beta_0,
        beta_1,
        beta_2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(t: f64, iv: f64) -> ExpiryIv {
        ExpiryIv {
            time_to_expiry_years: t,
            atm_iv: iv,
        }
    }

    #[test]
    fn too_few_expiries_returns_none() {
        assert!(compute(&[e(0.1, 0.2)]).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[e(f64::NAN, 0.2), e(0.5, 0.18)]).is_none());
        assert!(compute(&[e(0.1, 0.0), e(0.5, 0.18)]).is_none());
        assert!(compute(&[e(0.0, 0.20), e(0.5, 0.18)]).is_none());
    }

    #[test]
    fn contango_detected() {
        let expiries = vec![
            e(1.0 / 12.0, 0.18), // 1m: 18%
            e(3.0 / 12.0, 0.20), // 3m: 20%
            e(6.0 / 12.0, 0.22), // 6m: 22%
        ];
        let r = compute(&expiries).unwrap();
        assert_eq!(r.shape, TermShape::Contango);
        assert!(r.slope > 0.0);
        assert!(r.front_back_spread < 0.0);
    }

    #[test]
    fn backwardation_detected() {
        let expiries = vec![
            e(1.0 / 12.0, 0.40), // 1m: 40% (event-rich)
            e(3.0 / 12.0, 0.30), // 3m: 30%
            e(6.0 / 12.0, 0.25), // 6m: 25%
        ];
        let r = compute(&expiries).unwrap();
        assert_eq!(r.shape, TermShape::Backwardation);
        assert!(r.slope < 0.0);
        assert!(r.front_back_spread > 0.0);
    }

    #[test]
    fn flat_term_structure_classified_flat() {
        let expiries = vec![
            e(1.0 / 12.0, 0.20),
            e(3.0 / 12.0, 0.20),
            e(6.0 / 12.0, 0.20),
        ];
        let r = compute(&expiries).unwrap();
        assert_eq!(r.shape, TermShape::Flat);
        assert!(r.slope.abs() < 1e-9);
    }

    #[test]
    fn convexity_present_for_smile_term() {
        // U-shaped: front and back high, middle low.
        let expiries = vec![
            e(1.0 / 12.0, 0.30),
            e(3.0 / 12.0, 0.20),
            e(6.0 / 12.0, 0.25),
            e(12.0 / 12.0, 0.32),
        ];
        let r = compute(&expiries).unwrap();
        assert!(
            r.convexity_beta_2.unwrap() > 0.0,
            "smile term should have positive convexity, got {:?}",
            r.convexity_beta_2
        );
    }

    #[test]
    fn convexity_none_with_only_two_expiries() {
        let expiries = vec![e(1.0 / 12.0, 0.18), e(6.0 / 12.0, 0.22)];
        let r = compute(&expiries).unwrap();
        assert!(r.convexity_beta_2.is_none());
    }

    #[test]
    fn n_expiries_reported() {
        let expiries = vec![e(0.08, 0.20), e(0.25, 0.22), e(0.5, 0.24), e(1.0, 0.26)];
        let r = compute(&expiries).unwrap();
        assert_eq!(r.n_expiries, 4);
    }
}
