//! IRC § 6621 — Determination of Rate of Interest Module.
//!
//! Pure-compute check for IRC § 6621 interest-rate
//! determination for both overpayments and underpayments of
//! federal tax. § 6621 is the master rate-setting provision
//! that feeds every § 6601 interest-on-underpayments
//! computation and every § 6611 interest-on-overpayments
//! computation. § 6621(b) federal short-term rate is
//! determined quarterly under the § 1274(d) AFR methodology
//! and serves as the spine for the entire civil-penalty
//! interest framework. § 6622 imposes daily compounding on
//! all interest accrued under § 6601/§ 6611, so the module's
//! linear-rate approximations are simplifications of the
//! actual daily-compounded computation.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6621(a)(1) Overpayment Rate**: the overpayment
//!   rate is the sum of (A) the **FEDERAL SHORT-TERM RATE**
//!   determined under subsection (b) PLUS (B) **3 PERCENTAGE
//!   POINTS** (or **2 PERCENTAGE POINTS** in the case of a
//!   CORPORATION). For the portion of a CORPORATE
//!   overpayment of tax EXCEEDING **$10,000** for any
//!   taxable period, the rate is the federal short-term
//!   rate plus **0.5 PERCENTAGE POINTS** (substituting
//!   "0.5 percentage point" for "2 percentage points")
//!   ([Cornell LII 26 USC § 6621](https://www.law.cornell.edu/uscode/text/26/6621);
//!   [Bloomberg Tax Sec. 6621](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6621);
//!   [IRS Rev. Rul. 2022-23 — Section 6621 Determination of
//!   Rate of Interest](https://www.irs.gov/pub/irs-drop/rr-22-23.pdf)).
//! - **IRC § 6621(a)(2) Underpayment Rate**: the underpayment
//!   rate is the sum of (A) the **FEDERAL SHORT-TERM RATE**
//!   determined under subsection (b) PLUS (B) **3 PERCENTAGE
//!   POINTS** — applies to ALL taxpayers (individuals,
//!   corporations, trusts, estates) for standard underpayments.
//! - **IRC § 6621(b) Federal Short-Term Rate Determination**:
//!   the **SECRETARY shall determine the federal short-term
//!   rate for the FIRST MONTH IN EACH CALENDAR QUARTER**;
//!   this rate applies during the first calendar quarter
//!   beginning after that month. The rate follows the
//!   **§ 1274(d) AFR methodology** and is **ROUNDED to the
//!   NEAREST FULL PERCENT** (or to the **NEXT HIGHEST FULL
//!   PERCENT** if the rate is a multiple of 0.5 percent).
//! - **IRC § 6621(c) Large Corporate Underpayment**: for
//!   purposes of interest payable under § 6601 on any
//!   **LARGE CORPORATE UNDERPAYMENT**, the underpayment
//!   rate under § 6621(a)(2) is determined by substituting
//!   **"5 PERCENTAGE POINTS" for "3 PERCENTAGE POINTS"**. A
//!   "large corporate underpayment" means an underpayment of
//!   tax by a **C CORPORATION** for any taxable period if
//!   the amount of such underpayment EXCEEDS **$100,000**.
//!   The increased rate applies only to interest accruing
//!   AFTER the **APPLICABLE DATE** = **30 DAYS** following
//!   the earlier of (i) the date on which the first letter
//!   of proposed deficiency is sent, OR (ii) the date on
//!   which the formal notice of deficiency is sent.
//! - **IRC § 6621(d) Elimination of Interest on Overlapping
//!   Periods of Tax Overpayments and Underpayments**: to the
//!   extent that, for any period, interest is payable under
//!   subchapter A AND allowable under subchapter B on
//!   EQUIVALENT amounts of overpayment and underpayment by
//!   the SAME taxpayer of tax imposed by Title 26, the **NET
//!   RATE OF INTEREST** under § 6621 on such amounts is
//!   **ZERO** for such period.
//! - **§ 6622 Daily Compounding (cross-reference)**: interest
//!   on underpayments and overpayments is **COMPOUNDED
//!   DAILY** under § 6622; the module's linear-rate
//!   computations are simplifications. The IRS publishes
//!   quarterly tables of daily-compounded factors based on
//!   the § 6621 rate for accurate computation.
//! - **§ 1274(d) AFR Methodology (cross-reference)**: the
//!   federal short-term rate under § 6621(b) is determined
//!   using the § 1274(d) AFR methodology. The federal
//!   short-term rate applies to obligations with a term of
//!   3 years or less.
//! - **Companion Provisions**: § 6601 (interest on
//!   underpayments — primary consumer of § 6621 rate);
//!   § 6611 (interest on overpayments — primary consumer of
//!   § 6621(a)(1) rate); § 6622 (interest compounded daily);
//!   § 1274 (issue price + AFR determination); § 6655 (failure
//!   to make estimated tax by corp); § 6654 (failure to make
//!   estimated tax by individual).
//! - **Quarterly Rate Publication**: the IRS publishes a
//!   Revenue Ruling each quarter announcing the § 6621 rates
//!   for the upcoming quarter (e.g., Rev. Rul. 2022-23,
//!   2019-28).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6621_OVERPAYMENT_INDIVIDUAL_RATE_BASIS_POINTS: u64 = 300;
pub const IRC_6621_OVERPAYMENT_CORPORATE_BASE_RATE_BASIS_POINTS: u64 = 200;
pub const IRC_6621_OVERPAYMENT_CORPORATE_EXCESS_RATE_BASIS_POINTS: u64 = 50;
pub const IRC_6621_OVERPAYMENT_CORPORATE_EXCESS_THRESHOLD_DOLLARS: u64 = 10_000;
pub const IRC_6621_UNDERPAYMENT_STANDARD_RATE_BASIS_POINTS: u64 = 300;
pub const IRC_6621_LARGE_CORPORATE_UNDERPAYMENT_RATE_BASIS_POINTS: u64 = 500;
pub const IRC_6621_LARGE_CORPORATE_UNDERPAYMENT_THRESHOLD_DOLLARS: u64 = 100_000;
pub const IRC_6621_LARGE_CORPORATE_APPLICABLE_DATE_DAYS_AFTER_NOTICE: u32 = 30;
pub const IRC_6621_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    Individual,
    CCorporation,
    OtherEntityNotCCorporationTreatedAsIndividualForRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountStatus {
    OverpaymentByTaxpayer,
    UnderpaymentByTaxpayer,
    NoUnderpaymentOrOverpayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LargeCorporateApplicableDateStatus {
    ThirtyDayWindowAfterDeficiencyNoticeHasPassed,
    ThirtyDayWindowAfterDeficiencyNoticeNotPassed,
    NotApplicableNotLargeCorporateUnderpayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NetZeroOverlappingPeriodStatus {
    NetZeroOverlappingOverpaymentAndUnderpaymentAppliesUnderSection6621D,
    NoOverlappingPeriodSection6621DNotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6621Mode {
    NotApplicableNoUnderpaymentOrOverpayment,
    CompliantIndividualOverpaymentRateFstrPlus3PercentagePoints,
    CompliantCorporateOverpaymentRateFstrPlus2PercentagePointsForFirst10000,
    CompliantCorporateOverpaymentRateFstrPlus0_5PercentagePointsForPortionExceeding10000,
    CompliantStandardUnderpaymentRateFstrPlus3PercentagePoints,
    CompliantLargeCorporateUnderpaymentRateFstrPlus5PercentagePointsUnderSection6621C,
    CompliantNetZeroRateForOverlappingOverpaymentAndUnderpaymentUnderSection6621D,
    PendingLargeCorporateRateBefore30DayApplicableDateAfterDeficiencyNotice,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub taxpayer_type: TaxpayerType,
    pub amount_status: AmountStatus,
    pub amount_dollars: u64,
    pub federal_short_term_rate_basis_points: u64,
    pub corporate_overpayment_portion_exceeding_10000_dollars: u64,
    pub corporate_underpayment_total_dollars: u64,
    pub large_corporate_applicable_date_status: LargeCorporateApplicableDateStatus,
    pub net_zero_overlapping_period_status: NetZeroOverlappingPeriodStatus,
    pub number_of_full_years_interest_accrues: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6621Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub applicable_rate_basis_points: u64,
    pub estimated_linear_interest_dollars: u64,
}

pub type Section6621Input = Input;
pub type Section6621Output = Output;
pub type Section6621Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6621(a)(1) Overpayment Rate — sum of (A) federal short-term rate determined under (b) PLUS (B) 3 percentage points (individuals); 2 percentage points (corporations); 0.5 percentage points for corporate overpayment portion exceeding $10,000 in any taxable period".to_string(),
        "IRC § 6621(a)(2) Underpayment Rate — sum of (A) federal short-term rate determined under (b) PLUS (B) 3 percentage points; applies to all taxpayers for standard underpayments".to_string(),
        "IRC § 6621(b) Federal Short-Term Rate Determination — Secretary determines federal short-term rate for FIRST MONTH IN EACH CALENDAR QUARTER; applies during first calendar quarter beginning after that month; follows § 1274(d) AFR methodology; rounded to nearest full percent (or next highest full percent if multiple of 0.5 percent)".to_string(),
        "IRC § 6621(c) Large Corporate Underpayment — for interest under § 6601 on LARGE CORPORATE UNDERPAYMENT, underpayment rate under (a)(2) determined by substituting '5 percentage points' for '3 percentage points'; large corporate underpayment = C corporation underpayment EXCEEDING $100,000 in any taxable period; increased rate applies only to interest accruing AFTER applicable date = 30 days following earlier of proposed deficiency notice or formal notice of deficiency".to_string(),
        "IRC § 6621(d) Elimination of Interest on Overlapping Periods — to extent interest payable under subchapter A AND allowable under subchapter B on EQUIVALENT overpayment and underpayment amounts by SAME taxpayer of Title 26 tax, NET RATE on such amounts is ZERO for such period".to_string(),
        "§ 6622 Daily Compounding (cross-reference) — interest on underpayments and overpayments COMPOUNDED DAILY under § 6622; § 6621 linear rates are simplifications; IRS publishes quarterly daily-compounded factor tables".to_string(),
        "§ 1274(d) AFR Methodology (cross-reference) — federal short-term rate under § 6621(b) determined using § 1274(d) AFR methodology; federal short-term rate applies to obligations with term of 3 years or less".to_string(),
        "Companion Provisions — § 6601 (interest on underpayments — primary consumer of § 6621 rate); § 6611 (interest on overpayments — primary consumer of § 6621(a)(1) rate); § 6622 (interest compounded daily); § 1274 (issue price + AFR determination); § 6655 (failure to make estimated tax by corp); § 6654 (failure to make estimated tax by individual)".to_string(),
        "Quarterly Rate Publication — IRS publishes Revenue Ruling each quarter announcing § 6621 rates for upcoming quarter (e.g., Rev. Rul. 2022-23 for 4Q 2022; Rev. Rul. 2019-28 for 1Q 2020); cumulative rate table maintained at IRS.gov".to_string(),
        "Cornell LII 26 USC § 6621 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6621 — comprehensive code commentary".to_string(),
        "IRS Rev. Rul. 2022-23 — Section 6621 Determination of Rate of Interest (4Q 2022 quarterly rate publication)".to_string(),
        "U.S. Department of Labor — IRC § 6621 Table of Underpayment Rates (used by ERISA Voluntary Fiduciary Correction Program)".to_string(),
    ];

    if input.amount_status == AmountStatus::NoUnderpaymentOrOverpayment {
        return Output {
            mode: Section6621Mode::NotApplicableNoUnderpaymentOrOverpayment,
            statutory_basis: "IRC § 6621 — applies only when there is an underpayment or overpayment to which interest attaches".to_string(),
            notes: "NOT APPLICABLE: no underpayment or overpayment to which § 6621 interest rate attaches.".to_string(),
            citations,
            applicable_rate_basis_points: 0,
            estimated_linear_interest_dollars: 0,
        };
    }

    if input.net_zero_overlapping_period_status
        == NetZeroOverlappingPeriodStatus::NetZeroOverlappingOverpaymentAndUnderpaymentAppliesUnderSection6621D
    {
        return Output {
            mode: Section6621Mode::CompliantNetZeroRateForOverlappingOverpaymentAndUnderpaymentUnderSection6621D,
            statutory_basis: "IRC § 6621(d) — net zero rate for overlapping overpayment and underpayment by same taxpayer".to_string(),
            notes: "COMPLIANT: net zero rate applies under § 6621(d) for overlapping period of equivalent overpayment and underpayment by same taxpayer; no interest accrues on the overlapping amounts.".to_string(),
            citations,
            applicable_rate_basis_points: 0,
            estimated_linear_interest_dollars: 0,
        };
    }

    if input.amount_status == AmountStatus::UnderpaymentByTaxpayer {
        let is_large_corporate_underpayment = input.taxpayer_type == TaxpayerType::CCorporation
            && input.corporate_underpayment_total_dollars
                > IRC_6621_LARGE_CORPORATE_UNDERPAYMENT_THRESHOLD_DOLLARS;

        if is_large_corporate_underpayment {
            if input.large_corporate_applicable_date_status
                == LargeCorporateApplicableDateStatus::ThirtyDayWindowAfterDeficiencyNoticeNotPassed
            {
                let rate = input
                    .federal_short_term_rate_basis_points
                    .saturating_add(IRC_6621_UNDERPAYMENT_STANDARD_RATE_BASIS_POINTS);
                let interest = simple_interest(
                    input.amount_dollars,
                    rate,
                    input.number_of_full_years_interest_accrues,
                );
                return Output {
                    mode: Section6621Mode::PendingLargeCorporateRateBefore30DayApplicableDateAfterDeficiencyNotice,
                    statutory_basis: "IRC § 6621(c) — large corporate underpayment 5-percentage-point rate applies only after applicable date = 30 days after earlier of proposed deficiency notice or formal deficiency notice".to_string(),
                    notes: format!(
                        "PENDING: large corporate underpayment of ${} (> $100,000 threshold) identified, but the 30-day window after the earlier of proposed or formal deficiency notice has NOT yet passed; standard § 6621(a)(2) rate of FSTR + 3 pp = {} basis points applies in the interim; the 5-pp rate begins accruing on day 31.",
                        input.corporate_underpayment_total_dollars, rate
                    ),
                    citations,
                    applicable_rate_basis_points: rate,
                    estimated_linear_interest_dollars: interest,
                };
            }
            let rate = input
                .federal_short_term_rate_basis_points
                .saturating_add(IRC_6621_LARGE_CORPORATE_UNDERPAYMENT_RATE_BASIS_POINTS);
            let interest = simple_interest(
                input.amount_dollars,
                rate,
                input.number_of_full_years_interest_accrues,
            );
            return Output {
                mode: Section6621Mode::CompliantLargeCorporateUnderpaymentRateFstrPlus5PercentagePointsUnderSection6621C,
                statutory_basis: "IRC § 6621(c) — large corporate underpayment rate = federal short-term rate + 5 percentage points".to_string(),
                notes: format!(
                    "COMPLIANT: large corporate C-corporation underpayment of ${} (> $100,000 threshold) past the applicable date (30 days after earlier of proposed/formal deficiency notice); § 6621(c) substitutes '5 percentage points' for '3 percentage points'; applicable rate = FSTR ({} bps) + 5 pp = {} basis points; estimated linear interest = ${} (note: statutory § 6622 imposes daily compounding so this is an approximation).",
                    input.corporate_underpayment_total_dollars,
                    input.federal_short_term_rate_basis_points,
                    rate,
                    interest
                ),
                citations,
                applicable_rate_basis_points: rate,
                estimated_linear_interest_dollars: interest,
            };
        }

        let rate = input
            .federal_short_term_rate_basis_points
            .saturating_add(IRC_6621_UNDERPAYMENT_STANDARD_RATE_BASIS_POINTS);
        let interest = simple_interest(
            input.amount_dollars,
            rate,
            input.number_of_full_years_interest_accrues,
        );
        return Output {
            mode: Section6621Mode::CompliantStandardUnderpaymentRateFstrPlus3PercentagePoints,
            statutory_basis: "IRC § 6621(a)(2) — standard underpayment rate = federal short-term rate + 3 percentage points".to_string(),
            notes: format!(
                "COMPLIANT: standard underpayment rate applies under § 6621(a)(2); FSTR ({} bps) + 3 pp = {} basis points; estimated linear interest = ${} on underpayment of ${} over {} year(s) (§ 6622 imposes daily compounding so this is a linear approximation).",
                input.federal_short_term_rate_basis_points,
                rate,
                interest,
                input.amount_dollars,
                input.number_of_full_years_interest_accrues
            ),
            citations,
            applicable_rate_basis_points: rate,
            estimated_linear_interest_dollars: interest,
        };
    }

    // Overpayment branches
    match input.taxpayer_type {
        TaxpayerType::CCorporation => {
            if input.corporate_overpayment_portion_exceeding_10000_dollars > 0 {
                let rate = input
                    .federal_short_term_rate_basis_points
                    .saturating_add(IRC_6621_OVERPAYMENT_CORPORATE_EXCESS_RATE_BASIS_POINTS);
                let interest = simple_interest(
                    input.corporate_overpayment_portion_exceeding_10000_dollars,
                    rate,
                    input.number_of_full_years_interest_accrues,
                );
                Output {
                    mode: Section6621Mode::CompliantCorporateOverpaymentRateFstrPlus0_5PercentagePointsForPortionExceeding10000,
                    statutory_basis: "IRC § 6621(a)(1)(B) — corporate overpayment portion exceeding $10,000 receives federal short-term rate + 0.5 percentage points".to_string(),
                    notes: format!(
                        "COMPLIANT: corporate overpayment portion exceeding $10,000 = ${}; rate = FSTR ({} bps) + 0.5 pp (= 50 bps) = {} basis points; estimated linear interest on excess portion = ${} over {} year(s) (statutory § 6622 daily compounding so approximation only).",
                        input.corporate_overpayment_portion_exceeding_10000_dollars,
                        input.federal_short_term_rate_basis_points,
                        rate,
                        interest,
                        input.number_of_full_years_interest_accrues
                    ),
                    citations,
                    applicable_rate_basis_points: rate,
                    estimated_linear_interest_dollars: interest,
                }
            } else {
                let rate = input
                    .federal_short_term_rate_basis_points
                    .saturating_add(IRC_6621_OVERPAYMENT_CORPORATE_BASE_RATE_BASIS_POINTS);
                let interest = simple_interest(
                    input.amount_dollars,
                    rate,
                    input.number_of_full_years_interest_accrues,
                );
                Output {
                    mode: Section6621Mode::CompliantCorporateOverpaymentRateFstrPlus2PercentagePointsForFirst10000,
                    statutory_basis: "IRC § 6621(a)(1)(B) — corporate overpayment first $10,000 receives federal short-term rate + 2 percentage points".to_string(),
                    notes: format!(
                        "COMPLIANT: corporate overpayment of ${} at or below $10,000 threshold; rate = FSTR ({} bps) + 2 pp = {} basis points; estimated linear interest = ${} over {} year(s) (statutory § 6622 daily compounding so approximation only).",
                        input.amount_dollars,
                        input.federal_short_term_rate_basis_points,
                        rate,
                        interest,
                        input.number_of_full_years_interest_accrues
                    ),
                    citations,
                    applicable_rate_basis_points: rate,
                    estimated_linear_interest_dollars: interest,
                }
            }
        }
        TaxpayerType::Individual
        | TaxpayerType::OtherEntityNotCCorporationTreatedAsIndividualForRate => {
            let rate = input
                .federal_short_term_rate_basis_points
                .saturating_add(IRC_6621_OVERPAYMENT_INDIVIDUAL_RATE_BASIS_POINTS);
            let interest = simple_interest(
                input.amount_dollars,
                rate,
                input.number_of_full_years_interest_accrues,
            );
            Output {
                mode: Section6621Mode::CompliantIndividualOverpaymentRateFstrPlus3PercentagePoints,
                statutory_basis: "IRC § 6621(a)(1) — individual overpayment rate = federal short-term rate + 3 percentage points".to_string(),
                notes: format!(
                    "COMPLIANT: individual (non-corporate) overpayment of ${}; rate = FSTR ({} bps) + 3 pp = {} basis points; estimated linear interest = ${} over {} year(s) (statutory § 6622 daily compounding so approximation only).",
                    input.amount_dollars,
                    input.federal_short_term_rate_basis_points,
                    rate,
                    interest,
                    input.number_of_full_years_interest_accrues
                ),
                citations,
                applicable_rate_basis_points: rate,
                estimated_linear_interest_dollars: interest,
            }
        }
    }
}

fn simple_interest(amount_dollars: u64, rate_basis_points: u64, years: u32) -> u64 {
    u128::from(amount_dollars)
        .saturating_mul(u128::from(rate_basis_points))
        .saturating_mul(u128::from(years))
        .checked_div(u128::from(IRC_6621_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_individual_overpayment_input() -> Input {
        Input {
            taxpayer_type: TaxpayerType::Individual,
            amount_status: AmountStatus::OverpaymentByTaxpayer,
            amount_dollars: 50_000,
            federal_short_term_rate_basis_points: 500,
            corporate_overpayment_portion_exceeding_10000_dollars: 0,
            corporate_underpayment_total_dollars: 0,
            large_corporate_applicable_date_status:
                LargeCorporateApplicableDateStatus::NotApplicableNotLargeCorporateUnderpayment,
            net_zero_overlapping_period_status:
                NetZeroOverlappingPeriodStatus::NoOverlappingPeriodSection6621DNotApplicable,
            number_of_full_years_interest_accrues: 2,
        }
    }

    fn baseline_underpayment_input() -> Input {
        Input {
            taxpayer_type: TaxpayerType::Individual,
            amount_status: AmountStatus::UnderpaymentByTaxpayer,
            amount_dollars: 50_000,
            federal_short_term_rate_basis_points: 500,
            corporate_overpayment_portion_exceeding_10000_dollars: 0,
            corporate_underpayment_total_dollars: 0,
            large_corporate_applicable_date_status:
                LargeCorporateApplicableDateStatus::NotApplicableNotLargeCorporateUnderpayment,
            net_zero_overlapping_period_status:
                NetZeroOverlappingPeriodStatus::NoOverlappingPeriodSection6621DNotApplicable,
            number_of_full_years_interest_accrues: 2,
        }
    }

    #[test]
    fn no_underpayment_or_overpayment_not_applicable() {
        let mut input = baseline_individual_overpayment_input();
        input.amount_status = AmountStatus::NoUnderpaymentOrOverpayment;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::NotApplicableNoUnderpaymentOrOverpayment
        );
        assert_eq!(output.applicable_rate_basis_points, 0);
        assert_eq!(output.estimated_linear_interest_dollars, 0);
    }

    #[test]
    fn net_zero_overlapping_period_applies_zero_rate() {
        let mut input = baseline_individual_overpayment_input();
        input.net_zero_overlapping_period_status =
            NetZeroOverlappingPeriodStatus::NetZeroOverlappingOverpaymentAndUnderpaymentAppliesUnderSection6621D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantNetZeroRateForOverlappingOverpaymentAndUnderpaymentUnderSection6621D
        );
        assert_eq!(output.applicable_rate_basis_points, 0);
        assert_eq!(output.estimated_linear_interest_dollars, 0);
    }

    #[test]
    fn individual_overpayment_fstr_plus_3pp_compliant() {
        let output = check(&baseline_individual_overpayment_input());
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantIndividualOverpaymentRateFstrPlus3PercentagePoints
        );
        // FSTR 500 + 300 = 800 bps = 8%; on $50,000 over 2 years simple interest = 50_000 * 800 * 2 / 10_000 = 8000
        assert_eq!(output.applicable_rate_basis_points, 800);
        assert_eq!(output.estimated_linear_interest_dollars, 8_000);
    }

    #[test]
    fn corporate_overpayment_first_10000_fstr_plus_2pp_compliant() {
        let mut input = baseline_individual_overpayment_input();
        input.taxpayer_type = TaxpayerType::CCorporation;
        input.amount_dollars = 8_000; // ≤ $10,000 threshold
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantCorporateOverpaymentRateFstrPlus2PercentagePointsForFirst10000
        );
        // FSTR 500 + 200 = 700 bps = 7%; on $8,000 over 2 years = 8000 * 700 * 2 / 10_000 = 1120
        assert_eq!(output.applicable_rate_basis_points, 700);
        assert_eq!(output.estimated_linear_interest_dollars, 1_120);
    }

    #[test]
    fn corporate_overpayment_portion_exceeding_10000_fstr_plus_0_5pp_compliant() {
        let mut input = baseline_individual_overpayment_input();
        input.taxpayer_type = TaxpayerType::CCorporation;
        input.amount_dollars = 50_000;
        input.corporate_overpayment_portion_exceeding_10000_dollars = 40_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantCorporateOverpaymentRateFstrPlus0_5PercentagePointsForPortionExceeding10000
        );
        // FSTR 500 + 50 = 550 bps = 5.5%; on $40,000 portion over 2 years = 40_000 * 550 * 2 / 10_000 = 4400
        assert_eq!(output.applicable_rate_basis_points, 550);
        assert_eq!(output.estimated_linear_interest_dollars, 4_400);
    }

    #[test]
    fn standard_underpayment_fstr_plus_3pp_compliant() {
        let output = check(&baseline_underpayment_input());
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantStandardUnderpaymentRateFstrPlus3PercentagePoints
        );
        // FSTR 500 + 300 = 800 bps = 8%; on $50,000 over 2 years = 50_000 * 800 * 2 / 10_000 = 8000
        assert_eq!(output.applicable_rate_basis_points, 800);
        assert_eq!(output.estimated_linear_interest_dollars, 8_000);
    }

    #[test]
    fn large_corporate_underpayment_post_30_day_window_fstr_plus_5pp_compliant() {
        let mut input = baseline_underpayment_input();
        input.taxpayer_type = TaxpayerType::CCorporation;
        input.corporate_underpayment_total_dollars = 200_000; // > $100,000 threshold
        input.amount_dollars = 200_000;
        input.large_corporate_applicable_date_status =
            LargeCorporateApplicableDateStatus::ThirtyDayWindowAfterDeficiencyNoticeHasPassed;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantLargeCorporateUnderpaymentRateFstrPlus5PercentagePointsUnderSection6621C
        );
        // FSTR 500 + 500 = 1000 bps = 10%; on $200,000 over 2 years = 200_000 * 1000 * 2 / 10_000 = 40_000
        assert_eq!(output.applicable_rate_basis_points, 1_000);
        assert_eq!(output.estimated_linear_interest_dollars, 40_000);
    }

    #[test]
    fn large_corporate_underpayment_pre_30_day_window_pending_standard_rate() {
        let mut input = baseline_underpayment_input();
        input.taxpayer_type = TaxpayerType::CCorporation;
        input.corporate_underpayment_total_dollars = 200_000;
        input.amount_dollars = 200_000;
        input.large_corporate_applicable_date_status =
            LargeCorporateApplicableDateStatus::ThirtyDayWindowAfterDeficiencyNoticeNotPassed;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::PendingLargeCorporateRateBefore30DayApplicableDateAfterDeficiencyNotice
        );
        // Standard rate applies pre-window: FSTR 500 + 300 = 800 bps
        assert_eq!(output.applicable_rate_basis_points, 800);
    }

    #[test]
    fn c_corp_underpayment_at_exactly_100000_threshold_standard_rate() {
        // Statutory test: EXCEEDS $100,000 (strict greater-than)
        let mut input = baseline_underpayment_input();
        input.taxpayer_type = TaxpayerType::CCorporation;
        input.corporate_underpayment_total_dollars = 100_000; // = $100,000 = NOT exceeding
        input.amount_dollars = 100_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantStandardUnderpaymentRateFstrPlus3PercentagePoints
        );
        assert_eq!(output.applicable_rate_basis_points, 800);
    }

    #[test]
    fn c_corp_underpayment_at_100001_threshold_plus_1_large_corp_rate() {
        // Statutory boundary +1: $100,001 EXCEEDS $100,000
        let mut input = baseline_underpayment_input();
        input.taxpayer_type = TaxpayerType::CCorporation;
        input.corporate_underpayment_total_dollars = 100_001;
        input.amount_dollars = 100_001;
        input.large_corporate_applicable_date_status =
            LargeCorporateApplicableDateStatus::ThirtyDayWindowAfterDeficiencyNoticeHasPassed;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantLargeCorporateUnderpaymentRateFstrPlus5PercentagePointsUnderSection6621C
        );
        assert_eq!(output.applicable_rate_basis_points, 1_000);
    }

    #[test]
    fn other_entity_not_c_corp_treated_as_individual_rate() {
        let mut input = baseline_individual_overpayment_input();
        input.taxpayer_type = TaxpayerType::OtherEntityNotCCorporationTreatedAsIndividualForRate;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6621Mode::CompliantIndividualOverpaymentRateFstrPlus3PercentagePoints
        );
        assert_eq!(output.applicable_rate_basis_points, 800);
    }

    #[test]
    fn underpayment_zero_years_zero_interest() {
        let mut input = baseline_underpayment_input();
        input.number_of_full_years_interest_accrues = 0;
        let output = check(&input);
        assert_eq!(output.applicable_rate_basis_points, 800);
        assert_eq!(output.estimated_linear_interest_dollars, 0);
    }

    #[test]
    fn constants_pin_statutory_rates() {
        assert_eq!(IRC_6621_OVERPAYMENT_INDIVIDUAL_RATE_BASIS_POINTS, 300);
        assert_eq!(IRC_6621_OVERPAYMENT_CORPORATE_BASE_RATE_BASIS_POINTS, 200);
        assert_eq!(IRC_6621_OVERPAYMENT_CORPORATE_EXCESS_RATE_BASIS_POINTS, 50);
        assert_eq!(
            IRC_6621_OVERPAYMENT_CORPORATE_EXCESS_THRESHOLD_DOLLARS,
            10_000
        );
        assert_eq!(IRC_6621_UNDERPAYMENT_STANDARD_RATE_BASIS_POINTS, 300);
        assert_eq!(IRC_6621_LARGE_CORPORATE_UNDERPAYMENT_RATE_BASIS_POINTS, 500);
        assert_eq!(
            IRC_6621_LARGE_CORPORATE_UNDERPAYMENT_THRESHOLD_DOLLARS,
            100_000
        );
        assert_eq!(
            IRC_6621_LARGE_CORPORATE_APPLICABLE_DATE_DAYS_AFTER_NOTICE,
            30
        );
        assert_eq!(IRC_6621_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_section_6621_landmarks() {
        let output = check(&baseline_individual_overpayment_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 6621(a)(1)"));
        assert!(joined.contains("§ 6621(a)(2)"));
        assert!(joined.contains("§ 6621(b)"));
        assert!(joined.contains("§ 6621(c)"));
        assert!(joined.contains("§ 6621(d)"));
        assert!(joined.contains("§ 6622"));
        assert!(joined.contains("§ 6601"));
        assert!(joined.contains("§ 1274(d)"));
        assert!(joined.contains("3 percentage points"));
        assert!(joined.contains("5 percentage points"));
        assert!(joined.contains("0.5 percentage points"));
        assert!(joined.contains("$100,000"));
        assert!(joined.contains("$10,000"));
        assert!(joined.contains("FIRST MONTH IN EACH CALENDAR QUARTER"));
    }

    #[test]
    fn saturating_overflow_defense() {
        let mut input = baseline_underpayment_input();
        input.amount_dollars = u64::MAX;
        input.federal_short_term_rate_basis_points = u64::MAX;
        input.number_of_full_years_interest_accrues = u32::MAX;
        let output = check(&input);
        // Standard underpayment branch; rate computation saturates to u64::MAX
        assert_eq!(output.applicable_rate_basis_points, u64::MAX);
        assert_eq!(output.estimated_linear_interest_dollars, u64::MAX);
    }
}
