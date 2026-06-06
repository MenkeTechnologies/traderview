//! Forward Implied Volatility Bootstrap — extracts the forward
//! volatility between two expiries from their spot-implied vols using
//! variance additivity.
//!
//! For a flat-forward (piecewise-constant) vol model on [0, T2]:
//!
//!   σ²(0, T2) · T2 = σ²(0, T1) · T1 + σ²_fwd(T1, T2) · (T2 - T1)
//!
//! Solving for σ_fwd:
//!
//!   σ_fwd(T1, T2) = sqrt( (σ²(0, T2) · T2 - σ²(0, T1) · T1) / (T2 - T1) )
//!
//! Useful for:
//!   - Pricing calendar spreads (the value of long-back / short-front
//!     vol exposure on a single strike).
//!   - Detecting vol-curve dislocations: if a published "forward vol"
//!     differs from the bootstrap, it's an arb signal.
//!
//! Caller passes (expiry, spot_iv) pairs sorted ascending by expiry.
//! Returns forward-vols between each adjacent pair plus the
//! cumulative variance series for sanity checks.
//!
//! Failure modes returned via `arbitrage_violations`:
//!   - Negative forward variance — the longer-dated vol is too low
//!     relative to the shorter-dated one (no-arb violation).
//!
//! Pure compute. Companion to `volatility_smile`, `svi_volatility_smile`,
//! `term_premium_estimator`.

#[derive(Debug)]
pub struct Report {
    pub forward_vols: Vec<f64>, // length = n - 1
    pub cumulative_variance: Vec<f64>,
    pub arbitrage_violations: Vec<usize>, // indices in `forward_vols` flagged
}

pub fn compute(expiries: &[f64], spot_iv: &[f64]) -> Option<Report> {
    let n = expiries.len();
    if n < 2 || spot_iv.len() != n {
        return None;
    }
    if expiries
        .iter()
        .chain(spot_iv.iter())
        .any(|x| !x.is_finite())
    {
        return None;
    }
    if expiries.iter().any(|&t| t <= 0.0) {
        return None;
    }
    if spot_iv.iter().any(|&v| v < 0.0) {
        return None;
    }
    for w in expiries.windows(2) {
        if w[1] <= w[0] {
            return None;
        }
    }
    let total_variance: Vec<f64> = expiries
        .iter()
        .zip(spot_iv.iter())
        .map(|(t, iv)| iv * iv * t)
        .collect();
    let mut forward_vols = Vec::with_capacity(n - 1);
    let mut violations = Vec::new();
    for i in 1..n {
        let dt = expiries[i] - expiries[i - 1];
        let dv = total_variance[i] - total_variance[i - 1];
        if dv < 0.0 {
            violations.push(i - 1);
            forward_vols.push(0.0);
        } else {
            forward_vols.push((dv / dt).sqrt());
        }
    }
    Some(Report {
        forward_vols,
        cumulative_variance: total_variance,
        arbitrage_violations: violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let t = vec![0.5_f64, 1.0];
        let v = vec![0.2_f64, 0.25];
        assert!(compute(&[], &[]).is_none());
        assert!(compute(&t, &v[..1]).is_none());
        let zero_t = vec![0.0_f64, 1.0];
        assert!(compute(&zero_t, &v).is_none());
        let neg_v = vec![-0.1_f64, 0.25];
        assert!(compute(&t, &neg_v).is_none());
        let non_mono_t = vec![1.0_f64, 0.5];
        assert!(compute(&non_mono_t, &v).is_none());
        let mut nan = v.clone();
        nan[0] = f64::NAN;
        assert!(compute(&t, &nan).is_none());
    }

    #[test]
    fn flat_iv_term_structure_yields_constant_forward() {
        // σ(0, 0.5) = σ(0, 1) = 0.20 → forward σ(0.5, 1) = 0.20.
        let t = vec![0.5_f64, 1.0];
        let v = vec![0.20_f64, 0.20];
        let r = compute(&t, &v).unwrap();
        assert_eq!(r.forward_vols.len(), 1);
        assert!((r.forward_vols[0] - 0.20).abs() < 1e-9);
        assert!(r.arbitrage_violations.is_empty());
    }

    #[test]
    fn rising_term_structure_yields_higher_forward() {
        // Total variance grows faster than linearly → forward > both spots.
        let t = vec![0.5_f64, 1.0];
        let v = vec![0.20_f64, 0.30];
        let r = compute(&t, &v).unwrap();
        // var(0,1) - var(0,0.5) = 0.09 - 0.02 = 0.07; /0.5 = 0.14 → σ = 0.374
        assert!(r.forward_vols[0] > 0.30);
        assert!(r.arbitrage_violations.is_empty());
    }

    #[test]
    fn arbitrage_violation_detected_when_back_vol_too_low() {
        // σ(0, 1) = 0.10 < σ(0, 0.5) = 0.30 implies total variance falls
        // from 0.045 to 0.01 → no-arb violated.
        let t = vec![0.5_f64, 1.0];
        let v = vec![0.30_f64, 0.10];
        let r = compute(&t, &v).unwrap();
        assert_eq!(r.arbitrage_violations, vec![0]);
        assert_eq!(r.forward_vols[0], 0.0);
    }

    #[test]
    fn cumulative_variance_matches_iv_squared_t() {
        let t = vec![0.25_f64, 0.5, 1.0, 2.0];
        let v = vec![0.20_f64, 0.22, 0.25, 0.28];
        let r = compute(&t, &v).unwrap();
        for i in 0..t.len() {
            let expected = v[i] * v[i] * t[i];
            assert!((r.cumulative_variance[i] - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn forward_vol_count_is_n_minus_one() {
        let t = vec![0.25_f64, 0.5, 1.0, 2.0];
        let v = vec![0.20_f64, 0.22, 0.25, 0.28];
        let r = compute(&t, &v).unwrap();
        assert_eq!(r.forward_vols.len(), 3);
    }

    #[test]
    fn multi_tenor_bootstrap_is_monotone_when_variance_rises_linearly() {
        // Total variance linear in T → forward equals spot at every step.
        let t = vec![0.25_f64, 0.5, 1.0];
        // Construct so that iv²·t = 0.04·t linearly: iv = 0.2 always.
        let v = vec![0.20_f64; 3];
        let r = compute(&t, &v).unwrap();
        for f in &r.forward_vols {
            assert!((f - 0.20).abs() < 1e-9);
        }
    }
}
