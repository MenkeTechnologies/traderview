//! White (1980) Heteroskedasticity-Consistent Standard Errors (HC0).
//!
//! For an OLS regression y = α + β·x + ε with heteroskedastic
//! residuals, the HC0 variance of β̂ is:
//!
//!   V̂_β,HC0 = (X'X)⁻¹ · (Σ_t x_t · x_t' · ê_t²) · (X'X)⁻¹
//!
//! Unlike Newey-West, White does NOT model autocorrelation — it
//! handles only heteroskedasticity. For iid-but-heteroskedastic data
//! (e.g. cross-sectional regressions), White is the right choice.
//!
//! Common finite-sample corrections (selectable):
//!   - HC0: no correction (raw White)
//!   - HC1: multiply by n / (n − p)
//!   - HC2: divide each ê_t² by (1 − h_t)
//!   - HC3: divide each ê_t² by (1 − h_t)²       (MacKinnon-White 1985)
//!
//! Where h_t is the leverage of observation t (diagonal of the hat
//! matrix). HC3 has the best small-sample behavior; HC0 is the
//! "textbook White".
//!
//! Univariate predictor only.
//!
//! Pure compute. Companion to `newey_west`, `breusch_pagan_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HcVariant {
    #[default]
    Hc0,
    Hc1,
    Hc2,
    Hc3,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhiteRobustReport {
    pub alpha: f64,
    pub beta: f64,
    pub se_beta_white: f64,
    pub se_alpha_white: f64,
    pub t_stat_beta_white: f64,
    pub se_beta_ols: f64,
    pub variant: HcVariant,
    pub n_observations: usize,
}

pub fn estimate(x: &[f64], y: &[f64], variant: HcVariant) -> Option<WhiteRobustReport> {
    let n = x.len();
    if n < 5 || y.len() != n {
        return None;
    }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let x_mean: f64 = x.iter().sum::<f64>() / n_f;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..n {
        sxx += (x[i] - x_mean).powi(2);
        sxy += (x[i] - x_mean) * (y[i] - y_mean);
    }
    if sxx <= 0.0 {
        return None;
    }
    let beta = sxy / sxx;
    let alpha = y_mean - beta * x_mean;
    let resid: Vec<f64> = (0..n).map(|i| y[i] - alpha - beta * x[i]).collect();
    // Classical OLS β SE.
    let sigma2_ols = resid.iter().map(|r| r * r).sum::<f64>() / (n_f - 2.0);
    let se_beta_ols = (sigma2_ols / sxx).sqrt();
    // Leverages h_t for HC2/HC3.
    let leverages: Vec<f64> = (0..n)
        .map(|i| 1.0 / n_f + (x[i] - x_mean).powi(2) / sxx)
        .collect();
    // White HC sandwich.
    let mut meat_beta = 0.0_f64;
    let mut meat_alpha = 0.0_f64;
    for i in 0..n {
        let scaled = match variant {
            HcVariant::Hc0 => resid[i] * resid[i],
            HcVariant::Hc1 => resid[i] * resid[i],
            HcVariant::Hc2 => {
                let denom = (1.0 - leverages[i]).max(1e-12);
                resid[i] * resid[i] / denom
            }
            HcVariant::Hc3 => {
                let denom = (1.0 - leverages[i]).max(1e-12);
                resid[i] * resid[i] / (denom * denom)
            }
        };
        let xc = x[i] - x_mean;
        meat_beta += xc * xc * scaled;
        meat_alpha += scaled;
    }
    let mut var_beta = meat_beta / (sxx * sxx);
    let mut var_alpha = meat_alpha * (1.0 / n_f + x_mean * x_mean / sxx).powi(2);
    if matches!(variant, HcVariant::Hc1) {
        let dof_correction = n_f / (n_f - 2.0);
        var_beta *= dof_correction;
        var_alpha *= dof_correction;
    }
    let se_beta_white = var_beta.max(0.0).sqrt();
    let se_alpha_white = var_alpha.max(0.0).sqrt();
    let t_stat = if se_beta_white > 0.0 {
        beta / se_beta_white
    } else {
        0.0
    };
    Some(WhiteRobustReport {
        alpha,
        beta,
        se_beta_white,
        se_alpha_white,
        t_stat_beta_white: t_stat,
        se_beta_ols,
        variant,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(estimate(&[1.0; 3], &[2.0; 3], HcVariant::Hc0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let x = vec![1.0_f64; 30];
        let mut y = vec![1.0_f64; 30];
        y[5] = f64::NAN;
        assert!(estimate(&x, &y, HcVariant::Hc0).is_none());
    }

    #[test]
    fn flat_predictor_returns_none() {
        let x = vec![1.0_f64; 30];
        let y: Vec<f64> = (0..30).map(|i| i as f64).collect();
        assert!(estimate(&x, &y, HcVariant::Hc0).is_none());
    }

    #[test]
    fn homoskedastic_hc_close_to_ols() {
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
                2.0 * xi + eps
            })
            .collect();
        let r = estimate(&x, &y, HcVariant::Hc0).unwrap();
        let rel = (r.se_beta_white - r.se_beta_ols).abs() / r.se_beta_ols;
        assert!(
            rel < 0.30,
            "homoskedastic: HC SE {} close to OLS {}",
            r.se_beta_white,
            r.se_beta_ols
        );
    }

    #[test]
    fn heteroskedastic_hc_diverges_from_ols() {
        // Variance strongly proportional to x² → large HC adjustment.
        let mut state: u64 = 11;
        let x: Vec<f64> = (1..=300).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * (xi * xi / 200.0);
                2.0 * xi + eps
            })
            .collect();
        let r = estimate(&x, &y, HcVariant::Hc0).unwrap();
        let rel = (r.se_beta_white - r.se_beta_ols).abs() / r.se_beta_ols;
        assert!(
            rel > 0.10,
            "heteroskedastic: HC SE {} should diverge from OLS {}",
            r.se_beta_white,
            r.se_beta_ols
        );
    }

    #[test]
    fn hc1_inflates_vs_hc0() {
        let mut state: u64 = 99;
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * (xi / 5.0 + 1.0);
                2.0 * xi + eps
            })
            .collect();
        let hc0 = estimate(&x, &y, HcVariant::Hc0).unwrap();
        let hc1 = estimate(&x, &y, HcVariant::Hc1).unwrap();
        // HC1 multiplies by n/(n-2) so se = se_hc0 · √(n/(n-2)) > se_hc0.
        assert!(hc1.se_beta_white > hc0.se_beta_white);
    }

    #[test]
    fn hc3_largest_under_high_leverage() {
        // High-leverage observation at the right tail.
        let mut x: Vec<f64> = (0..29).map(|i| i as f64).collect();
        x.push(1000.0); // huge leverage
        let mut state: u64 = 7;
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
                2.0 * xi + eps
            })
            .collect();
        let hc0 = estimate(&x, &y, HcVariant::Hc0).unwrap();
        let hc3 = estimate(&x, &y, HcVariant::Hc3).unwrap();
        // HC3 divides by (1 − h)², so high-leverage residual gets inflated.
        assert!(hc3.se_beta_white >= hc0.se_beta_white);
    }

    #[test]
    fn coefficients_match_ols() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        let r = estimate(&x, &y, HcVariant::Hc0).unwrap();
        assert!((r.beta - 2.0).abs() < 1e-9);
        assert!((r.alpha - 1.0).abs() < 1e-9);
    }
}
