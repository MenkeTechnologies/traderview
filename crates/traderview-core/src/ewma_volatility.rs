//! EWMA Volatility — Exponentially-Weighted Moving Average of squared
//! returns (RiskMetrics-style).
//!
//!   σ²_t = λ · σ²_{t−1} + (1 − λ) · r²_t
//!   σ_t  = √σ²_t
//!
//! Default `lambda = 0.94` is the RiskMetrics convention for daily
//! returns. Output is per-bar instantaneous vol (per-period). To
//! annualize, multiply by √annualization_factor (caller responsibility
//! — typical: √252 for daily bars).
//!
//! Pure compute.

pub fn compute(closes: &[f64], lambda: f64) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if !lambda.is_finite() || lambda <= 0.0 || lambda >= 1.0 || n < 2 {
        return out;
    }
    let mut variance: Option<f64> = None;
    for i in 1..n {
        if !closes[i].is_finite() || !closes[i - 1].is_finite()
            || closes[i] <= 0.0 || closes[i - 1] <= 0.0
        {
            // Carry the prior variance if we have one; just don't update.
            if let Some(v) = variance {
                if v >= 0.0 { out[i] = Some(v.sqrt()); }
            }
            continue;
        }
        let r = (closes[i] / closes[i - 1]).ln();
        let r2 = r * r;
        let new_var = match variance {
            None => r2,    // seed with first squared return
            Some(v) => lambda * v + (1.0 - lambda) * r2,
        };
        if new_var.is_finite() && new_var >= 0.0 {
            variance = Some(new_var);
            out[i] = Some(new_var.sqrt());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 0.94).is_empty());
    }

    #[test]
    fn invalid_lambda_returns_all_none() {
        let v = vec![100.0; 50];
        for bad in [0.0, 1.0, -0.1, 1.1, f64::NAN, f64::INFINITY] {
            assert!(compute(&v, bad).iter().all(|x| x.is_none()), "lambda={bad}");
        }
    }

    #[test]
    fn too_short_returns_all_none() {
        let v = vec![100.0];
        assert!(compute(&v, 0.94).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_volatility() {
        let v = vec![100.0; 50];
        let out = compute(&v, 0.94);
        for x in out.iter().flatten() {
            assert!(x.abs() < 1e-12, "flat series should yield zero vol, got {x}");
        }
    }

    #[test]
    fn rising_constant_pct_returns_yield_stable_vol() {
        // 1% daily up move repeatedly — variance should converge to (ln 1.01)².
        let mut v = vec![100.0];
        for _ in 0..200 { v.push(v.last().unwrap() * 1.01); }
        let out = compute(&v, 0.94);
        let last = out.last().copied().flatten().expect("populated");
        let expected = (1.01_f64).ln();
        assert!((last - expected.abs()).abs() < 1e-3,
            "should converge to ln(1.01) ≈ {expected}, got {last}");
    }

    #[test]
    fn extreme_outlier_spikes_then_decays() {
        // 100 flat bars, one +5% spike, then flat — variance should
        // jump then decay each step by factor λ.
        let mut v = vec![100.0; 100];
        v.push(105.0);
        v.extend_from_slice(&[100.0; 50]);
        let out = compute(&v, 0.94);
        let spike_vol = out[100].expect("populated");
        let later_vol = out[120].expect("populated");
        assert!(later_vol < spike_vol, "vol should decay after spike: spike={spike_vol} later={later_vol}");
    }

    #[test]
    fn nan_close_carries_prior_vol() {
        let mut v = vec![100.0; 50];
        v[25] = f64::NAN;
        let out = compute(&v, 0.94);
        // No panic; vol is populated around NaN.
        assert!(out[24].is_some());
        assert!(out[26].is_some());
    }

    #[test]
    fn zero_close_skipped_safely() {
        let mut v = vec![100.0; 50];
        v[25] = 0.0;
        let out = compute(&v, 0.94);
        assert!(out.iter().any(|x| x.is_some()));
    }
}
