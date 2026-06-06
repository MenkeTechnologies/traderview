//! Greeks Profile — vanilla Black-Scholes price + greeks across a
//! range of underlying spots.
//!
//! Generates a sampled curve of (spot, price, delta, gamma, vega,
//! theta, rho) for plotting a "P&L vs spot" or "greek profile vs spot"
//! display. Useful for risk visualization, scenario PnL, and option-
//! strategy payoff diagrams.
//!
//! Pure compute. Distinct from the existing `greeks` (point-estimate)
//! and `second_order_greeks` (vanna/charm/etc.) modules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GreeksPoint {
    pub spot: f64,
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GreeksProfileReport {
    pub points: Vec<GreeksPoint>,
    pub atm_index: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    spot_grid_low: f64,
    spot_grid_high: f64,
    n_points: usize,
    kind: OptionKind,
) -> Option<GreeksProfileReport> {
    if !strike.is_finite()
        || strike <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || !risk_free.is_finite()
        || !dividend_yield.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
        || !spot_grid_low.is_finite()
        || spot_grid_low <= 0.0
        || !spot_grid_high.is_finite()
        || spot_grid_high <= spot_grid_low
        || n_points < 2
    {
        return None;
    }
    let step = (spot_grid_high - spot_grid_low) / (n_points as f64 - 1.0);
    let mut points = Vec::with_capacity(n_points);
    let mut atm_index = 0;
    let mut atm_dist = f64::INFINITY;
    for k in 0..n_points {
        let s = spot_grid_low + step * k as f64;
        let pt = compute_point(
            s,
            strike,
            time_to_expiry,
            risk_free,
            dividend_yield,
            sigma,
            kind,
        );
        let dist = (s - strike).abs();
        if dist < atm_dist {
            atm_dist = dist;
            atm_index = k;
        }
        points.push(pt);
    }
    Some(GreeksProfileReport { points, atm_index })
}

fn compute_point(
    s: f64,
    k: f64,
    t: f64,
    r: f64,
    q: f64,
    sigma: f64,
    kind: OptionKind,
) -> GreeksPoint {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let pdf_d1 = (-0.5 * d1 * d1).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let dq = (-q * t).exp();
    let dr = (-r * t).exp();
    let (price, delta, rho) = match kind {
        OptionKind::Call => (s * dq * nd1 - k * dr * nd2, dq * nd1, k * t * dr * nd2),
        OptionKind::Put => (
            k * dr * (1.0 - nd2) - s * dq * (1.0 - nd1),
            -dq * (1.0 - nd1),
            -k * t * dr * (1.0 - nd2),
        ),
    };
    let gamma = dq * pdf_d1 / (s * sigma * sqrt_t);
    let vega = s * dq * sqrt_t * pdf_d1;
    let theta_term1 = -s * dq * pdf_d1 * sigma / (2.0 * sqrt_t);
    let theta = match kind {
        OptionKind::Call => theta_term1 - r * k * dr * nd2 + q * s * dq * nd1,
        OptionKind::Put => theta_term1 + r * k * dr * (1.0 - nd2) - q * s * dq * (1.0 - nd1),
    };
    GreeksPoint {
        spot: s,
        price,
        delta,
        gamma,
        vega,
        theta,
        rho,
    }
}

fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 = 1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 = 1.061405429_f64;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(compute(bad, 0.5, 0.05, 0.0, 0.2, 80.0, 120.0, 41, OptionKind::Call).is_none());
            assert!(compute(
                100.0,
                bad,
                0.05,
                0.0,
                0.2,
                80.0,
                120.0,
                41,
                OptionKind::Call
            )
            .is_none());
            assert!(compute(
                100.0,
                0.5,
                0.05,
                0.0,
                bad,
                80.0,
                120.0,
                41,
                OptionKind::Call
            )
            .is_none());
        }
        assert!(compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.2,
            120.0,
            80.0,
            41,
            OptionKind::Call
        )
        .is_none());
        assert!(compute(100.0, 0.5, 0.05, 0.0, 0.2, 80.0, 120.0, 1, OptionKind::Call).is_none());
    }

    #[test]
    fn delta_monotonic_increasing_for_call() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            60.0,
            140.0,
            41,
            OptionKind::Call,
        )
        .unwrap();
        let mut prev = -f64::INFINITY;
        for pt in &r.points {
            assert!(pt.delta >= prev - 1e-9);
            prev = pt.delta;
        }
    }

    #[test]
    fn put_delta_negative_and_increasing() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            60.0,
            140.0,
            41,
            OptionKind::Put,
        )
        .unwrap();
        for pt in &r.points {
            assert!(pt.delta <= 0.0 && pt.delta >= -1.0);
        }
        let mut prev = -f64::INFINITY;
        for pt in &r.points {
            assert!(pt.delta >= prev - 1e-9);
            prev = pt.delta;
        }
    }

    #[test]
    fn gamma_peaks_near_atm() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            60.0,
            140.0,
            41,
            OptionKind::Call,
        )
        .unwrap();
        // The bar with maximum gamma should be near (but not exactly at)
        // the ATM strike — peak gamma sits slightly OTM for OTM calls.
        let max_gamma_idx = r
            .points
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.gamma.partial_cmp(&b.1.gamma).unwrap())
            .unwrap()
            .0;
        assert!((max_gamma_idx as isize - r.atm_index as isize).abs() <= 5);
    }

    #[test]
    fn vega_always_positive_for_long_options() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            50.0,
            150.0,
            41,
            OptionKind::Call,
        )
        .unwrap();
        for pt in &r.points {
            assert!(pt.vega >= 0.0);
        }
    }

    #[test]
    fn atm_index_closest_to_strike() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            80.0,
            120.0,
            41,
            OptionKind::Call,
        )
        .unwrap();
        let atm = r.points[r.atm_index];
        for pt in &r.points {
            let dist_other = (pt.spot - 100.0).abs();
            let dist_atm = (atm.spot - 100.0).abs();
            assert!(dist_atm <= dist_other + 1e-9);
        }
    }

    #[test]
    fn grid_step_uniform() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            80.0,
            120.0,
            41,
            OptionKind::Call,
        )
        .unwrap();
        let diffs: Vec<f64> = r.points.windows(2).map(|w| w[1].spot - w[0].spot).collect();
        for d in &diffs[1..] {
            assert!((d - diffs[0]).abs() < 1e-9);
        }
    }

    #[test]
    fn n_points_matches_grid_length() {
        let r = compute(
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            80.0,
            120.0,
            41,
            OptionKind::Call,
        )
        .unwrap();
        assert_eq!(r.points.len(), 41);
    }
}
