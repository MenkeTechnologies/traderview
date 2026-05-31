//! IRS Form 8606 — Nondeductible IRAs + §408(d)(2) pro-rata rule.
//!
//! High-income traders (above the Roth IRA phase-out: $161k single /
//! $240k MFJ for 2024 contribution-eligibility) use the "backdoor
//! Roth" — contribute to a traditional IRA nondeductibly, then
//! convert to Roth. The conversion is supposed to be tax-free since
//! basis equals contribution. **Pro-rata rule blows this up when the
//! taxpayer has ANY pre-tax IRA balance**: under §408(d)(2), every
//! distribution and every conversion is taxed pro-rata across the
//! full IRA aggregate. A user with $10,000 of pre-tax SEP-IRA plus
//! a $7,000 nondeductible traditional IRA contribution doing a
//! $7,000 Roth conversion gets taxed on $7,000 × ($10k / $17k) =
//! $4,118 — **not zero**.
//!
//! Form 8606 is the IRS's official basis tracker for this. Each year:
//!
//!   * Line 1: nondeductible contributions made this year for THIS year.
//!   * Line 2: prior years' basis (= prior year's line 14).
//!   * Line 3: total basis available (line 1 + line 2).
//!   * Line 4-5: contribution timing carve-outs (handled by caller).
//!   * Line 6: year-end value of ALL traditional, SEP, SIMPLE IRAs.
//!   * Line 7: distributions taken this year.
//!   * Line 8: conversions to Roth this year.
//!   * Line 9: line 6 + 7 + 8 (the denominator).
//!   * Line 10: basis / line 9 (the proration ratio).
//!   * Line 11: line 8 × line 10 (nontaxable conversion portion).
//!   * Line 12: line 7 × line 10 (nontaxable distribution portion).
//!   * Line 13: line 11 + line 12 (total nontaxable).
//!   * Line 14: basis carryover = line 3 − line 13.
//!   * Line 15c: taxable distribution = line 7 − line 12.
//!   * Line 16: amount converted = line 8 (echoed).
//!   * Line 17: nontaxable conversion = line 11 (echoed).
//!   * Line 18: taxable conversion = line 16 − line 17.
//!
//! Pure compute. The DB ledger persistence lives in 0035 + a small DAL.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form8606Input {
    pub tax_year: i32,
    /// Line 1: nondeductible contributions made this year FOR this year.
    pub nondeductible_contributions: Decimal,
    /// Line 2: prior years' basis carryover (last year's line 14).
    pub prior_basis: Decimal,
    /// Line 6: year-end aggregate value of ALL traditional / SEP /
    /// SIMPLE IRAs (the pro-rata denominator component).
    pub year_end_aggregate_value: Decimal,
    /// Line 7: distributions taken from any traditional IRA this year.
    pub distributions_this_year: Decimal,
    /// Line 8: amount converted from traditional to Roth this year.
    pub conversions_to_roth: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Form8606Result {
    pub tax_year: i32,
    pub line_3_total_basis_available: Decimal,
    pub line_9_proration_denominator: Decimal,
    pub line_10_proration_ratio: Decimal, // basis / denominator, rounded to 5 dp
    pub line_11_nontaxable_conversion: Decimal,
    pub line_12_nontaxable_distribution: Decimal,
    pub line_13_total_nontaxable: Decimal,
    pub line_14_basis_carryover: Decimal,
    pub line_15c_taxable_distribution: Decimal,
    pub line_18_taxable_conversion: Decimal,
    pub total_taxable: Decimal,
    pub note: String,
}

