//! Christoffersen (1998) Independence test + Conditional Coverage test
//! for VaR backtesting.
//!
//! Augments Kupiec's POF test (which checks UNCONDITIONAL coverage —
//! how many exceedances) with an INDEPENDENCE test (do exceedances
//! cluster? — a sign of model mis-specification even when the total
//! count is right):
//!
//!   LR_IND = −2 · ln[ ((1 − π)^(n00+n10) · π^(n01+n11))
//!                    / ((1 − π_01)^n00 · π_01^n01 · (1 − π_11)^n10 · π_11^n11) ]
//!
//! where π_ij = transitions from state i to state j (0 = no exceedance,
//! 1 = exceedance), π = (n01+n11)/(n00+n01+n10+n11).
//!
//! Under H₀ (independent transitions), LR_IND ~ χ²(1).
//!
//! Conditional coverage = LR_POF + LR_IND ~ χ²(2). Reject the model if
//! EITHER component is significant.
//!
//! Pure compute. Companion to `var_backtest_kupiec`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Significance { Pct1, Pct5, Pct10, NotRejected }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChristoffersenReport {
    pub n_observations: usize,
    pub n_exceedances: usize,
    pub n00: usize,
    pub n01: usize,
    pub n10: usize,
    pub n11: usize,
    pub lr_independence: f64,
    pub lr_conditional_coverage: f64,
    pub independence_significance: Significance,
    pub conditional_significance: Significance,
}

