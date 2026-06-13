//! HSA triple-tax advantage — HSA vs a taxable account over time.
//!
//! The HSA is the only account taxed favorably three ways: contributions are
//! deductible (pre-tax), growth is tax-free, and withdrawals for qualified
//! medical expenses are tax-free. This quantifies that edge by projecting an
//! HSA against a taxable brokerage account funded with the same gross income:
//!
//!   * **HSA**: the full contribution goes in pre-tax, compounds tax-free, and
//!     comes out tax-free → ending value = the annuity future value.
//!   * **Taxable**: the same pre-tax dollars are taxed first (so only
//!     `contribution × (1 − marginal)` is invested), compound, and the gains
//!     are taxed at the long-term rate at the end.
//!
//! The difference is the dollar value of the triple-tax treatment. Pure
//! compute (end-of-year contributions, gains taxed once at the horizon).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct HsaInput {
    pub annual_contribution_usd: f64,
    pub years: u32,
    pub annual_growth_pct: f64,
    /// Marginal income-tax rate — the deduction value and the taxable
    /// account's haircut on contributions.
    pub marginal_tax_rate_pct: f64,
    /// Long-term cap-gains rate applied to the taxable account's gains.
    pub ltcg_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HsaResult {
    pub total_contributions_usd: f64,
    /// Upfront tax saved by deducting the contributions.
    pub upfront_tax_savings_usd: f64,
    /// HSA ending value (tax-free in and out).
    pub hsa_ending_usd: f64,
    /// Taxable account ending value, net of cap-gains tax on the gains.
    pub taxable_ending_usd: f64,
    /// HSA − taxable: the dollar value of the triple-tax treatment.
    pub hsa_advantage_usd: f64,
}

/// Future value of `n` end-of-year contributions of `c` growing at rate `r`.
fn annuity_fv(c: f64, r: f64, n: u32) -> f64 {
    if r.abs() < 1e-12 {
        c * n as f64
    } else {
        c * (((1.0 + r).powi(n as i32) - 1.0) / r)
    }
}

pub fn analyze(i: &HsaInput) -> HsaResult {
    let c = i.annual_contribution_usd.max(0.0);
    let r = i.annual_growth_pct / 100.0;
    let marginal = i.marginal_tax_rate_pct / 100.0;
    let ltcg = i.ltcg_rate_pct / 100.0;
    let n = i.years;

    let total_contributions = c * n as f64;
    let upfront_tax_savings = total_contributions * marginal;

    // HSA: full pre-tax contribution, tax-free throughout.
    let hsa_ending = annuity_fv(c, r, n);

    // Taxable: the same gross dollars are taxed before investing.
    let taxable_contrib = c * (1.0 - marginal);
    let taxable_fv = annuity_fv(taxable_contrib, r, n);
    let taxable_basis = taxable_contrib * n as f64;
    let taxable_gains = (taxable_fv - taxable_basis).max(0.0);
    let taxable_ending = taxable_fv - taxable_gains * ltcg;

    HsaResult {
        total_contributions_usd: total_contributions,
        upfront_tax_savings_usd: upfront_tax_savings,
        hsa_ending_usd: hsa_ending,
        taxable_ending_usd: taxable_ending,
        hsa_advantage_usd: hsa_ending - taxable_ending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> HsaInput {
        HsaInput {
            annual_contribution_usd: 4_000.0,
            years: 20,
            annual_growth_pct: 7.0,
            marginal_tax_rate_pct: 24.0,
            ltcg_rate_pct: 15.0,
        }
    }

    #[test]
    fn total_contributions_and_upfront_savings() {
        let r = analyze(&base());
        assert!((r.total_contributions_usd - 80_000.0).abs() < 1e-6); // 4k × 20
        assert!((r.upfront_tax_savings_usd - 19_200.0).abs() < 1e-6); // 80k × 24%
    }

    #[test]
    fn no_growth_hsa_is_just_contributions() {
        let r = analyze(&HsaInput { annual_growth_pct: 0.0, ..base() });
        assert!((r.hsa_ending_usd - 80_000.0).abs() < 1e-6);
        // Taxable: 4k×(1−.24)=3040/yr × 20 = 60,800, no gains taxed.
        assert!((r.taxable_ending_usd - 60_800.0).abs() < 1e-6);
    }

    #[test]
    fn hsa_annuity_fv_with_growth() {
        // 4000 × ((1.07^20 − 1)/0.07).
        let r = analyze(&base());
        let expected = 4_000.0 * ((1.07_f64.powi(20) - 1.0) / 0.07);
        assert!((r.hsa_ending_usd - expected).abs() < 1e-3);
    }

    #[test]
    fn hsa_beats_taxable() {
        let r = analyze(&base());
        assert!(r.hsa_advantage_usd > 0.0);
        assert!(r.hsa_ending_usd > r.taxable_ending_usd);
    }

    #[test]
    fn taxable_gains_are_taxed_at_end() {
        let r = analyze(&base());
        let taxable_contrib = 4_000.0 * 0.76;
        let taxable_fv = taxable_contrib * ((1.07_f64.powi(20) - 1.0) / 0.07);
        let gains = taxable_fv - taxable_contrib * 20.0;
        let expected = taxable_fv - gains * 0.15;
        assert!((r.taxable_ending_usd - expected).abs() < 1e-3);
    }

    #[test]
    fn higher_marginal_rate_widens_advantage() {
        let low = analyze(&HsaInput { marginal_tax_rate_pct: 12.0, ..base() });
        let high = analyze(&HsaInput { marginal_tax_rate_pct: 37.0, ..base() });
        assert!(high.hsa_advantage_usd > low.hsa_advantage_usd);
    }

    #[test]
    fn upfront_savings_scale_with_marginal() {
        let r = analyze(&HsaInput { marginal_tax_rate_pct: 37.0, ..base() });
        assert!((r.upfront_tax_savings_usd - 80_000.0 * 0.37).abs() < 1e-6);
    }

    #[test]
    fn zero_contribution_zero_everything() {
        let r = analyze(&HsaInput { annual_contribution_usd: 0.0, ..base() });
        assert!(r.hsa_ending_usd.abs() < 1e-9);
        assert!(r.taxable_ending_usd.abs() < 1e-9);
        assert!(r.hsa_advantage_usd.abs() < 1e-9);
    }
}
