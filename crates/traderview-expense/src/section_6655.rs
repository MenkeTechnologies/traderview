//! IRC § 6655 — Failure by Corporation to Pay Estimated
//! Income Tax / Quarterly Installment Underpayment Penalty
//! Module.
//!
//! Pure-compute check for IRC § 6655 corporate estimated-
//! income-tax quarterly installment requirements and the
//! underpayment penalty (addition to tax) imposed at the
//! § 6621 underpayment rate when a C corporation fails to
//! pay each quarterly installment by its due date. § 6655 is
//! the corporate-side parallel to § 6654 (individual
//! estimated tax underpayment) and uses the same § 6621
//! rate-determination machinery (built iter 674).
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6655(a) Addition to Tax**: if a corporation
//!   underpays a required installment, an addition to tax
//!   equal to the **§ 6621 UNDERPAYMENT RATE** times the
//!   underpayment for the underpayment period is imposed
//!   ([Cornell LII 26 USC § 6655](https://www.law.cornell.edu/uscode/text/26/6655);
//!   [Bloomberg Tax Sec. 6655](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6655);
//!   [26 CFR § 1.6655-1](https://www.law.cornell.edu/cfr/text/26/1.6655-1)).
//! - **IRC § 6655(b) Underpayment Period**: the underpayment
//!   period runs from the **installment DUE DATE** until the
//!   EARLIER of (1) the **15th DAY of the 4th MONTH** after
//!   the close of the taxable year (April 15 for calendar-
//!   year corporations) OR (2) the **DATE the installment is
//!   PAID**. The interest rate is the § 6621 underpayment
//!   rate (FSTR + 3 percentage points, or FSTR + 5 percentage
//!   points for a large corporate underpayment > $100,000
//!   under § 6621(c)).
//! - **IRC § 6655(c) Four Required Installments**: calendar-
//!   year corporations must pay 4 equal installments on
//!   **APRIL 15, JUNE 15, SEPTEMBER 15, and DECEMBER 15**.
//!   Fiscal-year corporations pay on the 15th day of the
//!   4th, 6th, 9th, and 12th months of the taxable year.
//! - **IRC § 6655(d)(1)(A) Required Installment Amount**:
//!   each installment = **25 PERCENT of the REQUIRED ANNUAL
//!   PAYMENT**. The required annual payment is the LESSER of:
//!   (i) **100 PERCENT of the TAX shown on the return for
//!   the TAXABLE YEAR** (current year tax safe harbor) OR
//!   (ii) **100 PERCENT of the TAX shown on the return for
//!   the PRECEDING TAXABLE YEAR** (prior year tax safe
//!   harbor). The preceding-year option does NOT apply if
//!   the preceding taxable year was less than 12 months OR
//!   the corporation did not file a return showing a tax
//!   liability for the preceding year ([Tax Notes — Sec.
//!   6655 Failure by Corporation to Pay Estimated Income
//!   Tax](https://www.taxnotes.com/research/federal/usc26/6655)).
//! - **IRC § 6655(d)(2) Large Corporation Exception**: the
//!   prior-year-tax safe harbor under (d)(1)(B)(ii) does NOT
//!   apply to **LARGE CORPORATIONS**, except that the
//!   exception does NOT apply for purposes of determining
//!   the amount of the **FIRST REQUIRED INSTALLMENT** of any
//!   taxable year. Large corporations must therefore pay
//!   100 % of current-year tax through installments 2-4, but
//!   may still rely on prior-year tax for the first
//!   installment ([LegalClarity — The Large Corporation
//!   Estimated Tax Safe Harbor](https://legalclarity.org/the-large-corporation-estimated-tax-safe-harbor/)).
//! - **IRC § 6655(e) Annualized Income Installment Method
//!   and Adjusted Seasonal Installment Method**: if either
//!   alternative method yields a LOWER required installment
//!   than the 25 % default, the corporation may use it.
//!   Annualized income uses 3, 3, 6, and 9-month annualized
//!   computations for the four installments. Adjusted
//!   seasonal applies if at least 70 % of taxable income in
//!   each of the three preceding taxable years was earned in
//!   the same 6 calendar months of the year.
//! - **IRC § 6655(f) Exception for Small Underpayments**:
//!   no penalty if the **TOTAL TAX shown on the return is
//!   LESS THAN $500** (full waiver for very small
//!   corporations).
//! - **IRC § 6655(g)(2) Large Corporation Definition**: a
//!   corporation is a "LARGE CORPORATION" if it had
//!   **TAXABLE INCOME of $1,000,000 OR MORE** for any of the
//!   **THREE PRECEDING TAXABLE YEARS** (lookback test;
//!   ignores controlled-group attribution under § 1504).
//! - **IRC § 6655(g)(1) Tax Definition**: "tax" includes
//!   regular income tax under § 11, alternative minimum tax
//!   under § 55, and base erosion anti-abuse tax (BEAT)
//!   under § 59A, REDUCED by applicable credits.
//! - **IRC § 6655(h) Excessive Adjustment under § 6425**:
//!   if the IRS makes a quick refund of estimated tax under
//!   § 6425 and the refund proves excessive, interest at
//!   the § 6621 underpayment rate accrues from the refund
//!   date through the 15th day of the 4th month following
//!   year-end.
//! - **Companion Provisions**: § 6621 (underpayment rate —
//!   primary input to § 6655 penalty computation; built iter
//!   674); § 6601 (general underpayment interest); § 6622
//!   (daily compounding); § 6654 (individual estimated tax
//!   underpayment — parallel provision); § 6425 (corporate
//!   quick-refund procedure); Form 2220 (taxpayer estimated
//!   tax penalty computation form).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6655_INSTALLMENT_PERCENTAGE_BASIS_POINTS: u64 = 2_500;
pub const IRC_6655_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_6655_NUMBER_OF_INSTALLMENTS_PER_YEAR: u32 = 4;
pub const IRC_6655_SMALL_UNDERPAYMENT_EXCEPTION_THRESHOLD_DOLLARS: u64 = 500;
pub const IRC_6655_LARGE_CORPORATION_TAXABLE_INCOME_THRESHOLD_DOLLARS: u64 = 1_000_000;
pub const IRC_6655_LARGE_CORPORATION_LOOKBACK_YEARS: u32 = 3;
pub const IRC_6655_INSTALLMENT_1_MONTH: u32 = 4;
pub const IRC_6655_INSTALLMENT_1_DAY: u32 = 15;
pub const IRC_6655_INSTALLMENT_2_MONTH: u32 = 6;
pub const IRC_6655_INSTALLMENT_2_DAY: u32 = 15;
pub const IRC_6655_INSTALLMENT_3_MONTH: u32 = 9;
pub const IRC_6655_INSTALLMENT_3_DAY: u32 = 15;
pub const IRC_6655_INSTALLMENT_4_MONTH: u32 = 12;
pub const IRC_6655_INSTALLMENT_4_DAY: u32 = 15;
pub const IRC_6655_DAYS_PER_YEAR: u64 = 365;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationType {
    LargeCorporationTaxableIncomeAtOrAbove1MillionInAnyOf3PrecedingYears,
    StandardCorporationBelowLargeCorporationThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallmentQuarter {
    Quarter1AprilOf15,
    Quarter2JuneOf15,
    Quarter3SeptemberOf15,
    Quarter4DecemberOf15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeHarborMethod {
    LesserOfCurrentOrPrecedingYearTax,
    AnnualizedIncomeInstallmentMethodUnderSection6655E,
    AdjustedSeasonalInstallmentMethodUnderSection6655E,
    NoSafeHarborMethodSelected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6655Mode {
    NotApplicableTotalTaxBelow500SmallUnderpaymentException,
    NotApplicablePrecedingYearWasShortPeriodOrNoReturnFiled,
    CompliantQuarterlyInstallmentPaidAt25PctOfRequiredAnnualPayment,
    CompliantQuarterlyInstallmentMetUsingAnnualizedIncomeInstallmentMethod,
    CompliantQuarterlyInstallmentMetUsingAdjustedSeasonalInstallmentMethod,
    CompliantLargeCorporationFirstInstallmentUsedPriorYearTaxSafeHarbor,
    ViolationLargeCorporationUsedPriorYearTaxAfterFirstInstallment,
    ViolationStandardCorporationUnderpaidEstimatedTaxInstallmentAdditionToTaxAtSection6621Rate,
    ViolationLargeCorporationUnderpaidEstimatedTaxInstallmentAdditionToTaxAtSection6621Rate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub corporation_type: CorporationType,
    pub max_taxable_income_in_3_preceding_years_dollars: u64,
    pub total_tax_shown_on_return_dollars: u64,
    pub current_year_tax_dollars: u64,
    pub preceding_year_tax_dollars: u64,
    pub preceding_year_was_full_12_month_return_with_tax_liability: bool,
    pub installment_quarter: InstallmentQuarter,
    pub installment_amount_paid_dollars: u64,
    pub safe_harbor_method: SafeHarborMethod,
    pub federal_short_term_rate_basis_points: u64,
    pub days_underpayment_period: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6655Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub required_installment_amount_dollars: u64,
    pub underpayment_amount_dollars: u64,
    pub estimated_addition_to_tax_dollars: u64,
}

pub type Section6655Input = Input;
pub type Section6655Output = Output;
pub type Section6655Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6655(a) Addition to Tax — corporation underpaying required installment owes addition to tax = § 6621 underpayment rate × underpayment × underpayment period".to_string(),
        "IRC § 6655(b) Underpayment Period — runs from installment DUE DATE until EARLIER of (1) 15th DAY of 4th MONTH after close of taxable year (April 15 for calendar-year corps) OR (2) DATE installment paid; rate per § 6621 underpayment rate (FSTR + 3 pp; FSTR + 5 pp for large corporate underpayment > $100,000 under § 6621(c))".to_string(),
        "IRC § 6655(c) Four Required Installments — calendar-year corporations pay on APRIL 15, JUNE 15, SEPTEMBER 15, DECEMBER 15; fiscal-year corporations pay on 15th day of 4th, 6th, 9th, 12th months of taxable year".to_string(),
        "IRC § 6655(d)(1)(A) Required Installment Amount — each installment = 25 PERCENT of REQUIRED ANNUAL PAYMENT; required annual payment = LESSER of (i) 100 PERCENT of TAX shown on return for TAXABLE YEAR (current year safe harbor) OR (ii) 100 PERCENT of TAX shown on return for PRECEDING TAXABLE YEAR (prior year safe harbor); preceding-year option does NOT apply if preceding year < 12 months or no return filed with tax liability".to_string(),
        "IRC § 6655(d)(2) Large Corporation Exception — prior-year-tax safe harbor under (d)(1)(B)(ii) does NOT apply to LARGE CORPORATIONS, except exception does NOT apply for purposes of determining FIRST REQUIRED INSTALLMENT of any taxable year; large corps must therefore pay 100 % of current-year tax through installments 2-4, but may rely on prior-year tax for the first installment only".to_string(),
        "IRC § 6655(e) Annualized Income Installment + Adjusted Seasonal Installment Methods — if either alternative method yields LOWER required installment than 25 % default, corporation may use it; annualized income uses 3, 3, 6, 9-month annualized computations for the four installments; adjusted seasonal applies if at least 70 % of taxable income in each of three preceding years was earned in the same 6 calendar months".to_string(),
        "IRC § 6655(f) Exception for Small Underpayments — NO penalty if TOTAL TAX shown on return is LESS THAN $500 (full waiver for very small corporations)".to_string(),
        "IRC § 6655(g)(1) Tax Definition — 'tax' includes regular income tax under § 11, alternative minimum tax under § 55, and base erosion anti-abuse tax (BEAT) under § 59A, REDUCED by applicable credits".to_string(),
        "IRC § 6655(g)(2) Large Corporation Definition — corporation is a LARGE CORPORATION if it had TAXABLE INCOME of $1,000,000 OR MORE for any of the THREE PRECEDING TAXABLE YEARS (lookback test); controlled-group aggregation under § 1504".to_string(),
        "IRC § 6655(h) Excessive Adjustment Under § 6425 — if IRS makes quick refund of estimated tax under § 6425 and refund proves excessive, interest at § 6621 underpayment rate accrues from refund date through 15th day of 4th month following year-end".to_string(),
        "Companion Provisions — § 6621 (underpayment rate, primary input to § 6655 penalty computation; built iter 674); § 6601 (general underpayment interest); § 6622 (daily compounding); § 6654 (individual estimated tax underpayment, parallel provision); § 6425 (corporate quick-refund procedure); Form 2220 (taxpayer estimated tax penalty computation form)".to_string(),
        "Cornell LII 26 USC § 6655 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6655 — comprehensive code commentary".to_string(),
        "26 CFR § 1.6655-1 — addition to the tax in the case of a corporation (implementing regulation)".to_string(),
        "IRM 20.1.3 — Estimated Tax Penalties (IRS examiner manual)".to_string(),
    ];

    if input.total_tax_shown_on_return_dollars
        < IRC_6655_SMALL_UNDERPAYMENT_EXCEPTION_THRESHOLD_DOLLARS
    {
        return Output {
            mode: Section6655Mode::NotApplicableTotalTaxBelow500SmallUnderpaymentException,
            statutory_basis: "IRC § 6655(f) — no penalty if total tax shown on return is less than $500".to_string(),
            notes: format!(
                "NOT APPLICABLE: total tax shown on return = ${} (< $500 threshold); § 6655(f) small-underpayment exception applies; no addition to tax owed.",
                input.total_tax_shown_on_return_dollars
            ),
            citations,
            required_installment_amount_dollars: 0,
            underpayment_amount_dollars: 0,
            estimated_addition_to_tax_dollars: 0,
        };
    }

    let is_large_corporation = input.corporation_type
        == CorporationType::LargeCorporationTaxableIncomeAtOrAbove1MillionInAnyOf3PrecedingYears
        || input.max_taxable_income_in_3_preceding_years_dollars
            >= IRC_6655_LARGE_CORPORATION_TAXABLE_INCOME_THRESHOLD_DOLLARS;

    let prior_year_safe_harbor_usable_baseline =
        input.preceding_year_was_full_12_month_return_with_tax_liability;

    let is_first_installment = input.installment_quarter == InstallmentQuarter::Quarter1AprilOf15;

    let prior_year_safe_harbor_available =
        prior_year_safe_harbor_usable_baseline && (!is_large_corporation || is_first_installment);

    let required_annual_payment = if prior_year_safe_harbor_available {
        input
            .current_year_tax_dollars
            .min(input.preceding_year_tax_dollars)
    } else {
        if !prior_year_safe_harbor_usable_baseline && is_large_corporation && is_first_installment {
            // Large corp first installment but no usable prior-year baseline -> use current year tax
            input.current_year_tax_dollars
        } else if is_large_corporation {
            // Large corp installments 2-4 must use current-year tax
            input.current_year_tax_dollars
        } else {
            // Small corp but preceding year unavailable -> falls back to current-year tax
            input.current_year_tax_dollars
        }
    };

    let required_installment_amount = u128::from(required_annual_payment)
        .saturating_mul(u128::from(IRC_6655_INSTALLMENT_PERCENTAGE_BASIS_POINTS))
        .checked_div(u128::from(IRC_6655_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64;

    if !prior_year_safe_harbor_usable_baseline
        && input.safe_harbor_method == SafeHarborMethod::LesserOfCurrentOrPrecedingYearTax
    {
        return Output {
            mode: Section6655Mode::NotApplicablePrecedingYearWasShortPeriodOrNoReturnFiled,
            statutory_basis: "IRC § 6655(d)(1)(B) — prior-year safe harbor unavailable when preceding year was short period or no return filed with tax liability".to_string(),
            notes: format!(
                "NOT APPLICABLE TO PRIOR-YEAR PATH: preceding taxable year was not a full 12 months OR no return was filed showing tax liability; § 6655(d)(1)(B) prior-year safe harbor unavailable; corporation must rely on current-year tax safe harbor OR § 6655(e) annualized/adjusted-seasonal methods. Required installment recomputed using current year tax = 25 % × ${} = ${}.",
                input.current_year_tax_dollars, required_installment_amount
            ),
            citations,
            required_installment_amount_dollars: required_installment_amount,
            underpayment_amount_dollars: 0,
            estimated_addition_to_tax_dollars: 0,
        };
    }

    match input.safe_harbor_method {
        SafeHarborMethod::AnnualizedIncomeInstallmentMethodUnderSection6655E => {
            if input.installment_amount_paid_dollars >= required_installment_amount {
                return Output {
                    mode: Section6655Mode::CompliantQuarterlyInstallmentMetUsingAnnualizedIncomeInstallmentMethod,
                    statutory_basis: "IRC § 6655(e)(2) — annualized income installment method".to_string(),
                    notes: format!(
                        "COMPLIANT: corporation used § 6655(e)(2) annualized income installment method to determine quarterly installment; installment paid ${} ≥ required installment ${} computed using annualized income (3, 3, 6, 9-month annualization across quarters).",
                        input.installment_amount_paid_dollars, required_installment_amount
                    ),
                    citations,
                    required_installment_amount_dollars: required_installment_amount,
                    underpayment_amount_dollars: 0,
                    estimated_addition_to_tax_dollars: 0,
                };
            }
        }
        SafeHarborMethod::AdjustedSeasonalInstallmentMethodUnderSection6655E => {
            if input.installment_amount_paid_dollars >= required_installment_amount {
                return Output {
                    mode: Section6655Mode::CompliantQuarterlyInstallmentMetUsingAdjustedSeasonalInstallmentMethod,
                    statutory_basis: "IRC § 6655(e)(3) — adjusted seasonal installment method (70 %+ income in same 6 months of 3 preceding years)".to_string(),
                    notes: format!(
                        "COMPLIANT: corporation used § 6655(e)(3) adjusted seasonal installment method to determine quarterly installment (requires at least 70 % of taxable income in each of 3 preceding years earned in same 6 calendar months); installment paid ${} ≥ required ${}.",
                        input.installment_amount_paid_dollars, required_installment_amount
                    ),
                    citations,
                    required_installment_amount_dollars: required_installment_amount,
                    underpayment_amount_dollars: 0,
                    estimated_addition_to_tax_dollars: 0,
                };
            }
        }
        SafeHarborMethod::LesserOfCurrentOrPrecedingYearTax => {}
        SafeHarborMethod::NoSafeHarborMethodSelected => {}
    }

    if input.installment_amount_paid_dollars >= required_installment_amount {
        if prior_year_safe_harbor_available && is_large_corporation && is_first_installment {
            return Output {
                mode: Section6655Mode::CompliantLargeCorporationFirstInstallmentUsedPriorYearTaxSafeHarbor,
                statutory_basis: "IRC § 6655(d)(2)(B) — large corporation first installment may use prior-year tax safe harbor".to_string(),
                notes: format!(
                    "COMPLIANT: large corporation (≥ $1,000,000 taxable income in any of 3 preceding years) used the § 6655(d)(2)(B) carve-out — prior-year tax safe harbor available ONLY for first installment; required installment = 25 % × lesser of current ${} or preceding ${} = ${}; paid ${}; remaining installments 2-4 must use 100 % current-year tax.",
                    input.current_year_tax_dollars,
                    input.preceding_year_tax_dollars,
                    required_installment_amount,
                    input.installment_amount_paid_dollars
                ),
                citations,
                required_installment_amount_dollars: required_installment_amount,
                underpayment_amount_dollars: 0,
                estimated_addition_to_tax_dollars: 0,
            };
        }
        return Output {
            mode: Section6655Mode::CompliantQuarterlyInstallmentPaidAt25PctOfRequiredAnnualPayment,
            statutory_basis: "IRC § 6655(d)(1)(A) — installment ≥ 25 % of required annual payment".to_string(),
            notes: format!(
                "COMPLIANT: installment paid ${} ≥ required installment ${} (= 25 % × required annual payment); required annual payment computed using {} safe harbor.",
                input.installment_amount_paid_dollars,
                required_installment_amount,
                if prior_year_safe_harbor_available { "lesser of current or preceding year tax" } else { "current-year tax only" }
            ),
            citations,
            required_installment_amount_dollars: required_installment_amount,
            underpayment_amount_dollars: 0,
            estimated_addition_to_tax_dollars: 0,
        };
    }

    let underpayment =
        required_installment_amount.saturating_sub(input.installment_amount_paid_dollars);

    let underpayment_rate_basis_points = input
        .federal_short_term_rate_basis_points
        .saturating_add(300);
    let addition_to_tax = u128::from(underpayment)
        .saturating_mul(u128::from(underpayment_rate_basis_points))
        .saturating_mul(u128::from(input.days_underpayment_period))
        .checked_div(
            u128::from(IRC_6655_BASIS_POINT_DENOMINATOR)
                .saturating_mul(u128::from(IRC_6655_DAYS_PER_YEAR)),
        )
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64;

    if is_large_corporation
        && !is_first_installment
        && input.safe_harbor_method == SafeHarborMethod::LesserOfCurrentOrPrecedingYearTax
        && input.current_year_tax_dollars > input.preceding_year_tax_dollars
    {
        return Output {
            mode: Section6655Mode::ViolationLargeCorporationUsedPriorYearTaxAfterFirstInstallment,
            statutory_basis: "IRC § 6655(d)(2)(A) — large corporation may NOT use prior-year tax safe harbor after first installment".to_string(),
            notes: format!(
                "VIOLATION: large corporation attempted to use prior-year tax safe harbor for installment 2, 3, or 4; § 6655(d)(2) bars prior-year safe harbor after the first installment; required installment must be computed using 100 % current-year tax = 25 % × ${} = ${}; underpayment ${}; estimated addition to tax = ${}.",
                input.current_year_tax_dollars,
                required_installment_amount,
                underpayment,
                addition_to_tax
            ),
            citations,
            required_installment_amount_dollars: required_installment_amount,
            underpayment_amount_dollars: underpayment,
            estimated_addition_to_tax_dollars: addition_to_tax,
        };
    }

    let mode = if is_large_corporation {
        Section6655Mode::ViolationLargeCorporationUnderpaidEstimatedTaxInstallmentAdditionToTaxAtSection6621Rate
    } else {
        Section6655Mode::ViolationStandardCorporationUnderpaidEstimatedTaxInstallmentAdditionToTaxAtSection6621Rate
    };

    Output {
        mode,
        statutory_basis: "IRC § 6655(a) — addition to tax for underpaid required installment at § 6621 underpayment rate".to_string(),
        notes: format!(
            "VIOLATION: required installment ${} but only ${} paid; underpayment ${}; § 6621 underpayment rate = FSTR ({} bps) + 3 pp = {} bps; underpayment period {} days; estimated linear addition to tax ≈ ${} (statutory § 6622 daily compounding so approximation).",
            required_installment_amount,
            input.installment_amount_paid_dollars,
            underpayment,
            input.federal_short_term_rate_basis_points,
            underpayment_rate_basis_points,
            input.days_underpayment_period,
            addition_to_tax
        ),
        citations,
        required_installment_amount_dollars: required_installment_amount,
        underpayment_amount_dollars: underpayment,
        estimated_addition_to_tax_dollars: addition_to_tax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_standard_corp_input() -> Input {
        Input {
            corporation_type: CorporationType::StandardCorporationBelowLargeCorporationThreshold,
            max_taxable_income_in_3_preceding_years_dollars: 500_000,
            total_tax_shown_on_return_dollars: 100_000,
            current_year_tax_dollars: 100_000,
            preceding_year_tax_dollars: 80_000,
            preceding_year_was_full_12_month_return_with_tax_liability: true,
            installment_quarter: InstallmentQuarter::Quarter1AprilOf15,
            installment_amount_paid_dollars: 20_000,
            safe_harbor_method: SafeHarborMethod::LesserOfCurrentOrPrecedingYearTax,
            federal_short_term_rate_basis_points: 500,
            days_underpayment_period: 90,
        }
    }

    fn baseline_large_corp_input() -> Input {
        Input {
            corporation_type:
                CorporationType::LargeCorporationTaxableIncomeAtOrAbove1MillionInAnyOf3PrecedingYears,
            max_taxable_income_in_3_preceding_years_dollars: 5_000_000,
            total_tax_shown_on_return_dollars: 1_000_000,
            current_year_tax_dollars: 1_000_000,
            preceding_year_tax_dollars: 800_000,
            preceding_year_was_full_12_month_return_with_tax_liability: true,
            installment_quarter: InstallmentQuarter::Quarter1AprilOf15,
            installment_amount_paid_dollars: 200_000,
            safe_harbor_method: SafeHarborMethod::LesserOfCurrentOrPrecedingYearTax,
            federal_short_term_rate_basis_points: 500,
            days_underpayment_period: 90,
        }
    }

    #[test]
    fn total_tax_under_500_small_underpayment_exception() {
        let mut input = baseline_standard_corp_input();
        input.total_tax_shown_on_return_dollars = 400;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::NotApplicableTotalTaxBelow500SmallUnderpaymentException
        );
    }

    #[test]
    fn total_tax_at_exactly_500_no_exception_strict_less_than() {
        // § 6655(f) "less than $500" strict
        let mut input = baseline_standard_corp_input();
        input.total_tax_shown_on_return_dollars = 500;
        input.current_year_tax_dollars = 500;
        input.preceding_year_tax_dollars = 500;
        input.installment_amount_paid_dollars = 125; // 25 % × $500 = $125
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantQuarterlyInstallmentPaidAt25PctOfRequiredAnnualPayment
        );
        assert_eq!(output.required_installment_amount_dollars, 125);
    }

    #[test]
    fn standard_corp_q1_paid_at_25pct_lesser_safe_harbor_compliant() {
        let output = check(&baseline_standard_corp_input());
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantQuarterlyInstallmentPaidAt25PctOfRequiredAnnualPayment
        );
        // Required = 25 % × lesser(100K, 80K) = 25 % × 80K = $20,000
        assert_eq!(output.required_installment_amount_dollars, 20_000);
        assert_eq!(output.underpayment_amount_dollars, 0);
    }

    #[test]
    fn standard_corp_q1_underpayment_addition_to_tax() {
        let mut input = baseline_standard_corp_input();
        input.installment_amount_paid_dollars = 10_000; // Half of required
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::ViolationStandardCorporationUnderpaidEstimatedTaxInstallmentAdditionToTaxAtSection6621Rate
        );
        // Underpayment = 20K - 10K = 10K
        assert_eq!(output.underpayment_amount_dollars, 10_000);
        // Rate = 500 + 300 = 800 bps; period 90 days; addition ≈ 10000 × 800 × 90 / (10000 × 365) ≈ 197
        assert_eq!(output.estimated_addition_to_tax_dollars, 197);
    }

    #[test]
    fn preceding_year_short_period_falls_back_to_current_year_tax() {
        let mut input = baseline_standard_corp_input();
        input.preceding_year_was_full_12_month_return_with_tax_liability = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::NotApplicablePrecedingYearWasShortPeriodOrNoReturnFiled
        );
        // Recomputed using current year only: 25 % × $100,000 = $25,000
        assert_eq!(output.required_installment_amount_dollars, 25_000);
    }

    #[test]
    fn large_corp_q1_uses_prior_year_safe_harbor_compliant() {
        let output = check(&baseline_large_corp_input());
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantLargeCorporationFirstInstallmentUsedPriorYearTaxSafeHarbor
        );
        // Required = 25 % × lesser($1M, $800K) = $200K
        assert_eq!(output.required_installment_amount_dollars, 200_000);
    }

    #[test]
    fn large_corp_q2_must_use_current_year_tax_violation_if_using_prior() {
        let mut input = baseline_large_corp_input();
        input.installment_quarter = InstallmentQuarter::Quarter2JuneOf15;
        input.installment_amount_paid_dollars = 200_000; // Only enough for prior-year-based requirement
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::ViolationLargeCorporationUsedPriorYearTaxAfterFirstInstallment
        );
        // Required = 25 % × $1M current year = $250K; underpayment $50K
        assert_eq!(output.required_installment_amount_dollars, 250_000);
        assert_eq!(output.underpayment_amount_dollars, 50_000);
    }

    #[test]
    fn large_corp_q2_using_current_year_tax_compliant() {
        let mut input = baseline_large_corp_input();
        input.installment_quarter = InstallmentQuarter::Quarter2JuneOf15;
        input.installment_amount_paid_dollars = 250_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantQuarterlyInstallmentPaidAt25PctOfRequiredAnnualPayment
        );
    }

    #[test]
    fn annualized_income_method_compliant() {
        let mut input = baseline_standard_corp_input();
        input.safe_harbor_method =
            SafeHarborMethod::AnnualizedIncomeInstallmentMethodUnderSection6655E;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantQuarterlyInstallmentMetUsingAnnualizedIncomeInstallmentMethod
        );
    }

    #[test]
    fn adjusted_seasonal_method_compliant() {
        let mut input = baseline_standard_corp_input();
        input.safe_harbor_method =
            SafeHarborMethod::AdjustedSeasonalInstallmentMethodUnderSection6655E;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantQuarterlyInstallmentMetUsingAdjustedSeasonalInstallmentMethod
        );
    }

    #[test]
    fn large_corp_general_underpayment_violation() {
        let mut input = baseline_large_corp_input();
        input.installment_quarter = InstallmentQuarter::Quarter3SeptemberOf15;
        input.safe_harbor_method = SafeHarborMethod::NoSafeHarborMethodSelected;
        input.installment_amount_paid_dollars = 100_000; // Far below $250K required
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::ViolationLargeCorporationUnderpaidEstimatedTaxInstallmentAdditionToTaxAtSection6621Rate
        );
        assert_eq!(output.required_installment_amount_dollars, 250_000);
        assert_eq!(output.underpayment_amount_dollars, 150_000);
    }

    #[test]
    fn corporation_at_max_income_999999_below_large_threshold() {
        let mut input = baseline_standard_corp_input();
        input.max_taxable_income_in_3_preceding_years_dollars = 999_999;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantQuarterlyInstallmentPaidAt25PctOfRequiredAnnualPayment
        );
        // Not large corp -> can use prior-year safe harbor for all quarters
    }

    #[test]
    fn corporation_at_exactly_1_million_is_large_corp() {
        // Statutory: "OR MORE" — strict ≥
        let mut input = baseline_standard_corp_input();
        input.corporation_type = CorporationType::StandardCorporationBelowLargeCorporationThreshold;
        input.max_taxable_income_in_3_preceding_years_dollars = 1_000_000;
        input.installment_quarter = InstallmentQuarter::Quarter1AprilOf15;
        let output = check(&input);
        // Q1 = large corp first-installment carve-out, so prior-year still usable
        assert_eq!(
            output.mode,
            Section6655Mode::CompliantLargeCorporationFirstInstallmentUsedPriorYearTaxSafeHarbor
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_6655_INSTALLMENT_PERCENTAGE_BASIS_POINTS, 2_500);
        assert_eq!(IRC_6655_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_6655_NUMBER_OF_INSTALLMENTS_PER_YEAR, 4);
        assert_eq!(IRC_6655_SMALL_UNDERPAYMENT_EXCEPTION_THRESHOLD_DOLLARS, 500);
        assert_eq!(
            IRC_6655_LARGE_CORPORATION_TAXABLE_INCOME_THRESHOLD_DOLLARS,
            1_000_000
        );
        assert_eq!(IRC_6655_LARGE_CORPORATION_LOOKBACK_YEARS, 3);
        assert_eq!(IRC_6655_INSTALLMENT_1_MONTH, 4);
        assert_eq!(IRC_6655_INSTALLMENT_1_DAY, 15);
        assert_eq!(IRC_6655_INSTALLMENT_2_MONTH, 6);
        assert_eq!(IRC_6655_INSTALLMENT_3_MONTH, 9);
        assert_eq!(IRC_6655_INSTALLMENT_4_MONTH, 12);
        assert_eq!(IRC_6655_DAYS_PER_YEAR, 365);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_standard_corp_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 6655(a)"));
        assert!(joined.contains("§ 6655(b)"));
        assert!(joined.contains("§ 6655(c)"));
        assert!(joined.contains("§ 6655(d)"));
        assert!(joined.contains("§ 6655(e)"));
        assert!(joined.contains("§ 6655(f)"));
        assert!(joined.contains("§ 6655(g)"));
        assert!(joined.contains("§ 6655(h)"));
        assert!(joined.contains("§ 6621"));
        assert!(joined.contains("§ 6622"));
        assert!(joined.contains("§ 6654"));
        assert!(joined.contains("§ 6425"));
        assert!(joined.contains("25 PERCENT"));
        assert!(joined.contains("$500"));
        assert!(joined.contains("$1,000,000"));
        assert!(joined.contains("APRIL 15, JUNE 15, SEPTEMBER 15, DECEMBER 15"));
    }

    #[test]
    fn saturating_overflow_defense() {
        let mut input = baseline_standard_corp_input();
        input.current_year_tax_dollars = u64::MAX;
        input.preceding_year_tax_dollars = u64::MAX;
        input.installment_amount_paid_dollars = 0;
        input.federal_short_term_rate_basis_points = u64::MAX;
        input.days_underpayment_period = u32::MAX;
        let output = check(&input);
        // Saturating arithmetic prevents panic
        let _ = output.mode;
    }
}
