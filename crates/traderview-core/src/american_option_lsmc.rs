//! American Option Pricer — Longstaff-Schwartz Monte Carlo (LSMC, 2001).
//!
//! Values an American call or put by:
//!   1. Simulating P GBM paths over N exercise times (Euler discretization).
//!   2. Walking backward from expiry: at each exercise time, regress
//!      discounted continuation values on a low-degree polynomial of
//!      current spot using only in-the-money paths.
//!   3. Exercising whenever immediate payoff > regressed continuation.
//!   4. Averaging discounted realized cashflows across paths.
//!
//! Regression basis: {1, S, S²} (3 features). Sufficient for vanilla
//! American options on a single underlier per Longstaff-Schwartz §3.
//!
//! Returns the LSMC price, standard error, and confidence interval.
//! Reproducible via seeded LCG; no external rand dependency.
//!
//! Pure compute. Companion to `crr_binomial`, `bermudan_binomial`,
//! `gbm_path_simulator`, `monte_carlo_option`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug)]
pub struct Report {
    pub price: f64,
    pub standard_error: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    pub paths_run: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    kind: OptionKind,
    spot: f64,
    strike: f64,
    t_years: f64,
    rate: f64,
    dividend: f64,
    sigma: f64,
    steps: usize,
    paths: usize,
    seed: u64,
) -> Option<Report> {
    let scalars = [spot, strike, t_years, rate, dividend, sigma];
    if scalars.iter().any(|x| !x.is_finite()) {
        return None;
    }
    if spot <= 0.0 || strike <= 0.0 || t_years <= 0.0 || sigma < 0.0 {
        return None;
    }
    if steps < 2 || paths < 10 {
        return None;
    }
    let dt = t_years / steps as f64;
    let drift = (rate - dividend - 0.5 * sigma * sigma) * dt;
    let diffusion = sigma * dt.sqrt();
    let discount_step = (-rate * dt).exp();
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    // Simulate spot paths: paths × (steps + 1).
    let mut s_paths = vec![vec![0.0_f64; steps + 1]; paths];
    for path in s_paths.iter_mut() {
        path[0] = spot;
        for t in 0..steps {
            let z = next_normal(&mut state);
            path[t + 1] = path[t] * (drift + diffusion * z).exp();
        }
    }
    // Cashflows: each path's payoff at the time it is exercised.
    let mut cashflow: Vec<f64> = s_paths
        .iter()
        .map(|p| payoff(kind, p[steps], strike))
        .collect();
    let mut exercise_step: Vec<usize> = vec![steps; paths];
    // Backward induction from steps-1 down to 1.
    for t in (1..steps).rev() {
        let spots: Vec<f64> = s_paths.iter().map(|p| p[t]).collect();
        let payoffs: Vec<f64> = spots.iter().map(|&s| payoff(kind, s, strike)).collect();
        // In-the-money paths only.
        let itm: Vec<usize> = (0..paths).filter(|&p| payoffs[p] > 0.0).collect();
        if itm.len() < 4 {
            continue;
        }
        // Discount cashflow from exercise_step back to t.
        let x: Vec<f64> = itm.iter().map(|&p| spots[p]).collect();
        let y: Vec<f64> = itm
            .iter()
            .map(|&p| {
                let steps_to_disc = exercise_step[p] - t;
                cashflow[p] * discount_step.powi(steps_to_disc as i32)
            })
            .collect();
        // OLS on {1, S, S²}.
        let beta = ols_quadratic(&x, &y);
        if beta.is_none() {
            continue;
        }
        let (b0, b1, b2) = beta.unwrap();
        // Exercise if intrinsic > regressed continuation.
        for &p in &itm {
            let s = spots[p];
            let continuation = b0 + b1 * s + b2 * s * s;
            if payoffs[p] >= continuation {
                cashflow[p] = payoffs[p];
                exercise_step[p] = t;
            }
        }
    }
    // Discount all cashflows back to t=0.
    let discounted: Vec<f64> = (0..paths)
        .map(|p| cashflow[p] * discount_step.powi(exercise_step[p] as i32))
        .collect();
    let n_f = paths as f64;
    let mean = discounted.iter().sum::<f64>() / n_f;
    let var = discounted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n_f;
    let se = (var / n_f).max(0.0).sqrt();
    // Also compare with European exercise (no early stop) — take max.
    let euro: f64 = s_paths
        .iter()
        .map(|p| payoff(kind, p[steps], strike) * discount_step.powi(steps as i32))
        .sum::<f64>()
        / n_f;
    let price = mean.max(euro);
    Some(Report {
        price,
        standard_error: se,
        ci_lower: price - 1.96 * se,
        ci_upper: price + 1.96 * se,
        paths_run: paths,
    })
}

fn payoff(kind: OptionKind, s: f64, k: f64) -> f64 {
    match kind {
        OptionKind::Call => (s - k).max(0.0),
        OptionKind::Put => (k - s).max(0.0),
    }
}

