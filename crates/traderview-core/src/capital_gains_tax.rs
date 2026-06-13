//! Federal capital-gains tax on a single sale.
//!
//! Long-term gains use the preferential 0% / 15% / 20% brackets, which are
//! determined by *total* taxable income: the gain stacks on top of ordinary
//! taxable income and is taxed in slices as it crosses each threshold. Short-
//! term gains are ordinary income, taxed at the marginal rate the caller gives
//! (bracket modeling lives in `income_tax_estimator`).
//!
//! 2026 long-term thresholds (overridable): the 0% rate runs up to $49,450
//! (single) / $98,900 (married-joint) of total taxable income, and the 20%
//! rate begins above $545,500 / $613,700. The 15% rate fills the gap.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Term {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CapGainsInput {
    pub proceeds_usd: f64,
    pub cost_basis_usd: f64,
    pub term: Term,
    pub filing_status: FilingStatus,
    /// Ordinary taxable income the gain stacks on (for the long-term brackets).
    #[serde(default)]
    pub ordinary_taxable_income_usd: f64,
    /// Marginal ordinary rate for a short-term gain, percent.
    #[serde(default)]
    pub ordinary_rate_pct: f64,
    /// Override the 0%-rate ceiling (0 = use the built-in 2026 value).
    #[serde(default)]
    pub threshold_0_usd: f64,
    /// Override the 20%-rate floor (0 = use the built-in 2026 value).
    #[serde(default)]
    pub threshold_20_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CapGainsResult {
    /// proceeds − basis (negative for a loss).
    pub gain_usd: f64,
    pub is_long_term: bool,
    /// Long-term gain taxed at 0% / 15% / 20% (0 for a short-term gain).
    pub taxed_at_0_usd: f64,
    pub taxed_at_15_usd: f64,
    pub taxed_at_20_usd: f64,
    /// Short-term gain taxed at the ordinary rate (0 for a long-term gain).
    pub taxed_at_ordinary_usd: f64,
    pub tax_usd: f64,
    /// gain − tax.
    pub after_tax_gain_usd: f64,
    /// tax / gain, percent (0 when there is no positive gain).
    pub effective_rate_pct: f64,
}

fn default_thresholds(status: FilingStatus) -> (f64, f64) {
    match status {
        FilingStatus::Single => (49_450.0, 545_500.0),
        FilingStatus::MarriedJoint => (98_900.0, 613_700.0),
    }
}

