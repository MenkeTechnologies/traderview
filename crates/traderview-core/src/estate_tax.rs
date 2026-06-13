//! Federal estate tax — the tax on a taxable estate above the unified exclusion.
//!
//! Gross estate less deductible debts/expenses, the unlimited marital deduction,
//! and charitable bequests gives the taxable estate. Adjusted taxable gifts made
//! during life are added back (they already used exclusion), and the result above
//! the basic exclusion amount (plus any ported DSUE from a predeceased spouse) is
//! taxed at the top rate.
//!
//! 2026 defaults (web-verified, OBBBA-set, overridable): basic exclusion
//! $15,000,000 per person ($30M for a couple via portability), top rate 40%.
//!
//! A flat top-rate model is exact for any taxable estate: the graduated 18–40%
//! schedule applies only to the first ~$1M, which the multi-million exclusion
//! fully shelters, so the unified credit exactly offsets the tax on the exclusion
//! amount and the marginal portion is taxed at the top rate.

use serde::{Deserialize, Serialize};

fn d_exemption() -> f64 {
    15_000_000.0
}
fn d_rate() -> f64 {
    40.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct EstateTaxInput {
    pub gross_estate_usd: f64,
    /// Funeral costs, debts, administration expenses (deductible).
    #[serde(default)]
    pub debts_expenses_usd: f64,
    /// Amount passing to a surviving spouse (unlimited marital deduction).
    #[serde(default)]
    pub marital_deduction_usd: f64,
    /// Charitable bequests (deductible).
    #[serde(default)]
    pub charitable_deduction_usd: f64,
    /// Adjusted taxable gifts made during life (added to the tax base).
    #[serde(default)]
    pub lifetime_gifts_usd: f64,
    /// Basic exclusion amount.
    #[serde(default = "d_exemption")]
    pub exemption_usd: f64,
    /// Deceased spousal unused exclusion ported from a predeceased spouse.
    #[serde(default)]
    pub dsue_usd: f64,
    /// Top estate tax rate, percent.
    #[serde(default = "d_rate")]
    pub rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EstateTaxResult {
    /// Gross estate less debts, marital, and charitable deductions.
    pub taxable_estate_usd: f64,
    /// Exemption + ported DSUE.
    pub total_exemption_usd: f64,
    /// Taxable estate + lifetime gifts.
    pub estate_tax_base_usd: f64,
    /// Base above the total exemption (the portion taxed).
    pub amount_taxed_usd: f64,
    pub estate_tax_usd: f64,
    /// Taxable estate left for heirs after the tax.
    pub net_to_heirs_usd: f64,
    /// Exemption consumed by the base.
    pub exemption_used_usd: f64,
    /// Exemption left (relevant when under the threshold).
    pub exemption_remaining_usd: f64,
    /// Tax as a share of the taxable estate. None when the estate is zero.
    pub effective_rate_pct: Option<f64>,
    pub is_taxable: bool,
}

pub fn analyze(input: &EstateTaxInput) -> EstateTaxResult {
    let taxable_estate = (input.gross_estate_usd
        - input.debts_expenses_usd
        - input.marital_deduction_usd
        - input.charitable_deduction_usd)
        .max(0.0);

    let total_exemption = input.exemption_usd + input.dsue_usd;
    let base = taxable_estate + input.lifetime_gifts_usd;
    let amount_taxed = (base - total_exemption).max(0.0);
    let estate_tax = amount_taxed * input.rate_pct / 100.0;
    let net_to_heirs = (taxable_estate - estate_tax).max(0.0);

    EstateTaxResult {
        taxable_estate_usd: taxable_estate,
        total_exemption_usd: total_exemption,
        estate_tax_base_usd: base,
        amount_taxed_usd: amount_taxed,
        estate_tax_usd: estate_tax,
        net_to_heirs_usd: net_to_heirs,
        exemption_used_usd: base.min(total_exemption),
        exemption_remaining_usd: (total_exemption - base).max(0.0),
        effective_rate_pct: if taxable_estate > 0.0 {
            Some(estate_tax / taxable_estate * 100.0)
        } else {
            None
        },
        is_taxable: estate_tax > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> EstateTaxInput {
        EstateTaxInput {
            gross_estate_usd: 10_000_000.0,
            debts_expenses_usd: 0.0,
            marital_deduction_usd: 0.0,
            charitable_deduction_usd: 0.0,
            lifetime_gifts_usd: 0.0,
            exemption_usd: 15_000_000.0,
            dsue_usd: 0.0,
            rate_pct: 40.0,
        }
    }

    #[test]
    fn below_exemption_no_tax() {
        let r = analyze(&base());
        assert!(close(r.estate_tax_usd, 0.0));
        assert!(!r.is_taxable);
        assert!(close(r.exemption_remaining_usd, 5_000_000.0));
        assert!(close(r.net_to_heirs_usd, 10_000_000.0));
    }

    #[test]
    fn above_exemption_top_rate() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 20_000_000.0,
            ..base()
        });
        // (20M − 15M) × 40% = 2M.
        assert!(close(r.amount_taxed_usd, 5_000_000.0));
        assert!(close(r.estate_tax_usd, 2_000_000.0));
        assert!(close(r.net_to_heirs_usd, 18_000_000.0));
        assert!(close(r.exemption_remaining_usd, 0.0));
    }

    #[test]
    fn unlimited_marital_deduction() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 20_000_000.0,
            marital_deduction_usd: 20_000_000.0,
            ..base()
        });
        assert!(close(r.taxable_estate_usd, 0.0));
        assert!(close(r.estate_tax_usd, 0.0));
        assert!(r.effective_rate_pct.is_none());
    }

    #[test]
    fn debts_and_charity_reduce_base() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 20_000_000.0,
            debts_expenses_usd: 1_000_000.0,
            charitable_deduction_usd: 2_000_000.0,
            ..base()
        });
        // taxable 17M; (17M − 15M) × 40% = 800,000.
        assert!(close(r.taxable_estate_usd, 17_000_000.0));
        assert!(close(r.estate_tax_usd, 800_000.0));
    }

    #[test]
    fn lifetime_gifts_pull_over_exemption() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 14_000_000.0,
            lifetime_gifts_usd: 3_000_000.0,
            ..base()
        });
        // base 17M − 15M = 2M × 40% = 800,000.
        assert!(close(r.estate_tax_base_usd, 17_000_000.0));
        assert!(close(r.estate_tax_usd, 800_000.0));
    }

    #[test]
    fn dsue_portability_doubles_shelter() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 25_000_000.0,
            dsue_usd: 15_000_000.0,
            ..base()
        });
        assert!(close(r.total_exemption_usd, 30_000_000.0));
        assert!(close(r.estate_tax_usd, 0.0));
    }

    #[test]
    fn effective_rate_below_top_rate() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 20_000_000.0,
            ..base()
        });
        // 2M tax / 20M taxable = 10%, well below the 40% top rate.
        assert!(close(r.effective_rate_pct.unwrap(), 10.0));
    }

    #[test]
    fn custom_exemption_and_rate() {
        let r = analyze(&EstateTaxInput {
            gross_estate_usd: 5_000_000.0,
            exemption_usd: 1_000_000.0,
            rate_pct: 50.0,
            ..base()
        });
        // (5M − 1M) × 50% = 2M.
        assert!(close(r.estate_tax_usd, 2_000_000.0));
    }
}