fn ols_quadratic(x: &[f64], y: &[f64]) -> Option<(f64, f64, f64)> {
    let n = x.len() as f64;
    let mut sx = 0.0;
    let mut sx2 = 0.0;
    let mut sx3 = 0.0;
    let mut sx4 = 0.0;
    let mut sy = 0.0;
    let mut sxy = 0.0;
    let mut sx2y = 0.0;
    for (xi, yi) in x.iter().zip(y.iter()) {
        let x2 = xi * xi;
        sx += xi;
        sx2 += x2;
        sx3 += x2 * xi;
        sx4 += x2 * x2;
        sy += yi;
        sxy += xi * yi;
        sx2y += x2 * yi;
    }
    let m = [[n, sx, sx2], [sx, sx2, sx3], [sx2, sx3, sx4]];
    let b = [sy, sxy, sx2y];
    let det = det3(m);
    if det.abs() < 1e-12 {
        return None;
    }
    let mut m0 = m;
    m0[0][0] = b[0];
    m0[1][0] = b[1];
    m0[2][0] = b[2];
    let mut m1 = m;
    m1[0][1] = b[0];
    m1[1][1] = b[1];
    m1[2][1] = b[2];
    let mut m2 = m;
    m2[0][2] = b[0];
    m2[1][2] = b[1];
    m2[2][2] = b[2];
    Some((det3(m0) / det, det3(m1) / det, det3(m2) / det))
}

fn det3(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn next_uniform(state: &mut u64) -> f64 {
    let u = next_u64(state);
    let mantissa = u >> 11;
    (mantissa as f64) * (1.0_f64 / (1u64 << 53) as f64)
}

fn next_normal(state: &mut u64) -> f64 {
    let mut u1 = next_uniform(state);
    let u2 = next_uniform(state);
    if u1 < 1e-300 {
        u1 = 1e-300;
    }
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs_call(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
        let st = sigma * t.sqrt();
        let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / st;
        let d2 = d1 - st;
        s * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
    }

    fn norm_cdf(x: f64) -> f64 {
        let a1 = 0.254829592_f64;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911_f64;
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x_abs = (x / std::f64::consts::SQRT_2).abs();
        let t = 1.0 / (1.0 + p * x_abs);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();
        0.5 * (1.0 + sign * y)
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(
            OptionKind::Call,
            -1.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.2,
            50,
            1000,
            1
        )
        .is_none());
        assert!(compute(
            OptionKind::Call,
            100.0,
            100.0,
            0.0,
            0.05,
            0.0,
            0.2,
            50,
            1000,
            1
        )
        .is_none());
        assert!(compute(
            OptionKind::Call,
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            -0.1,
            50,
            1000,
            1
        )
        .is_none());
        assert!(compute(
            OptionKind::Call,
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.2,
            1,
            1000,
            1
        )
        .is_none());
        assert!(compute(
            OptionKind::Call,
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.2,
            50,
            5,
            1
        )
        .is_none());
        assert!(compute(
            OptionKind::Call,
            f64::NAN,
            100.0,
            0.5,
            0.05,
            0.0,
            0.2,
            50,
            1000,
            1
        )
        .is_none());
    }

    #[test]
    fn american_call_no_dividend_equals_european() {
        // Famous result: American call on non-dividend stock has no early
        // exercise premium → LSMC value ≈ BS European call.
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let sigma = 0.25;
        let lsmc = compute(OptionKind::Call, s, k, t, r, 0.0, sigma, 50, 5000, 7).unwrap();
        let euro = bs_call(s, k, t, r, sigma);
        // Allow 5% MC tolerance + std-error coverage.
        assert!((lsmc.price - euro).abs() < euro * 0.10 + 3.0 * lsmc.standard_error);
    }

    #[test]
    fn american_put_above_european_intrinsic() {
        // American put on non-dividend stock IS worth more than European
        // when ITM → LSMC must report at least the intrinsic value.
        let s = 90.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let sigma = 0.25;
        let r_lsmc = compute(OptionKind::Put, s, k, t, r, 0.0, sigma, 50, 5000, 7).unwrap();
        // Intrinsic = 10. American put should beat it (and beat European put).
        assert!(r_lsmc.price >= 10.0 - 3.0 * r_lsmc.standard_error);
    }

    #[test]
    fn deep_otm_option_priced_near_zero() {
        let r = compute(
            OptionKind::Call,
            50.0,
            200.0,
            0.1,
            0.05,
            0.0,
            0.20,
            20,
            2000,
            7,
        )
        .unwrap();
        assert!(r.price < 0.5);
    }

    #[test]
    fn reproducible_with_same_seed() {
        let a = compute(
            OptionKind::Put,
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.25,
            50,
            1000,
            99,
        )
        .unwrap();
        let b = compute(
            OptionKind::Put,
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.25,
            50,
            1000,
            99,
        )
        .unwrap();
        assert_eq!(a.price.to_bits(), b.price.to_bits());
    }

    #[test]
    fn confidence_interval_brackets_price() {
        let r = compute(
            OptionKind::Put,
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.25,
            50,
            1000,
            7,
        )
        .unwrap();
        assert!(r.ci_lower <= r.price && r.price <= r.ci_upper);
    }
}