pub fn analyze(input: &CapGainsInput) -> CapGainsResult {
    let gain = input.proceeds_usd - input.cost_basis_usd;

    let mut taxed_0 = 0.0;
    let mut taxed_15 = 0.0;
    let mut taxed_20 = 0.0;
    let mut taxed_ordinary = 0.0;
    let mut tax = 0.0;

    if gain > 0.0 {
        match input.term {
            Term::Short => {
                taxed_ordinary = gain;
                tax = gain * input.ordinary_rate_pct / 100.0;
            }
            Term::Long => {
                let (def0, def20) = default_thresholds(input.filing_status);
                let t0 = if input.threshold_0_usd > 0.0 {
                    input.threshold_0_usd
                } else {
                    def0
                };
                let t20 = if input.threshold_20_usd > 0.0 {
                    input.threshold_20_usd
                } else {
                    def20
                };

                // The gain occupies [start, end] of total taxable income.
                let start = input.ordinary_taxable_income_usd;
                let end = start + gain;

                taxed_0 = (t0.min(end) - start).max(0.0);
                taxed_15 = (t20.min(end) - t0.max(start)).max(0.0);
                taxed_20 = (end - t20.max(start)).max(0.0);
                tax = 0.15 * taxed_15 + 0.20 * taxed_20;
            }
        }
    }

    CapGainsResult {
        gain_usd: gain,
        is_long_term: matches!(input.term, Term::Long),
        taxed_at_0_usd: taxed_0,
        taxed_at_15_usd: taxed_15,
        taxed_at_20_usd: taxed_20,
        taxed_at_ordinary_usd: taxed_ordinary,
        tax_usd: tax,
        after_tax_gain_usd: gain - tax,
        effective_rate_pct: if gain > 0.0 { tax / gain * 100.0 } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn lt(proceeds: f64, basis: f64, income: f64, status: FilingStatus) -> CapGainsResult {
        analyze(&CapGainsInput {
            proceeds_usd: proceeds,
            cost_basis_usd: basis,
            term: Term::Long,
            filing_status: status,
            ordinary_taxable_income_usd: income,
            ordinary_rate_pct: 0.0,
            threshold_0_usd: 0.0,
            threshold_20_usd: 0.0,
        })
    }

    #[test]
    fn gain_computed() {
        let r = lt(50_000.0, 30_000.0, 40_000.0, FilingStatus::Single);
        assert!(close(r.gain_usd, 20_000.0));
        assert!(r.is_long_term);
    }

    #[test]
    fn all_zero_bracket_when_low_income() {
        // Income 10k + 20k gain = 30k < 49,450 → all at 0%.
        let r = lt(20_000.0, 0.0, 10_000.0, FilingStatus::Single);
        assert!(close(r.taxed_at_0_usd, 20_000.0));
        assert!(close(r.tax_usd, 0.0));
    }

    #[test]
    fn spans_zero_and_fifteen() {
        // Income 40k, gain 20k: 9,450 at 0%, 10,550 at 15% → $1,582.50.
        let r = lt(20_000.0, 0.0, 40_000.0, FilingStatus::Single);
        assert!(close(r.taxed_at_0_usd, 9_450.0));
        assert!(close(r.taxed_at_15_usd, 10_550.0));
        assert!(close(r.taxed_at_20_usd, 0.0));
        assert!(close(r.tax_usd, 1_582.50));
    }

    #[test]
    fn spans_fifteen_and_twenty() {
        // Income 540k, gain 20k: 5,500 at 15% + 14,500 at 20%.
        let r = lt(20_000.0, 0.0, 540_000.0, FilingStatus::Single);
        assert!(close(r.taxed_at_0_usd, 0.0));
        assert!(close(r.taxed_at_15_usd, 5_500.0));
        assert!(close(r.taxed_at_20_usd, 14_500.0));
        assert!(close(r.tax_usd, 0.15 * 5_500.0 + 0.20 * 14_500.0));
    }

    #[test]
    fn married_joint_threshold() {
        // Income 90k + 20k gain = 110k: 8,900 at 0%, 11,100 at 15%.
        let r = lt(20_000.0, 0.0, 90_000.0, FilingStatus::MarriedJoint);
        assert!(close(r.taxed_at_0_usd, 8_900.0));
        assert!(close(r.taxed_at_15_usd, 11_100.0));
        assert!(close(r.tax_usd, 0.15 * 11_100.0));
    }

    #[test]
    fn short_term_is_ordinary() {
        let r = analyze(&CapGainsInput {
            proceeds_usd: 20_000.0,
            cost_basis_usd: 0.0,
            term: Term::Short,
            filing_status: FilingStatus::Single,
            ordinary_taxable_income_usd: 40_000.0,
            ordinary_rate_pct: 24.0,
            threshold_0_usd: 0.0,
            threshold_20_usd: 0.0,
        });
        assert!(!r.is_long_term);
        assert!(close(r.taxed_at_ordinary_usd, 20_000.0));
        assert!(close(r.tax_usd, 4_800.0));
        assert!(close(r.taxed_at_15_usd, 0.0));
    }

    #[test]
    fn loss_has_no_tax() {
        let r = lt(10_000.0, 25_000.0, 40_000.0, FilingStatus::Single);
        assert!(close(r.gain_usd, -15_000.0));
        assert!(close(r.tax_usd, 0.0));
        assert!(close(r.effective_rate_pct, 0.0));
    }

    #[test]
    fn effective_rate_and_after_tax() {
        let r = lt(20_000.0, 0.0, 40_000.0, FilingStatus::Single);
        assert!(close(r.effective_rate_pct, 1_582.50 / 20_000.0 * 100.0));
        assert!(close(r.after_tax_gain_usd, 20_000.0 - 1_582.50));
    }
}
