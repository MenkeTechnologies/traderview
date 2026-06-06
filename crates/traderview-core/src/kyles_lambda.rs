//! Kyle's Lambda — Kyle (1985) price-impact slope estimator.
//!
//!   Δp = λ · signed_volume + ε
//!
//! Estimated via OLS on a rolling window of (signed_volume, Δprice)
//! pairs. λ is the price impact per unit of signed flow — the
//! microstructure measure of market depth (lower λ = deeper book).
//!
//! Inputs: per-bar price changes Δp and signed volumes (positive = net
//! buy pressure, negative = net sell pressure — typically Lee-Ready
//! classified). For the rolling regression we use the closed-form OLS
//! slope:
//!
//!   λ = Σ(x · y) / Σ(x²)     (no-intercept regression)
//!
//! which matches Kyle's setup where ε is mean-zero and we assume zero
//! drift at the microstructure timescale.
//!
//! Pure compute.

pub fn compute(price_changes: &[f64], signed_volumes: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = price_changes.len();
    let mut out = vec![None; n];
    if window < 2 || price_changes.len() != signed_volumes.len() || n < window {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(window - 1) {
        let lo = i + 1 - window;
        let mut sxy = 0.0_f64;
        let mut sxx = 0.0_f64;
        let mut valid = 0;
        for j in lo..=i {
            let x = signed_volumes[j];
            let y = price_changes[j];
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            sxy += x * y;
            sxx += x * x;
            valid += 1;
        }
        if valid < 2 {
            continue;
        }
        if sxx > 0.0 {
            let lam = sxy / sxx;
            if lam.is_finite() {
                *slot = Some(lam);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 20).is_empty());
    }

    #[test]
    fn length_mismatch_returns_all_none() {
        let p = vec![0.01; 30];
        let v = vec![1_000.0; 15];
        assert!(compute(&p, &v, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn window_too_small_returns_all_none() {
        let p = vec![0.01; 30];
        let v = vec![1_000.0; 30];
        assert!(compute(&p, &v, 0).iter().all(|x| x.is_none()));
        assert!(compute(&p, &v, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfect_linear_relationship_recovers_slope() {
        // Δp = 0.5 · v + 0 — λ should be 0.5.
        let v: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let p: Vec<f64> = v.iter().map(|x| 0.5 * x).collect();
        let out = compute(&p, &v, 20);
        let lam = out[29].expect("populated");
        assert!((lam - 0.5).abs() < 1e-9, "expected λ=0.5, got {lam}");
    }

    #[test]
    fn zero_signed_flow_window_yields_none() {
        // All volumes 0 → sxx = 0 → division-by-zero guard returns None.
        let p = vec![0.01; 30];
        let v = vec![0.0; 30];
        let out = compute(&p, &v, 20);
        assert!(out[29].is_none());
    }

    #[test]
    fn nan_pairs_skipped() {
        let mut p = vec![0.01; 30];
        let mut v = vec![1_000.0; 30];
        p[15] = f64::NAN;
        v[16] = f64::NAN;
        let out = compute(&p, &v, 20);
        // Still populated from the other 18+ valid pairs.
        assert!(out[29].is_some());
    }

    #[test]
    fn negative_signed_flow_handled() {
        // Δp = -0.3 · v.
        let v: Vec<f64> = (-15..15).map(|i| i as f64).collect();
        let p: Vec<f64> = v.iter().map(|x| -0.3 * x).collect();
        let out = compute(&p, &v, 20);
        let lam = out[29].expect("populated");
        assert!((lam + 0.3).abs() < 1e-9);
    }

    #[test]
    fn noisy_relationship_returns_finite_estimate() {
        // y = 0.4 x + small noise — λ should be close to 0.4.
        let v: Vec<f64> = (1..=50).map(|i| i as f64).collect();
        let p: Vec<f64> = v
            .iter()
            .enumerate()
            .map(|(i, x)| 0.4 * x + ((i as f64 * 0.7).sin() * 0.5))
            .collect();
        let out = compute(&p, &v, 30);
        let lam = out[49].expect("populated");
        assert!((lam - 0.4).abs() < 0.1, "expected λ ≈ 0.4, got {lam}");
    }
}
