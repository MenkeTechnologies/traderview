//! Dupire Local Volatility — implied local-vol surface from a call-price
//! grid via the Dupire (1994) formula:
//!
//!   σ_loc²(K, T) = (∂C/∂T + r·K·∂C/∂K + (r-q)·... )
//!                  / (½ K² · ∂²C/∂K²)
//!
//! Simplified form assuming q = 0 and risk-neutral measure:
//!
//!   σ_loc²(K, T) = (∂C/∂T + r·K·∂C/∂K) / (½ K² · ∂²C/∂K²)
//!
//! Finite-difference approximation:
//!   ∂C/∂T   ≈ (C(K, T+h_t) - C(K, T-h_t)) / (2 h_t)
//!   ∂C/∂K   ≈ (C(K+h_k, T) - C(K-h_k, T)) / (2 h_k)
//!   ∂²C/∂K² ≈ (C(K+h_k, T) - 2·C(K, T) + C(K-h_k, T)) / h_k²
//!
//! Caller provides a 2-D `call_prices[t][k]` grid plus the `strikes` and
//! `expiries` axes. Output is a `(T, K)` matrix of local variances.
//! Cells at grid boundaries are `None`.
//!
//! Pure compute. Companion to `black_scholes`, `volatility_smile`,
//! `svi_volatility_smile`.

#[derive(Debug)]
pub struct Report {
    pub local_var: Vec<Vec<Option<f64>>>,
    pub local_vol: Vec<Vec<Option<f64>>>,
}