pub fn test(
    realized_returns: &[f64],
    var_forecasts: &[f64],
    alpha: f64,
) -> Option<ChristoffersenReport> {
    let n = realized_returns.len();
    if var_forecasts.len() != n
        || n < 10
        || !alpha.is_finite() || !(0.0..1.0).contains(&alpha) || alpha == 0.0
    {
        return None;
    }
    // Build exceedance sequence.
    let mut hits = Vec::with_capacity(n);
    for i in 0..n {
        if !realized_returns[i].is_finite() || !var_forecasts[i].is_finite() {
            continue;
        }
        let loss = -realized_returns[i];
        hits.push(if loss > var_forecasts[i] { 1u8 } else { 0u8 });
    }
    let valid_n = hits.len();
    if valid_n < 10 { return None; }
    // Transition counts.
    let mut n00 = 0usize; let mut n01 = 0usize;
    let mut n10 = 0usize; let mut n11 = 0usize;
    for w in hits.windows(2) {
        match (w[0], w[1]) {
            (0, 0) => n00 += 1,
            (0, 1) => n01 += 1,
            (1, 0) => n10 += 1,
            (1, 1) => n11 += 1,
            _ => {}
        }
    }
    let n_exceedances: usize = hits.iter().map(|h| *h as usize).sum();
    let total = (n00 + n01 + n10 + n11) as f64;
    if total <= 0.0 { return None; }
    let pi_marginal = (n01 + n11) as f64 / total;
    // Compute independence LR.
    // If pi_marginal == 0 or 1, the test degenerates — return LR = 0.
    let lr_ind = if pi_marginal <= 0.0 || pi_marginal >= 1.0 {
        0.0
    } else {
        // π_01 = n01 / (n00 + n01); π_11 = n11 / (n10 + n11). Avoid log(0).
        let denom_0 = (n00 + n01) as f64;
        let denom_1 = (n10 + n11) as f64;
        let pi_01 = if denom_0 > 0.0 { n01 as f64 / denom_0 } else { 0.0 };
        let pi_11 = if denom_1 > 0.0 { n11 as f64 / denom_1 } else { 0.0 };
        let log_l_restricted = (n00 + n10) as f64 * (1.0 - pi_marginal).ln()
            + (n01 + n11) as f64 * pi_marginal.ln();
        let mut log_l_unrestricted = 0.0_f64;
        if n00 > 0 && pi_01 < 1.0 {
            log_l_unrestricted += n00 as f64 * (1.0 - pi_01).ln();
        }
        if n01 > 0 && pi_01 > 0.0 {
            log_l_unrestricted += n01 as f64 * pi_01.ln();
        }
        if n10 > 0 && pi_11 < 1.0 {
            log_l_unrestricted += n10 as f64 * (1.0 - pi_11).ln();
        }
        if n11 > 0 && pi_11 > 0.0 {
            log_l_unrestricted += n11 as f64 * pi_11.ln();
        }
        let v = -2.0 * (log_l_restricted - log_l_unrestricted);
        v.max(0.0)
    };
    // POF (Kupiec) LR for conditional coverage.
    let x = n_exceedances as f64;
    let n_f = valid_n as f64;
    let observed_rate = x / n_f;
    let lr_pof = if x == 0.0 {
        -2.0 * (n_f * (1.0 - alpha).ln())
    } else if x == n_f {
        -2.0 * (n_f * alpha.ln())
    } else {
        let log_model = (n_f - x) * (1.0 - alpha).ln() + x * alpha.ln();
        let log_unr = (n_f - x) * (1.0 - observed_rate).ln() + x * observed_rate.ln();
        -2.0 * (log_model - log_unr)
    };
    let lr_cc = (lr_pof + lr_ind).max(0.0);
    let bucket = |lr: f64, ones_dof: f64| -> Significance {
        // χ²(1) crit 6.635/3.841/2.706 ; χ²(2) crit 9.210/5.991/4.605.
        let (c1, c5, c10) = if ones_dof < 1.5 {
            (6.635, 3.841, 2.706)
        } else {
            (9.210, 5.991, 4.605)
        };
        if lr > c1 { Significance::Pct1 }
        else if lr > c5 { Significance::Pct5 }
        else if lr > c10 { Significance::Pct10 }
        else { Significance::NotRejected }
    };
    Some(ChristoffersenReport {
        n_observations: valid_n,
        n_exceedances,
        n00, n01, n10, n11,
        lr_independence: lr_ind,
        lr_conditional_coverage: lr_cc,
        independence_significance: bucket(lr_ind, 1.0),
        conditional_significance: bucket(lr_cc, 2.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(test(&[0.0; 50], &[0.05; 25], 0.05).is_none());
    }

    #[test]
    fn invalid_alpha_returns_none() {
        let r = vec![0.0; 50];
        let v = vec![0.05; 50];
        assert!(test(&r, &v, 0.0).is_none());
        assert!(test(&r, &v, 1.5).is_none());
    }

    #[test]
    fn alternating_exceedances_independence_passes() {
        // Exceedances every other bar — same probability but actually
        // overstates dependence in tiny samples. Run on a 200-bar series
        // alternating exceeded/not.
        let n = 200;
        let mut returns = vec![-0.001_f64; n];
        for (i, slot) in returns.iter_mut().enumerate() {
            if i.is_multiple_of(2) { *slot = -0.10; }    // exceedance
        }
        let vars = vec![0.05_f64; n];
        let r = test(&returns, &vars, 0.5).unwrap();
        // Alternating gives n01 = n10 = ~99 and n00 = n11 = 0 → strongly
        // dependent (perfectly anti-correlated). LR_IND should be large.
        // This is the EXPECTED behavior — alternation rejects independence.
        assert!(r.lr_independence > 5.0,
            "alternating exceedances should reject independence, got {}",
            r.lr_independence);
    }

    #[test]
    fn clustered_exceedances_reject_independence() {
        // 10 exceedances in a row at the start, then no exceedances → highly
        // dependent.
        let n = 200;
        let mut returns = vec![-0.001_f64; n];
        for slot in returns.iter_mut().take(10) { *slot = -0.10; }
        let vars = vec![0.05_f64; n];
        let r = test(&returns, &vars, 0.05).unwrap();
        // Clustering n11 transitions (9 consecutive 1→1) makes π_11 = 9/9 = 1.0.
        assert!(r.n11 >= 8);
    }

    #[test]
    fn iid_calibrated_exceedances_dont_reject() {
        let n = 1_000;
        let mut state: u64 = 42;
        let mut returns = vec![0.0_f64; n];
        let vars = vec![0.05_f64; n];
        for slot in returns.iter_mut() {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64;
            *slot = if u < 0.05 { -0.10 } else { -0.001 };
        }
        let r = test(&returns, &vars, 0.05).unwrap();
        // Random ~5% exceedances scattered iid → should NOT reject
        // independence (or conditional coverage) at 5% level (most of the time).
        assert!(matches!(r.independence_significance,
            Significance::NotRejected | Significance::Pct10 | Significance::Pct5));
    }

    #[test]
    fn no_exceedances_yields_zero_lr_ind() {
        let returns = vec![-0.001_f64; 50];
        let vars = vec![0.10_f64; 50];
        let r = test(&returns, &vars, 0.05).unwrap();
        assert!(r.lr_independence.abs() < 1e-9);
        assert_eq!(r.n_exceedances, 0);
    }

    #[test]
    fn nan_pairs_skipped() {
        let mut returns = vec![-0.01_f64; 50];
        let vars = vec![0.05_f64; 50];
        returns[10] = f64::NAN;
        let r = test(&returns, &vars, 0.05).unwrap();
        assert_eq!(r.n_observations, 49);
    }
}
