//! Belkhayate Timing — Mostafa Belkhayate's polynomial regression timing
//! oscillator (community-published variant).
//!
//! Centers a 4th-degree polynomial fit on a sliding window and reports
//! the standardized deviation of the current close from the fit value
//! (z-score). Belkhayate's heuristics flag overbought/oversold:
//!
//!   |z| > +2 / < -2 → extreme zone (potential reversal)
//!   ±1 → action zones
//!   ≈ 0 → neutral (curve fits price well)
//!
//! For practical use a quadratic fit captures most of the signal at a
//! fraction of the complexity. This module uses quadratic (degree 2)
//! to keep the math tractable while preserving the spirit of the
//! indicator (the curve adapts to local concavity, not just a line).
//!
//!   slope, intercept, curvature = OLS quadratic fit y = a + b·x + c·x²
//!   y_hat at endpoint = a + b·(N-1) + c·(N-1)²
//!   residual_t = close_t - y_hat
//!   stdev = sqrt(SSE / N)
//!   z = residual / stdev
//!
//! Pure compute. Default period = 30. Companion to `bollinger_percent_b`,
//! `linear_regression_channel`, `standard_error_bands`,
//! `z_score_indicator`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 5 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        // Solve y = a + b·x + c·x² via normal equations on a 3×3 system.
        let mut sx = 0.0;
        let mut sx2 = 0.0;
        let mut sx3 = 0.0;
        let mut sx4 = 0.0;
        let mut sy = 0.0;
        let mut sxy = 0.0;
        let mut sx2y = 0.0;
        for (k, &y) in win.iter().enumerate() {
            let x = k as f64;
            let x2 = x * x;
            sx += x;
            sx2 += x2;
            sx3 += x2 * x;
            sx4 += x2 * x2;
            sy += y;
            sxy += x * y;
            sx2y += x2 * y;
        }
        // System:
        //   [N    sx   sx2 ] [a]   [sy   ]
        //   [sx   sx2  sx3 ] [b] = [sxy  ]
        //   [sx2  sx3  sx4 ] [c]   [sx2y ]
        let m = [[p_f, sx, sx2], [sx, sx2, sx3], [sx2, sx3, sx4]];
        let b = [sy, sxy, sx2y];
        let Some(coeffs) = solve_3x3(m, b) else {
            continue;
        };
        let (a, bb, c) = (coeffs[0], coeffs[1], coeffs[2]);
        let x_end = p_f - 1.0;
        let y_hat_end = a + bb * x_end + c * x_end * x_end;
        let mut sse = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            let x = k as f64;
            let y_hat = a + bb * x + c * x * x;
            sse += (y - y_hat).powi(2);
        }
        let stdev = (sse / p_f).max(0.0).sqrt();
        // Float floor: when fit is near-perfect, stdev approaches the
        // precision limit and z amplifies pure rounding error. Treat
        // anything below 1e-9 · |y_hat| as effectively zero.
        let stdev_floor = y_hat_end.abs() * 1e-9;
        if stdev > stdev_floor {
            *slot = Some((closes[i] - y_hat_end) / stdev);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

/// Solve 3×3 linear system via Cramer's rule (returns None for singular).
fn solve_3x3(m: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = det_3x3(m);
    if det.abs() < 1e-12 {
        return None;
    }
    let mut m_a = m;
    m_a[0][0] = b[0];
    m_a[1][0] = b[1];
    m_a[2][0] = b[2];
    let mut m_b = m;
    m_b[0][1] = b[0];
    m_b[1][1] = b[1];
    m_b[2][1] = b[2];
    let mut m_c = m;
    m_c[0][2] = b[0];
    m_c[1][2] = b[1];
    m_c[2][2] = b[2];
    Some([det_3x3(m_a) / det, det_3x3(m_b) / det, det_3x3(m_c) / det])
}

fn det_3x3(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 4).iter().all(|x| x.is_none()));
        assert!(compute(&c[..10], 30).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 30).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 30);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn perfect_quadratic_yields_zero() {
        // Closes follow y = a + bx + cx² exactly. The stdev floor in
        // compute() catches the near-perfect-fit case and emits 0,
        // avoiding float-noise amplification.
        let c: Vec<f64> = (0..50)
            .map(|i| {
                let x = i as f64;
                100.0 + 0.5 * x + 0.01 * x * x
            })
            .collect();
        let r = compute(&c, 30);
        for v in r.iter().skip(30).flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn spike_above_fit_yields_positive_z() {
        let mut c: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        c[29] = 200.0; // far above fit
        let r = compute(&c, 30);
        assert!(r[29].unwrap() > 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 30).len(), 50);
    }
}