pub fn compute(
    call_prices: &[Vec<f64>],
    strikes: &[f64],
    expiries: &[f64],
    risk_free_rate: f64,
) -> Option<Report> {
    let nt = expiries.len();
    let nk = strikes.len();
    if nt < 3 || nk < 3 {
        return None;
    }
    if call_prices.len() != nt {
        return None;
    }
    if call_prices.iter().any(|row| row.len() != nk) {
        return None;
    }
    if !risk_free_rate.is_finite() {
        return None;
    }
    if expiries.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    if strikes.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    if call_prices.iter().any(|r| r.iter().any(|c| !c.is_finite())) {
        return None;
    }
    // Strikes and expiries must be monotonic increasing.
    for w in strikes.windows(2) {
        if w[1] <= w[0] {
            return None;
        }
    }
    for w in expiries.windows(2) {
        if w[1] <= w[0] {
            return None;
        }
    }
    let mut local_var = vec![vec![None::<f64>; nk]; nt];
    let mut local_vol = vec![vec![None::<f64>; nk]; nt];
    for ti in 1..nt - 1 {
        for ki in 1..nk - 1 {
            let h_t_lo = expiries[ti] - expiries[ti - 1];
            let h_t_hi = expiries[ti + 1] - expiries[ti];
            let h_k_lo = strikes[ki] - strikes[ki - 1];
            let h_k_hi = strikes[ki + 1] - strikes[ki];
            let dc_dt = (call_prices[ti + 1][ki] - call_prices[ti - 1][ki]) / (h_t_lo + h_t_hi);
            let dc_dk = (call_prices[ti][ki + 1] - call_prices[ti][ki - 1]) / (h_k_lo + h_k_hi);
            let d2c_dk2 = 2.0
                * (h_k_lo * call_prices[ti][ki + 1] - (h_k_lo + h_k_hi) * call_prices[ti][ki]
                    + h_k_hi * call_prices[ti][ki - 1])
                / (h_k_lo * h_k_hi * (h_k_lo + h_k_hi));
            let denom = 0.5 * strikes[ki] * strikes[ki] * d2c_dk2;
            if denom <= 1e-15 {
                continue;
            }
            let numer = dc_dt + risk_free_rate * strikes[ki] * dc_dk;
            let var = numer / denom;
            if var.is_finite() && var > 0.0 {
                local_var[ti][ki] = Some(var);
                local_vol[ti][ki] = Some(var.sqrt());
            }
        }
    }
    Some(Report {
        local_var,
        local_vol,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs_call(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
        if t <= 0.0 || sigma <= 0.0 {
            return (s - k).max(0.0);
        }
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
        let prices = vec![vec![1.0; 5]; 5];
        let strikes = vec![80.0, 90.0, 100.0, 110.0, 120.0];
        let expiries = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let bad = vec![vec![1.0; 5]; 2];
        assert!(compute(&bad, &strikes, &expiries, 0.05).is_none());
        let bad_strikes = vec![80.0, 120.0, 100.0, 110.0, 130.0]; // not monotonic
        assert!(compute(&prices, &bad_strikes, &expiries, 0.05).is_none());
        let bad_expiries = vec![0.0, 0.2, 0.3, 0.4, 0.5];
        assert!(compute(&prices, &strikes, &bad_expiries, 0.05).is_none());
        assert!(compute(&prices, &strikes, &expiries, f64::NAN).is_none());
    }

    #[test]
    fn dupire_recovers_constant_vol_for_bs_grid() {
        // Build a BS call grid with constant σ = 0.25, then run Dupire.
        // Interior cells should recover ~0.25² = 0.0625.
        let s0 = 100.0;
        let sigma = 0.25;
        let r = 0.0;
        let strikes: Vec<f64> = (-10_i32..=10).map(|i| s0 + i as f64 * 2.0).collect();
        let expiries: Vec<f64> = (1..=10).map(|i| i as f64 * 0.1).collect();
        let prices: Vec<Vec<f64>> = expiries
            .iter()
            .map(|&t| {
                strikes
                    .iter()
                    .map(|&k| bs_call(s0, k, t, r, sigma))
                    .collect()
            })
            .collect();
        let rep = compute(&prices, &strikes, &expiries, r).unwrap();
        // Check ATM cell in middle expiry.
        let mid_t = expiries.len() / 2;
        let mid_k = strikes.len() / 2;
        let v = rep.local_var[mid_t][mid_k].unwrap();
        assert!((v - sigma * sigma).abs() < 0.01);
    }

    #[test]
    fn boundary_cells_are_none() {
        let prices = vec![vec![1.0; 5]; 5];
        let strikes = vec![80.0, 90.0, 100.0, 110.0, 120.0];
        let expiries = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let rep = compute(&prices, &strikes, &expiries, 0.05).unwrap();
        // First/last rows always None.
        assert!(rep.local_var[0].iter().all(|x| x.is_none()));
        assert!(rep.local_var[4].iter().all(|x| x.is_none()));
        for row in &rep.local_var[1..4] {
            assert!(row[0].is_none() && row[4].is_none());
        }
    }

    #[test]
    fn output_dims_match_input_grid() {
        let prices = vec![vec![1.0; 5]; 4];
        let strikes = vec![80.0, 90.0, 100.0, 110.0, 120.0];
        let expiries = vec![0.1, 0.2, 0.3, 0.4];
        let rep = compute(&prices, &strikes, &expiries, 0.05).unwrap();
        assert_eq!(rep.local_var.len(), 4);
        assert!(rep.local_var.iter().all(|row| row.len() == 5));
    }

    #[test]
    fn local_vol_equals_sqrt_local_var() {
        let s0 = 100.0;
        let sigma = 0.25;
        let r = 0.0;
        let strikes: Vec<f64> = (-5_i32..=5).map(|i| s0 + i as f64 * 4.0).collect();
        let expiries: Vec<f64> = (1..=6).map(|i| i as f64 * 0.1).collect();
        let prices: Vec<Vec<f64>> = expiries
            .iter()
            .map(|&t| {
                strikes
                    .iter()
                    .map(|&k| bs_call(s0, k, t, r, sigma))
                    .collect()
            })
            .collect();
        let rep = compute(&prices, &strikes, &expiries, r).unwrap();
        for (i, row) in rep.local_var.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if let Some(v) = cell {
                    let vol = rep.local_vol[i][j].unwrap();
                    assert!((vol * vol - v).abs() < 1e-9);
                }
            }
        }
    }
}
