//! IRC §1374 — Built-in gains (BIG) tax for S corporations.
//!
//! Triggered when a C corporation elects S corporation status: the
//! built-in appreciation of its assets at conversion date remains
//! exposed to corporate-level taxation for a **5-year recognition
//! period** (§1374(d)(7), as permanently set by the PATH Act of
//! 2015). The Tax Cuts and Jobs Act of 2017 set the §11(b) highest
//! corporate rate at a flat **21%**, which §1374(b)(1) cross-references
//! as the BIG tax rate.
//!
//! **Without §1374, a converted S-corp could escape corporate tax on
//! all pre-conversion appreciation by simply electing S status and
//! distributing gain to shareholders at lower individual rates** —
//! §1374 closes that exact loophole for the 5-year window.
//!
//! **NUBIG (Net Unrealized Built-In Gain) at conversion** is the
//! LIFETIME CEILING on what can ever be taxed:
//!
//! ```text
//! NUBIG = Σ(FMV − adjusted basis) at conversion
//!       − liabilities and deductible items at conversion
//! ```
//!
//! No matter how much built-in gain is recognized during the 5-year
//! window, the cumulative tax base under §1374 never exceeds the
//! conversion-date NUBIG.
//!
//! **NRBIG (Net Recognized Built-In Gain)** for each year of the
//! recognition period (§1374(d)(2)) = **LESSER OF**:
//!
//! 1. **Recognized BIG limit**: (recognized BIG − recognized BIL) for
//!    the year
//! 2. **Taxable income limit (§1374(d)(2)(A)(ii))**: corporation's
//!    taxable income computed as if it were a C-corp under
//!    §1375(b)(1)(B)
//! 3. **NUBIG ceiling**: conversion-date NUBIG MINUS cumulative
//!    prior-year NRBIG
//!
//! **C-corp NOL carryforward deductible** (§1374(b)(2)): NOL
//! carryforwards from C-corp years are allowed as a deduction against
//! NRBIG. Other C-corp tax attributes (general business credits,
//! minimum tax credit, etc.) can also offset BIG tax liability under
//! §1374(b)(3) — surfaced via the `c_corp_credit_offset` field for
//! caller-side computation.
//!
//! **NRBIG carryforward when TI binds**: if the taxable-income limit
//! caps NRBIG below the recognized BIG amount, the excess is
//! CARRIED FORWARD within the recognition period (§1374(d)(2)(B)) and
//! treated as recognized BIG in subsequent years. Returned as
//! `nrbig_carryforward_for_subsequent_year` so callers can chain.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1374Input {
    /// 0 = first S-corp year, 4 = fifth (last) year of recognition.
    /// Values ≥ 5 are outside the recognition period.
    pub year_of_recognition_period: u32,
    /// NUBIG calculated as of the conversion date.
    pub nubig_at_conversion: Decimal,
    pub recognized_big_this_year: Decimal,
    pub recognized_bil_this_year: Decimal,
    /// Taxable income computed as if the corporation were a C-corp,
    /// per §1375(b)(1)(B).
    pub taxable_income_as_c_corp: Decimal,
    /// Cumulative NRBIG already taxed in prior years of the
    /// recognition period (counts against the NUBIG ceiling).
    pub cumulative_prior_nrbig: Decimal,
    /// Pre-conversion C-corp NOL carryforward available to deduct
    /// against NRBIG (§1374(b)(2)).
    pub c_corp_nol_carryforward: Decimal,
    /// Any §1374(b)(3) credit carryforwards from C-corp years that
    /// directly reduce BIG tax liability dollar-for-dollar.
    pub c_corp_credit_offset: Decimal,
    /// Highest §11(b) corporate rate in basis points (2100 = 21.00%
    /// post-TCJA).
    pub corporate_tax_rate_bp: u32,
    /// NRBIG carried forward from a prior year because the TI limit
    /// bound there (§1374(d)(2)(B)).
    pub nrbig_carryforward_from_prior_year: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1374Result {
    pub in_recognition_period: bool,
    pub recognized_big_limit: Decimal,
    pub taxable_income_limit: Decimal,
    pub nubig_ceiling_remaining: Decimal,
    pub net_recognized_big: Decimal,
    pub binding_limit: BindingLimit,
    pub c_corp_nol_applied: Decimal,
    pub big_tax_before_credits: Decimal,
    pub big_tax_liability_after_credits: Decimal,
    pub nrbig_carryforward_for_subsequent_year: Decimal,
    pub citation: String,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingLimit {
    RecognizedBigLimit,
    TaxableIncomeLimit,
    NubigCeiling,
    OutsideRecognitionPeriod,
}

pub fn compute(input: &Section1374Input) -> Section1374Result {
    // PATH Act 2015 permanently set the recognition period to 5 years.
    let in_period = input.year_of_recognition_period < 5;
    if !in_period {
        return Section1374Result {
            in_recognition_period: false,
            recognized_big_limit: Decimal::ZERO,
            taxable_income_limit: Decimal::ZERO,
            nubig_ceiling_remaining: (input.nubig_at_conversion
                - input.cumulative_prior_nrbig)
                .max(Decimal::ZERO),
            net_recognized_big: Decimal::ZERO,
            binding_limit: BindingLimit::OutsideRecognitionPeriod,
            c_corp_nol_applied: Decimal::ZERO,
            big_tax_before_credits: Decimal::ZERO,
            big_tax_liability_after_credits: Decimal::ZERO,
            nrbig_carryforward_for_subsequent_year: Decimal::ZERO,
            citation: "IRC §1374(d)(7) (PATH Act 2015) — 5-year recognition period elapsed; no §1374 BIG tax exposure".to_string(),
            note: format!(
                "Year {} of post-conversion life is OUTSIDE the §1374(d)(7) 5-year recognition period. No BIG tax. Pre-conversion appreciation no longer exposed.",
                input.year_of_recognition_period + 1,
            ),
        };
    }

    // Three limits on NRBIG.
    let recognized_big_limit = (input.recognized_big_this_year
        - input.recognized_bil_this_year
        + input.nrbig_carryforward_from_prior_year)
        .max(Decimal::ZERO);
    let taxable_income_limit = input.taxable_income_as_c_corp.max(Decimal::ZERO);
    let nubig_ceiling_remaining =
        (input.nubig_at_conversion - input.cumulative_prior_nrbig).max(Decimal::ZERO);

    // NRBIG = lesser of three limits (§1374(d)(2)).
    let nrbig = recognized_big_limit
        .min(taxable_income_limit)
        .min(nubig_ceiling_remaining);

    let binding_limit = if nrbig == nubig_ceiling_remaining
        && nubig_ceiling_remaining < recognized_big_limit
        && nubig_ceiling_remaining < taxable_income_limit
    {
        BindingLimit::NubigCeiling
    } else if nrbig == taxable_income_limit
        && taxable_income_limit < recognized_big_limit
    {
        BindingLimit::TaxableIncomeLimit
    } else {
        BindingLimit::RecognizedBigLimit
    };

    // §1374(b)(2): C-corp NOL deductible against NRBIG.
    let nol_applied = nrbig.min(input.c_corp_nol_carryforward);
    let nrbig_after_nol = (nrbig - nol_applied).max(Decimal::ZERO);

    // §1374(b)(1): tax = highest §11(b) rate × NRBIG (after NOL).
    let big_tax_before_credits = nrbig_after_nol
        * Decimal::from(input.corporate_tax_rate_bp)
        / Decimal::from(10_000);

    // §1374(b)(3): C-corp credit carryforwards offset tax liability.
    let big_tax_after_credits =
        (big_tax_before_credits - input.c_corp_credit_offset).max(Decimal::ZERO);

    // §1374(d)(2)(B): when TI limit binds, excess carries forward in
    // the recognition period.
    let nrbig_carryforward = if binding_limit == BindingLimit::TaxableIncomeLimit {
        (recognized_big_limit - taxable_income_limit).max(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };

    let note = format!(
        "§1374 year {} of 5: NRBIG = lesser of (rec BIG ${}, TI ${}, NUBIG ceiling ${}) = ${} (binding: {:?}). C-corp NOL applied: ${}. Tax = {}.{}% × ${} = ${} − ${} credits = ${} liability.{}",
        input.year_of_recognition_period + 1,
        recognized_big_limit.round_dp(2),
        taxable_income_limit.round_dp(2),
        nubig_ceiling_remaining.round_dp(2),
        nrbig.round_dp(2),
        binding_limit,
        nol_applied.round_dp(2),
        input.corporate_tax_rate_bp / 100,
        input.corporate_tax_rate_bp % 100,
        nrbig_after_nol.round_dp(2),
        big_tax_before_credits.round_dp(2),
        input.c_corp_credit_offset.round_dp(2),
        big_tax_after_credits.round_dp(2),
        if nrbig_carryforward > Decimal::ZERO {
            format!(
                " §1374(d)(2)(B) NRBIG carryforward to next year: ${}.",
                nrbig_carryforward.round_dp(2)
            )
        } else {
            String::new()
        },
    );

    Section1374Result {
        in_recognition_period: true,
        recognized_big_limit,
        taxable_income_limit,
        nubig_ceiling_remaining,
        net_recognized_big: nrbig,
        binding_limit,
        c_corp_nol_applied: nol_applied,
        big_tax_before_credits,
        big_tax_liability_after_credits: big_tax_after_credits,
        nrbig_carryforward_for_subsequent_year: nrbig_carryforward,
        citation: "IRC §1374(b)(1) tax = §11(b) rate × NRBIG; §1374(d)(2) NRBIG = lesser-of-three; §1374(d)(7) (PATH 2015) 5-year recognition period; §1374(b)(2) C-corp NOL deduction; §1374(b)(3) credit offset".to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section1374Input {
        Section1374Input {
            year_of_recognition_period: 1, // Year 2 of recognition
            nubig_at_conversion: dec!(5_000_000),
            recognized_big_this_year: dec!(1_000_000),
            recognized_bil_this_year: Decimal::ZERO,
            taxable_income_as_c_corp: dec!(2_000_000),
            cumulative_prior_nrbig: Decimal::ZERO,
            c_corp_nol_carryforward: Decimal::ZERO,
            c_corp_credit_offset: Decimal::ZERO,
            corporate_tax_rate_bp: 2100, // 21% post-TCJA
            nrbig_carryforward_from_prior_year: Decimal::ZERO,
        }
    }

    #[test]
    fn baseline_in_recognition_period_recognized_big_binds() {
        // Rec BIG $1M < TI $2M < NUBIG ceiling $5M → rec BIG binds.
        let r = compute(&base());
        assert!(r.in_recognition_period);
        assert_eq!(r.net_recognized_big, dec!(1_000_000));
        assert_eq!(r.binding_limit, BindingLimit::RecognizedBigLimit);
        // $1M × 21% = $210k tax.
        assert_eq!(r.big_tax_liability_after_credits, dec!(210_000));
    }

    #[test]
    fn taxable_income_limit_binds_excess_carries_forward() {
        // Rec BIG $3M > TI $500k → TI binds, $2.5M carries forward.
        let mut i = base();
        i.recognized_big_this_year = dec!(3_000_000);
        i.taxable_income_as_c_corp = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.net_recognized_big, dec!(500_000));
        assert_eq!(r.binding_limit, BindingLimit::TaxableIncomeLimit);
        assert_eq!(r.nrbig_carryforward_for_subsequent_year, dec!(2_500_000));
        // $500k × 21% = $105k.
        assert_eq!(r.big_tax_liability_after_credits, dec!(105_000));
    }

    #[test]
    fn nubig_ceiling_binds_when_prior_years_exhausted_most_of_it() {
        // NUBIG $5M, prior NRBIG $4.5M → only $500k remaining.
        // Rec BIG $1M, TI $1M → NUBIG ceiling $500k binds.
        let mut i = base();
        i.cumulative_prior_nrbig = dec!(4_500_000);
        i.recognized_big_this_year = dec!(1_000_000);
        i.taxable_income_as_c_corp = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.nubig_ceiling_remaining, dec!(500_000));
        assert_eq!(r.net_recognized_big, dec!(500_000));
        assert_eq!(r.binding_limit, BindingLimit::NubigCeiling);
    }

    #[test]
    fn outside_5_year_recognition_period_no_tax() {
        let mut i = base();
        i.year_of_recognition_period = 5; // Year 6 — outside
        let r = compute(&i);
        assert!(!r.in_recognition_period);
        assert_eq!(r.net_recognized_big, Decimal::ZERO);
        assert_eq!(r.big_tax_liability_after_credits, Decimal::ZERO);
        assert_eq!(r.binding_limit, BindingLimit::OutsideRecognitionPeriod);
    }

    #[test]
    fn year_5_exact_boundary_still_inside() {
        // year_of_recognition_period = 4 (year 5 = last year inside).
        let mut i = base();
        i.year_of_recognition_period = 4;
        let r = compute(&i);
        assert!(r.in_recognition_period);
    }

    #[test]
    fn recognized_bil_offsets_recognized_big() {
        // $1M BIG − $400k BIL = $600k net.
        let mut i = base();
        i.recognized_bil_this_year = dec!(400_000);
        let r = compute(&i);
        assert_eq!(r.recognized_big_limit, dec!(600_000));
        assert_eq!(r.net_recognized_big, dec!(600_000));
    }

    #[test]
    fn c_corp_nol_reduces_taxable_nrbig() {
        // NRBIG $1M − NOL $400k = $600k taxable → 21% × $600k = $126k.
        let mut i = base();
        i.c_corp_nol_carryforward = dec!(400_000);
        let r = compute(&i);
        assert_eq!(r.c_corp_nol_applied, dec!(400_000));
        assert_eq!(r.big_tax_before_credits, dec!(126_000));
    }

    #[test]
    fn c_corp_nol_exceeds_nrbig_clamps_to_zero_tax() {
        // NOL $5M, NRBIG $1M → all $1M absorbed, $0 tax.
        let mut i = base();
        i.c_corp_nol_carryforward = dec!(5_000_000);
        let r = compute(&i);
        assert_eq!(r.c_corp_nol_applied, dec!(1_000_000));
        assert_eq!(r.big_tax_before_credits, Decimal::ZERO);
    }

    #[test]
    fn credit_offset_reduces_tax_dollar_for_dollar() {
        // $210k tax − $50k credits = $160k after-credits.
        let mut i = base();
        i.c_corp_credit_offset = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.big_tax_before_credits, dec!(210_000));
        assert_eq!(r.big_tax_liability_after_credits, dec!(160_000));
    }

    #[test]
    fn credit_offset_exceeds_tax_clamps_to_zero() {
        let mut i = base();
        i.c_corp_credit_offset = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.big_tax_liability_after_credits, Decimal::ZERO);
    }

    #[test]
    fn nrbig_carryforward_from_prior_year_stacks() {
        // Prior carryforward $1M + this year $500k = $1.5M rec BIG limit.
        let mut i = base();
        i.recognized_big_this_year = dec!(500_000);
        i.nrbig_carryforward_from_prior_year = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.recognized_big_limit, dec!(1_500_000));
    }

    #[test]
    fn zero_nubig_no_tax_exposure() {
        // No appreciation at conversion → NUBIG = 0 → ceiling binds at 0.
        let mut i = base();
        i.nubig_at_conversion = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.net_recognized_big, Decimal::ZERO);
        assert_eq!(r.big_tax_liability_after_credits, Decimal::ZERO);
        assert_eq!(r.binding_limit, BindingLimit::NubigCeiling);
    }

    #[test]
    fn negative_recognized_big_clamped_to_zero() {
        // BIL > BIG → recognized BIG limit clamps to 0.
        let mut i = base();
        i.recognized_big_this_year = dec!(200_000);
        i.recognized_bil_this_year = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.recognized_big_limit, Decimal::ZERO);
        assert_eq!(r.net_recognized_big, Decimal::ZERO);
    }

    #[test]
    fn pre_tcja_35_pct_rate_path() {
        // Pre-TCJA highest C-corp rate was 35%. Module should respect input rate.
        let mut i = base();
        i.corporate_tax_rate_bp = 3500;
        let r = compute(&i);
        // $1M × 35% = $350k.
        assert_eq!(r.big_tax_before_credits, dec!(350_000));
    }

    #[test]
    fn zero_taxable_income_no_tax_full_carryforward() {
        // TI = $0 → no §1374 tax this year; entire recognized BIG carries forward.
        let mut i = base();
        i.taxable_income_as_c_corp = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.net_recognized_big, Decimal::ZERO);
        assert_eq!(r.big_tax_liability_after_credits, Decimal::ZERO);
        assert_eq!(r.nrbig_carryforward_for_subsequent_year, dec!(1_000_000));
    }

    #[test]
    fn nubig_ceiling_remaining_clamps_at_zero() {
        // Prior NRBIG $7M but NUBIG only $5M → ceiling clamps at 0, not negative.
        let mut i = base();
        i.cumulative_prior_nrbig = dec!(7_000_000);
        let r = compute(&i);
        assert_eq!(r.nubig_ceiling_remaining, Decimal::ZERO);
        assert_eq!(r.net_recognized_big, Decimal::ZERO);
    }

    #[test]
    fn note_describes_recognized_big_binding() {
        let r = compute(&base());
        assert!(r.note.contains("§1374 year 2 of 5"));
        assert!(r.note.contains("RecognizedBigLimit"));
    }

    #[test]
    fn note_describes_ti_binding_with_carryforward() {
        let mut i = base();
        i.recognized_big_this_year = dec!(3_000_000);
        i.taxable_income_as_c_corp = dec!(500_000);
        let r = compute(&i);
        assert!(r.note.contains("TaxableIncomeLimit"));
        assert!(r.note.contains("carryforward to next year"));
    }

    #[test]
    fn note_outside_period_describes_no_exposure() {
        let mut i = base();
        i.year_of_recognition_period = 5;
        let r = compute(&i);
        assert!(r.note.contains("OUTSIDE"));
        assert!(r.note.contains("recognition period"));
    }

    #[test]
    fn very_large_nubig_precision_path() {
        // $500M NUBIG, $100M recognized BIG, TI $200M.
        let mut i = base();
        i.nubig_at_conversion = dec!(500_000_000);
        i.recognized_big_this_year = dec!(100_000_000);
        i.taxable_income_as_c_corp = dec!(200_000_000);
        let r = compute(&i);
        assert_eq!(r.net_recognized_big, dec!(100_000_000));
        // $100M × 21% = $21M.
        assert_eq!(r.big_tax_liability_after_credits, dec!(21_000_000));
    }

    #[test]
    fn all_three_limits_at_same_value_recognized_binds_first() {
        // All limits exactly $1M. Binding-limit precedence: the test
        // documents the precedence used by the module
        // (RecognizedBigLimit wins ties).
        let mut i = base();
        i.recognized_big_this_year = dec!(1_000_000);
        i.taxable_income_as_c_corp = dec!(1_000_000);
        i.nubig_at_conversion = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.net_recognized_big, dec!(1_000_000));
        assert_eq!(r.binding_limit, BindingLimit::RecognizedBigLimit);
    }

    #[test]
    fn nol_does_not_create_negative_tax() {
        // NOL > NRBIG → tax = 0, not negative.
        let mut i = base();
        i.c_corp_nol_carryforward = dec!(10_000_000);
        i.recognized_big_this_year = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.big_tax_before_credits, Decimal::ZERO);
        assert_eq!(r.big_tax_liability_after_credits, Decimal::ZERO);
    }
}