pub fn compute(input: &Form8606Input) -> Form8606Result {
    let mut r = Form8606Result {
        tax_year: input.tax_year,
        ..Form8606Result::default()
    };

    r.line_3_total_basis_available = input.prior_basis + input.nondeductible_contributions;
    r.line_9_proration_denominator =
        input.year_end_aggregate_value + input.distributions_this_year + input.conversions_to_roth;

    if r.line_9_proration_denominator <= Decimal::ZERO {
        // No IRA activity at all. Basis just rolls forward. Differentiate
        // whether the user added a new nondeductible contribution this
        // year so the UI knows whether to congratulate or yawn.
        r.line_14_basis_carryover = r.line_3_total_basis_available;
        r.note = if input.nondeductible_contributions > Decimal::ZERO {
            "nondeductible contribution added to basis; no current-year tax event".into()
        } else {
            "no distributions / conversions / year-end balance — basis rolls forward".into()
        };
        return r;
    }

    // Proration ratio = basis / (year-end + distributions + conversions).
    // Capped at 1.0 — when basis exceeds the denominator (rare but
    // possible after a big loss), the IRS effectively treats everything
    // as basis return.
    let ratio = (r.line_3_total_basis_available / r.line_9_proration_denominator)
        .round_dp(5)
        .min(Decimal::ONE);
    r.line_10_proration_ratio = ratio;

    r.line_11_nontaxable_conversion = (input.conversions_to_roth * ratio).round_dp(2);
    r.line_12_nontaxable_distribution = (input.distributions_this_year * ratio).round_dp(2);
    r.line_13_total_nontaxable = r.line_11_nontaxable_conversion + r.line_12_nontaxable_distribution;

    r.line_15c_taxable_distribution =
        (input.distributions_this_year - r.line_12_nontaxable_distribution).max(Decimal::ZERO);
    r.line_18_taxable_conversion =
        (input.conversions_to_roth - r.line_11_nontaxable_conversion).max(Decimal::ZERO);
    r.total_taxable = r.line_15c_taxable_distribution + r.line_18_taxable_conversion;

    r.line_14_basis_carryover =
        (r.line_3_total_basis_available - r.line_13_total_nontaxable).max(Decimal::ZERO);

    // Annotate the pro-rata damage when relevant.
    let pretax_balance = (r.line_9_proration_denominator - r.line_3_total_basis_available)
        .max(Decimal::ZERO);
    r.note = if input.conversions_to_roth > Decimal::ZERO && pretax_balance > Decimal::ZERO {
        format!(
            "pro-rata: ${} of ${} conversion taxable (pre-tax balance ${} blends with basis ${})",
            r.line_18_taxable_conversion, input.conversions_to_roth,
            pretax_balance, r.line_3_total_basis_available,
        )
    } else if input.conversions_to_roth > Decimal::ZERO {
        "clean backdoor: no pre-tax balance, conversion entirely nontaxable".into()
    } else if input.distributions_this_year > Decimal::ZERO {
        format!(
            "distribution: ${} taxable (basis ${} apportioned)",
            r.line_15c_taxable_distribution, r.line_3_total_basis_available,
        )
    } else {
        "nondeductible contribution added to basis; no current-year tax event".into()
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Form8606Input {
        Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: dec!(7000),
            prior_basis: Decimal::ZERO,
            year_end_aggregate_value: Decimal::ZERO,
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: dec!(7000),
        }
    }

    #[test]
    fn clean_backdoor_no_pretax_balance_zero_tax() {
        // Classic backdoor: $7k nondeductible contribution, $7k conversion,
        // year-end balance $0, no pre-tax money. Tax = $0.
        let r = compute(&base());
        assert_eq!(r.line_3_total_basis_available, dec!(7000));
        assert_eq!(r.line_9_proration_denominator, dec!(7000));
        assert_eq!(r.line_10_proration_ratio, dec!(1.00000));
        assert_eq!(r.line_18_taxable_conversion, Decimal::ZERO);
        assert_eq!(r.line_14_basis_carryover, Decimal::ZERO);
        assert!(r.note.contains("clean"));
    }

    #[test]
    fn pro_rata_blows_up_backdoor_with_pretax_balance() {
        // $10k pre-tax SEP balance + $7k nondeductible + $7k conversion.
        // Denominator = $10k + $7k = $17k. Basis = $7k. Ratio = 7/17 ≈ 0.41176.
        // Nontaxable conversion = $7k × 0.41176 = $2,882.32.
        // Taxable conversion = $7k - $2,882.32 = $4,117.68.
        let mut i = base();
        i.year_end_aggregate_value = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.line_9_proration_denominator, dec!(17000));
        assert_eq!(r.line_10_proration_ratio, dec!(0.41176));
        assert_eq!(r.line_11_nontaxable_conversion, dec!(2882.32));
        assert_eq!(r.line_18_taxable_conversion, dec!(4117.68));
        // Basis carries forward = $7k - $2882.32 = $4117.68.
        assert_eq!(r.line_14_basis_carryover, dec!(4117.68));
        assert!(r.note.contains("pro-rata"));
    }

    #[test]
    fn prior_basis_carries_into_current_year() {
        // Last year: $4,117.68 basis carryover. This year: $7k contribution,
        // year-end zero, $7k conversion. Total basis available = $11,117.68.
        // Denominator = $7k. Ratio = $11,117.68 / $7k = 1.5882 → capped at 1.0.
        // All conversion nontaxable.
        let mut i = base();
        i.prior_basis = dec!(4117.68);
        let r = compute(&i);
        assert_eq!(r.line_3_total_basis_available, dec!(11117.68));
        assert_eq!(r.line_10_proration_ratio, Decimal::ONE);
        assert_eq!(r.line_18_taxable_conversion, Decimal::ZERO);
        // Basis carryover = $11,117.68 - $7,000 nontaxable = $4,117.68.
        assert_eq!(r.line_14_basis_carryover, dec!(4117.68));
    }

    #[test]
    fn distribution_only_no_conversion() {
        // $10k pre-tax + $5k basis + $3k distribution, no conversion.
        // Denominator = $10k + $3k = $13k. Ratio = 5/13 ≈ 0.38462.
        // Nontaxable distribution = $3k × 0.38462 = $1153.86.
        // Taxable distribution = $1846.14.
        let i = Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: Decimal::ZERO,
            prior_basis: dec!(5000),
            year_end_aggregate_value: dec!(10000),
            distributions_this_year: dec!(3000),
            conversions_to_roth: Decimal::ZERO,
        };
        let r = compute(&i);
        assert_eq!(r.line_10_proration_ratio, dec!(0.38462));
        assert_eq!(r.line_12_nontaxable_distribution, dec!(1153.86));
        assert_eq!(r.line_15c_taxable_distribution, dec!(1846.14));
        // Basis carryover = $5k - $1153.86 = $3846.14.
        assert_eq!(r.line_14_basis_carryover, dec!(3846.14));
    }

    #[test]
    fn mixed_distribution_and_conversion_both_pro_rated() {
        // $10k pre-tax + $7k basis (prior + this year), $3k distribution
        // + $4k conversion = $7k removed.
        // Denominator = $10k + $3k + $4k = $17k. Ratio = 7/17 ≈ 0.41176.
        // Nontaxable conversion = $4k × 0.41176 = $1647.04.
        // Nontaxable distribution = $3k × 0.41176 = $1235.28.
        // Total nontaxable = $2882.32.
        // Basis carryover = $7k - $2882.32 = $4117.68.
        let i = Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: dec!(7000),
            prior_basis: Decimal::ZERO,
            year_end_aggregate_value: dec!(10000),
            distributions_this_year: dec!(3000),
            conversions_to_roth: dec!(4000),
        };
        let r = compute(&i);
        assert_eq!(r.line_10_proration_ratio, dec!(0.41176));
        assert_eq!(r.line_13_total_nontaxable, dec!(2882.32));
        assert_eq!(r.line_14_basis_carryover, dec!(4117.68));
    }

    #[test]
    fn nondeductible_contribution_only_no_event() {
        // Just contribute, don't convert, don't distribute. Basis accumulates.
        let mut i = base();
        i.conversions_to_roth = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.line_14_basis_carryover, dec!(7000));
        assert_eq!(r.total_taxable, Decimal::ZERO);
        assert!(r.note.contains("nondeductible contribution"));
    }

    #[test]
    fn ratio_capped_at_one_when_basis_exceeds_denominator() {
        // Big prior basis, tiny everything else. Ratio would exceed 1.0
        // but we cap so the user doesn't over-claim nontaxable.
        let i = Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: Decimal::ZERO,
            prior_basis: dec!(50000),
            year_end_aggregate_value: Decimal::ZERO,
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: dec!(1000),
        };
        let r = compute(&i);
        assert_eq!(r.line_10_proration_ratio, Decimal::ONE);
        assert_eq!(r.line_18_taxable_conversion, Decimal::ZERO);
        assert_eq!(r.line_14_basis_carryover, dec!(49000));
    }

    #[test]
    fn empty_year_basis_rolls_forward() {
        // Nothing happened. Basis just carries forward.
        let i = Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: Decimal::ZERO,
            prior_basis: dec!(5000),
            year_end_aggregate_value: Decimal::ZERO,
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: Decimal::ZERO,
        };
        let r = compute(&i);
        assert_eq!(r.line_14_basis_carryover, dec!(5000));
        assert_eq!(r.total_taxable, Decimal::ZERO);
        assert!(r.note.contains("rolls forward"));
    }

    #[test]
    fn full_conversion_with_year_end_zero_clean_path() {
        // Year-end balance zero, conversion = basis. Ratio = 1.0.
        let mut i = base();
        i.nondeductible_contributions = dec!(6500);
        i.conversions_to_roth = dec!(6500);
        let r = compute(&i);
        assert_eq!(r.line_18_taxable_conversion, Decimal::ZERO);
    }

    #[test]
    fn pro_rata_50_50_balance_gives_50pct_taxable() {
        // $10k pre-tax + $10k nondeductible (no conversion this year).
        // Now convert $10k. Denominator = $10k (year-end) + $10k = $20k.
        // Ratio = 10/20 = 0.5. Half of conversion taxable.
        let i = Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: Decimal::ZERO,
            prior_basis: dec!(10000),
            year_end_aggregate_value: dec!(10000),
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: dec!(10000),
        };
        let r = compute(&i);
        assert_eq!(r.line_10_proration_ratio, dec!(0.50000));
        assert_eq!(r.line_18_taxable_conversion, dec!(5000));
    }

    #[test]
    fn multi_year_chain_basis_preserved() {
        // Y1: $7k nondeductible, no conversion → basis $7k.
        // Y2: $7k nondeductible (basis $14k), no conversion.
        // Y3: $14k conversion, year-end $0. Ratio = 14/14 = 1.0. Tax = $0.
        let y1 = compute(&Form8606Input {
            tax_year: 2022,
            nondeductible_contributions: dec!(7000),
            prior_basis: Decimal::ZERO,
            year_end_aggregate_value: dec!(7000),
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: Decimal::ZERO,
        });
        assert_eq!(y1.line_14_basis_carryover, dec!(7000));

        let y2 = compute(&Form8606Input {
            tax_year: 2023,
            nondeductible_contributions: dec!(7000),
            prior_basis: y1.line_14_basis_carryover,
            year_end_aggregate_value: dec!(14000),
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: Decimal::ZERO,
        });
        assert_eq!(y2.line_14_basis_carryover, dec!(14000));

        let y3 = compute(&Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: Decimal::ZERO,
            prior_basis: y2.line_14_basis_carryover,
            year_end_aggregate_value: Decimal::ZERO,
            distributions_this_year: Decimal::ZERO,
            conversions_to_roth: dec!(14000),
        });
        assert_eq!(y3.line_18_taxable_conversion, Decimal::ZERO);
        assert_eq!(y3.line_14_basis_carryover, Decimal::ZERO);
    }

    #[test]
    fn pro_rata_taxable_never_negative() {
        // Stress: weird inputs shouldn't produce negative numbers.
        let i = Form8606Input {
            tax_year: 2024,
            nondeductible_contributions: dec!(7000),
            prior_basis: dec!(1000000),
            year_end_aggregate_value: dec!(500),
            distributions_this_year: dec!(100),
            conversions_to_roth: dec!(200),
        };
        let r = compute(&i);
        assert!(r.line_18_taxable_conversion >= Decimal::ZERO);
        assert!(r.line_15c_taxable_distribution >= Decimal::ZERO);
        assert!(r.line_14_basis_carryover >= Decimal::ZERO);
    }

    #[test]
    fn note_distinguishes_clean_vs_prorated_backdoor() {
        let clean = compute(&base());
        assert!(clean.note.contains("clean"));

        let mut prorated = base();
        prorated.year_end_aggregate_value = dec!(10000);
        let p = compute(&prorated);
        assert!(p.note.contains("pro-rata"));
    }
}
