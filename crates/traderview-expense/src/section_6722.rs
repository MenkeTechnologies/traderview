//! IRC § 6722 — Failure to Furnish Correct Payee Statements
//! Tiered Penalty Module.
//!
//! Pure-compute check for IRC § 6722 payee-statement penalty
//! tiers. § 6722 is the structural parallel to § 6721 (built
//! iter 658) for the payer-side obligation to **furnish a
//! correct payee statement** (e.g., the recipient copy of
//! Form 1099-B, 1099-DIV, 1099-INT, 1099-K, 1099-NEC, 1099-
//! MISC, K-1, W-2) to the payee — distinct from the § 6721
//! obligation to **file with the IRS**. Same per-statement
//! amounts ($50/$100/$250 base; $60/$130/$340 for 2026 under
//! Rev. Proc. 2025-32), same small business exception, same
//! intentional disregard rule with NO MAXIMUM.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6722(a) General Rule**: any person required to
//!   furnish a payee statement who **fails to furnish such
//!   statement on or before the date prescribed therefor or
//!   fails to include all required information or includes
//!   incorrect information** shall pay a penalty per statement
//!   for each such failure ([Cornell LII 26 USC § 6722](https://www.law.cornell.edu/uscode/text/26/6722);
//!   [Bloomberg Tax Sec. 6722](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6722)).
//! - **IRC § 6722(b)(1) Tier 1 — Corrected Within 30 Days**:
//!   penalty reduced to **$50 per statement** (base statutory;
//!   $60 per statement inflation-adjusted for tax year 2026
//!   under Rev. Proc. 2025-32) with maximum **$500,000 per
//!   calendar year** (base; inflation-adjusted higher).
//! - **IRC § 6722(b)(2) Tier 2 — Corrected After 30 Days But
//!   By August 1**: penalty reduced to **$100 per statement**
//!   (base; $130 per statement for 2026) with maximum
//!   **$1,500,000 per calendar year** (base).
//! - **IRC § 6722(a)(1) Tier 3 — Not Corrected By August 1
//!   (or never)**: full penalty **$250 per statement** (base;
//!   $340 per statement for 2026) with maximum **$3,000,000
//!   per calendar year** (base; $4,191,500 for 2026 large
//!   filer).
//! - **IRC § 6722(d) Small Business Exception**: for any person
//!   with **average annual gross receipts of $5,000,000 or
//!   less** for the most recent 3 taxable years, the per-year
//!   maximums are reduced (Tier 1 $175,000; Tier 2 $500,000;
//!   Tier 3 $1,000,000 base; $1,397,000 for 2026 Tier 3).
//! - **IRC § 6722(e) Intentional Disregard**: if failures are
//!   due to **INTENTIONAL DISREGARD** of the requirement to
//!   furnish a payee statement, **§ 6722(b), (c), and (d)
//!   shall NOT apply**, and penalty is **GREATER OF $500 per
//!   statement ($680 for 2026) OR (a) 10 PERCENT of the
//!   aggregate amount required to be reported correctly for
//!   most payee statements OR (b) 5 PERCENT for certain
//!   specified statements**, with **NO MAXIMUM** dollar
//!   limitation ([IRS IRM 20.1.7 Information Return
//!   Penalties](https://www.irs.gov/irm/part20/irm_20-001-007r);
//!   [26 CFR § 301.6722-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-F/part-301/subpart-ECFRe7a848e7ecebb4b/subject-group-ECFR90240e9b8fcd266/section-301.6722-1)).
//! - **Treas. Reg. § 301.6722-1**: implementing regulation
//!   defining "payee statement" by reference to § 6724(d)(2),
//!   "date prescribed therefor," correction procedures, and
//!   reasonable-cause exception.
//! - **IRC § 6724 Reasonable Cause Waiver**: penalty waived if
//!   payer can show failure was due to reasonable cause AND
//!   not willful neglect.
//! - **§ 6721/§ 6722 Stacking**: same payment can trigger BOTH
//!   § 6721 (failure to file with IRS) AND § 6722 (failure to
//!   furnish to payee) penalties; § 6721 + § 6722 stacking
//!   commonly doubles penalty exposure for late or omitted
//!   1099 issuance cycles ([The Tax Adviser — Information
//!   return penalties: How to avoid or contest them](https://www.thetaxadviser.com/issues/2020/jan/avoid-contest-information-return-penalties/)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6722_TIER1_PER_STATEMENT_BASE_DOLLARS: u64 = 50;
pub const IRC_6722_TIER2_PER_STATEMENT_BASE_DOLLARS: u64 = 100;
pub const IRC_6722_TIER3_PER_STATEMENT_BASE_DOLLARS: u64 = 250;
pub const IRC_6722_INTENTIONAL_DISREGARD_PER_STATEMENT_BASE_DOLLARS: u64 = 500;
pub const IRC_6722_INTENTIONAL_DISREGARD_PCT_MOST_BASIS_POINTS: u64 = 1_000;
pub const IRC_6722_INTENTIONAL_DISREGARD_PCT_SPECIFIED_BASIS_POINTS: u64 = 500;
pub const IRC_6722_INTENTIONAL_DISREGARD_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_6722_TIER1_PER_STATEMENT_2026_DOLLARS: u64 = 60;
pub const IRC_6722_TIER2_PER_STATEMENT_2026_DOLLARS: u64 = 130;
pub const IRC_6722_TIER3_PER_STATEMENT_2026_DOLLARS: u64 = 340;
pub const IRC_6722_INTENTIONAL_DISREGARD_PER_STATEMENT_2026_DOLLARS: u64 = 680;
pub const IRC_6722_TIER1_MAX_BASE_DOLLARS: u64 = 500_000;
pub const IRC_6722_TIER2_MAX_BASE_DOLLARS: u64 = 1_500_000;
pub const IRC_6722_TIER3_MAX_BASE_DOLLARS: u64 = 3_000_000;
pub const IRC_6722_TIER3_MAX_2026_DOLLARS: u64 = 4_191_500;
pub const IRC_6722_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS: u64 = 175_000;
pub const IRC_6722_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS: u64 = 500_000;
pub const IRC_6722_TIER3_SMALL_BUSINESS_MAX_BASE_DOLLARS: u64 = 1_000_000;
pub const IRC_6722_TIER3_SMALL_BUSINESS_MAX_2026_DOLLARS: u64 = 1_397_000;
pub const IRC_6722_SMALL_BUSINESS_GROSS_RECEIPTS_THRESHOLD_DOLLARS: u64 = 5_000_000;
pub const IRC_6722_INTENTIONAL_DISREGARD_NO_MAX: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionTiming {
    NoFailureFurnishedOnTime,
    CorrectedWithin30DaysAfterPrescribedDate,
    CorrectedAfter30DaysButByAugust1,
    NotCorrectedByAugust1OrNotCorrected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilerSize {
    SmallBusinessAverageGrossReceiptsAtOrBelow5MillionFor3YearLookback,
    LargeCorporationOrLargeFilerAboveSmallBusinessThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PenaltyAmountVersion {
    BaseStatutoryAmounts,
    InflationAdjusted2026UnderRevProc202532,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentionalDisregardPctCategory {
    MostPayeeStatementsTenPercent,
    CertainSpecifiedStatementsFivePercent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6722Mode {
    NotApplicableNoFailureFurnishedOnTime,
    CompliantTier1Within30DaysCorrection,
    CompliantTier2AfterDay30ButByAugust1Correction,
    CompliantTier3NotCorrectedByAugust1FullPenalty,
    CompliantIntentionalDisregardGreaterOfPerStatementOrPctAggregateNoMax,
    CompliantReasonableCauseWaiverUnderSection6724,
    ViolationPayerFailedToFurnishButCorrectedWithin30Days,
    ViolationPayerFailedToFurnishAndCorrectedAfter30DaysButByAugust1,
    ViolationPayerFailedToFurnishAndDidNotCorrectByAugust1,
    ViolationIntentionalDisregardPenaltyOwedNoMax,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub correction_timing: CorrectionTiming,
    pub filer_size: FilerSize,
    pub penalty_amount_version: PenaltyAmountVersion,
    pub number_of_failed_statements: u64,
    pub intentional_disregard: bool,
    pub intentional_disregard_pct_category: IntentionalDisregardPctCategory,
    pub aggregate_amount_required_to_be_reported_dollars: u64,
    pub reasonable_cause_waiver_granted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6722Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub penalty_per_statement_dollars: u64,
    pub raw_penalty_dollars: u64,
    pub capped_penalty_dollars: u64,
}

pub type Section6722Input = Input;
pub type Section6722Output = Output;
pub type Section6722Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6722(a) — failure to furnish correct payee statement on or before prescribed date, failure to include all required information, or including incorrect information triggers per-statement penalty".to_string(),
        "IRC § 6722(b)(1) — Tier 1 reduced penalty when corrected within 30 days: $50 per statement base ($60 for 2026); maximum $500,000 base".to_string(),
        "IRC § 6722(b)(2) — Tier 2 reduced penalty when corrected after 30 days but by August 1: $100 per statement base ($130 for 2026); maximum $1,500,000 base".to_string(),
        "IRC § 6722(a)(1) — Tier 3 full penalty when not corrected by August 1: $250 per statement base ($340 for 2026); maximum $3,000,000 base ($4,191,500 for 2026)".to_string(),
        "IRC § 6722(d) — small business exception: average annual gross receipts ≤ $5,000,000 for most recent 3 taxable years; reduced per-year maximums (Tier 1 $175,000; Tier 2 $500,000; Tier 3 $1,000,000 base / $1,397,000 for 2026 Tier 3)".to_string(),
        "IRC § 6722(e) — INTENTIONAL DISREGARD: § 6722(b)/(c)/(d) NOT apply; penalty GREATER OF $500 per statement base ($680 for 2026) OR (a) 10 PERCENT of aggregate amount required to be reported correctly for most payee statements OR (b) 5 PERCENT for certain specified statements; NO MAXIMUM".to_string(),
        "Rev. Proc. 2025-32 — 2026 inflation-adjusted amounts (statements furnished 2027 covering 2026 reporting): general per-statement penalty $340; small-business maximum $1,397,000; large-corporation maximum $4,191,500".to_string(),
        "IRC § 6724 — reasonable cause waiver; penalty waived if payer shows failure due to events beyond control AND payer acted responsibly before and after failure; not willful neglect".to_string(),
        "Treas. Reg. § 301.6722-1 — implementing regulation defining 'payee statement' by reference to § 6724(d)(2), 'date prescribed therefor,' correction procedures, and reasonable-cause exception".to_string(),
        "IRC § 6721 — parallel penalty for failure to FILE correct information return with IRS (recipient-side complement; § 6722 covers payer-to-payee statement obligation)".to_string(),
        "§ 6721/§ 6722 Stacking — same payment can trigger BOTH § 6721 (failure to file with IRS) AND § 6722 (failure to furnish to payee); commonly doubles penalty exposure for late or omitted 1099 issuance cycles".to_string(),
        "IRS IRM 20.1.7 — Information Return Penalties operational guidance".to_string(),
        "Cornell LII 26 USC § 6722 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6722 — comprehensive code commentary".to_string(),
    ];

    if input.correction_timing == CorrectionTiming::NoFailureFurnishedOnTime
        && input.number_of_failed_statements == 0
    {
        return Output {
            mode: Section6722Mode::NotApplicableNoFailureFurnishedOnTime,
            statutory_basis: "IRC § 6722 — no failure occurred; payee statements furnished on time and accurately".to_string(),
            notes: "NOT APPLICABLE: no failure to furnish correct payee statement; § 6722 penalty does not apply.".to_string(),
            citations,
            penalty_per_statement_dollars: 0,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    if input.reasonable_cause_waiver_granted {
        return Output {
            mode: Section6722Mode::CompliantReasonableCauseWaiverUnderSection6724,
            statutory_basis: "IRC § 6724 — reasonable cause waiver granted".to_string(),
            notes: "COMPLIANT: § 6724 reasonable cause waiver granted; payer established failure was due to events beyond control and acted in responsible manner before and after; § 6722 penalty waived.".to_string(),
            citations,
            penalty_per_statement_dollars: 0,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    if input.intentional_disregard {
        let per_statement = match input.penalty_amount_version {
            PenaltyAmountVersion::BaseStatutoryAmounts => {
                IRC_6722_INTENTIONAL_DISREGARD_PER_STATEMENT_BASE_DOLLARS
            }
            PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532 => {
                IRC_6722_INTENTIONAL_DISREGARD_PER_STATEMENT_2026_DOLLARS
            }
        };
        let per_statement_penalty_total = input
            .number_of_failed_statements
            .saturating_mul(per_statement);
        let pct_basis_points = match input.intentional_disregard_pct_category {
            IntentionalDisregardPctCategory::MostPayeeStatementsTenPercent => {
                IRC_6722_INTENTIONAL_DISREGARD_PCT_MOST_BASIS_POINTS
            }
            IntentionalDisregardPctCategory::CertainSpecifiedStatementsFivePercent => {
                IRC_6722_INTENTIONAL_DISREGARD_PCT_SPECIFIED_BASIS_POINTS
            }
        };
        let pct_aggregate = input
            .aggregate_amount_required_to_be_reported_dollars
            .saturating_mul(pct_basis_points)
            / IRC_6722_INTENTIONAL_DISREGARD_BASIS_POINT_DENOMINATOR;
        let raw_penalty_dollars = per_statement_penalty_total.max(pct_aggregate);
        return Output {
            mode: Section6722Mode::ViolationIntentionalDisregardPenaltyOwedNoMax,
            statutory_basis: "IRC § 6722(e) — intentional disregard penalty NO MAXIMUM".to_string(),
            notes: format!(
                "VIOLATION: intentional disregard of § 6722 furnishing requirement; penalty = GREATER OF ${} per statement × {} = ${} OR {} % of aggregate ${} = ${}; NO MAXIMUM; payer owes ${}.",
                per_statement,
                input.number_of_failed_statements,
                per_statement_penalty_total,
                pct_basis_points / 100,
                input.aggregate_amount_required_to_be_reported_dollars,
                pct_aggregate,
                raw_penalty_dollars
            ),
            citations,
            penalty_per_statement_dollars: per_statement,
            raw_penalty_dollars,
            capped_penalty_dollars: raw_penalty_dollars,
        };
    }

    let (per_statement, max_general, max_small_biz, mode_violation, mode_compliant) =
        match (input.correction_timing, input.penalty_amount_version) {
            (CorrectionTiming::CorrectedWithin30DaysAfterPrescribedDate, PenaltyAmountVersion::BaseStatutoryAmounts) => (
                IRC_6722_TIER1_PER_STATEMENT_BASE_DOLLARS,
                IRC_6722_TIER1_MAX_BASE_DOLLARS,
                IRC_6722_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6722Mode::ViolationPayerFailedToFurnishButCorrectedWithin30Days,
                Section6722Mode::CompliantTier1Within30DaysCorrection,
            ),
            (CorrectionTiming::CorrectedWithin30DaysAfterPrescribedDate, PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532) => (
                IRC_6722_TIER1_PER_STATEMENT_2026_DOLLARS,
                IRC_6722_TIER1_MAX_BASE_DOLLARS,
                IRC_6722_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6722Mode::ViolationPayerFailedToFurnishButCorrectedWithin30Days,
                Section6722Mode::CompliantTier1Within30DaysCorrection,
            ),
            (CorrectionTiming::CorrectedAfter30DaysButByAugust1, PenaltyAmountVersion::BaseStatutoryAmounts) => (
                IRC_6722_TIER2_PER_STATEMENT_BASE_DOLLARS,
                IRC_6722_TIER2_MAX_BASE_DOLLARS,
                IRC_6722_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6722Mode::ViolationPayerFailedToFurnishAndCorrectedAfter30DaysButByAugust1,
                Section6722Mode::CompliantTier2AfterDay30ButByAugust1Correction,
            ),
            (CorrectionTiming::CorrectedAfter30DaysButByAugust1, PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532) => (
                IRC_6722_TIER2_PER_STATEMENT_2026_DOLLARS,
                IRC_6722_TIER2_MAX_BASE_DOLLARS,
                IRC_6722_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6722Mode::ViolationPayerFailedToFurnishAndCorrectedAfter30DaysButByAugust1,
                Section6722Mode::CompliantTier2AfterDay30ButByAugust1Correction,
            ),
            (CorrectionTiming::NotCorrectedByAugust1OrNotCorrected, PenaltyAmountVersion::BaseStatutoryAmounts) => (
                IRC_6722_TIER3_PER_STATEMENT_BASE_DOLLARS,
                IRC_6722_TIER3_MAX_BASE_DOLLARS,
                IRC_6722_TIER3_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6722Mode::ViolationPayerFailedToFurnishAndDidNotCorrectByAugust1,
                Section6722Mode::CompliantTier3NotCorrectedByAugust1FullPenalty,
            ),
            (CorrectionTiming::NotCorrectedByAugust1OrNotCorrected, PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532) => (
                IRC_6722_TIER3_PER_STATEMENT_2026_DOLLARS,
                IRC_6722_TIER3_MAX_2026_DOLLARS,
                IRC_6722_TIER3_SMALL_BUSINESS_MAX_2026_DOLLARS,
                Section6722Mode::ViolationPayerFailedToFurnishAndDidNotCorrectByAugust1,
                Section6722Mode::CompliantTier3NotCorrectedByAugust1FullPenalty,
            ),
            (CorrectionTiming::NoFailureFurnishedOnTime, _) => {
                return Output {
                    mode: Section6722Mode::NotApplicableNoFailureFurnishedOnTime,
                    statutory_basis: "IRC § 6722 — no failure occurred".to_string(),
                    notes: "NOT APPLICABLE: no failure to furnish payee statement; § 6722 penalty does not apply.".to_string(),
                    citations,
                    penalty_per_statement_dollars: 0,
                    raw_penalty_dollars: 0,
                    capped_penalty_dollars: 0,
                };
            }
        };

    let raw_penalty_dollars = input
        .number_of_failed_statements
        .saturating_mul(per_statement);
    let cap = match input.filer_size {
        FilerSize::SmallBusinessAverageGrossReceiptsAtOrBelow5MillionFor3YearLookback => max_small_biz,
        FilerSize::LargeCorporationOrLargeFilerAboveSmallBusinessThreshold => max_general,
    };
    let capped_penalty_dollars = raw_penalty_dollars.min(cap);

    if raw_penalty_dollars > cap {
        return Output {
            mode: mode_violation,
            statutory_basis: format!(
                "IRC § 6722 — {} per-statement penalty; capped at {} annual maximum",
                per_statement, cap
            ),
            notes: format!(
                "VIOLATION: {} failed statements × ${} per statement = ${} raw penalty; capped at ${} annual maximum for {:?}.",
                input.number_of_failed_statements,
                per_statement,
                raw_penalty_dollars,
                cap,
                input.filer_size
            ),
            citations,
            penalty_per_statement_dollars: per_statement,
            raw_penalty_dollars,
            capped_penalty_dollars,
        };
    }

    Output {
        mode: mode_compliant,
        statutory_basis: format!(
            "IRC § 6722 — {} per-statement penalty within annual maximum",
            per_statement
        ),
        notes: format!(
            "COMPLIANT: payer accepted § 6722 penalty assessment of ${} per statement × {} statements = ${}; below ${} annual maximum for {:?}.",
            per_statement,
            input.number_of_failed_statements,
            raw_penalty_dollars,
            cap,
            input.filer_size
        ),
        citations,
        penalty_per_statement_dollars: per_statement,
        raw_penalty_dollars,
        capped_penalty_dollars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_tier1_small_biz() -> Input {
        Input {
            correction_timing: CorrectionTiming::CorrectedWithin30DaysAfterPrescribedDate,
            filer_size: FilerSize::SmallBusinessAverageGrossReceiptsAtOrBelow5MillionFor3YearLookback,
            penalty_amount_version: PenaltyAmountVersion::BaseStatutoryAmounts,
            number_of_failed_statements: 100,
            intentional_disregard: false,
            intentional_disregard_pct_category:
                IntentionalDisregardPctCategory::MostPayeeStatementsTenPercent,
            aggregate_amount_required_to_be_reported_dollars: 1_000_000,
            reasonable_cause_waiver_granted: false,
        }
    }

    #[test]
    fn no_failure_not_applicable() {
        let input = Input {
            correction_timing: CorrectionTiming::NoFailureFurnishedOnTime,
            number_of_failed_statements: 0,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section6722Mode::NotApplicableNoFailureFurnishedOnTime);
    }

    #[test]
    fn reasonable_cause_waiver_compliant() {
        let input = Input {
            reasonable_cause_waiver_granted: true,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6722Mode::CompliantReasonableCauseWaiverUnderSection6724
        );
    }

    #[test]
    fn tier1_30_day_correction_compliant_within_cap() {
        let result = check(&baseline_tier1_small_biz());
        assert_eq!(
            result.mode,
            Section6722Mode::CompliantTier1Within30DaysCorrection
        );
        assert_eq!(result.penalty_per_statement_dollars, 50);
        assert_eq!(result.raw_penalty_dollars, 5_000);
    }

    #[test]
    fn tier1_30_day_correction_small_biz_above_cap_capped() {
        let input = Input {
            number_of_failed_statements: 5_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6722Mode::ViolationPayerFailedToFurnishButCorrectedWithin30Days
        );
        assert_eq!(result.raw_penalty_dollars, 250_000);
        assert_eq!(result.capped_penalty_dollars, 175_000);
    }

    #[test]
    fn tier2_after_30_days_by_aug_1_compliant() {
        let input = Input {
            correction_timing: CorrectionTiming::CorrectedAfter30DaysButByAugust1,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6722Mode::CompliantTier2AfterDay30ButByAugust1Correction
        );
        assert_eq!(result.penalty_per_statement_dollars, 100);
        assert_eq!(result.raw_penalty_dollars, 10_000);
    }

    #[test]
    fn tier3_not_corrected_compliant_within_small_biz_cap() {
        let input = Input {
            correction_timing: CorrectionTiming::NotCorrectedByAugust1OrNotCorrected,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6722Mode::CompliantTier3NotCorrectedByAugust1FullPenalty
        );
        assert_eq!(result.penalty_per_statement_dollars, 250);
        assert_eq!(result.raw_penalty_dollars, 25_000);
    }

    #[test]
    fn tier3_not_corrected_above_small_biz_cap_capped() {
        let input = Input {
            correction_timing: CorrectionTiming::NotCorrectedByAugust1OrNotCorrected,
            number_of_failed_statements: 10_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 2_500_000);
        assert_eq!(result.capped_penalty_dollars, 1_000_000);
    }

    #[test]
    fn tier3_2026_inflation_adjusted_large_filer_above_cap() {
        let input = Input {
            correction_timing: CorrectionTiming::NotCorrectedByAugust1OrNotCorrected,
            penalty_amount_version: PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532,
            filer_size: FilerSize::LargeCorporationOrLargeFilerAboveSmallBusinessThreshold,
            number_of_failed_statements: 20_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.penalty_per_statement_dollars, 340);
        assert_eq!(result.raw_penalty_dollars, 6_800_000);
        assert_eq!(result.capped_penalty_dollars, 4_191_500);
    }

    #[test]
    fn intentional_disregard_per_statement_greater_than_10_pct_aggregate() {
        let input = Input {
            number_of_failed_statements: 1_000,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: 100_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6722Mode::ViolationIntentionalDisregardPenaltyOwedNoMax
        );
        assert_eq!(result.raw_penalty_dollars, 500_000);
    }

    #[test]
    fn intentional_disregard_10_pct_aggregate_greater_than_per_statement() {
        let input = Input {
            number_of_failed_statements: 10,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: 100_000_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 10_000_000);
    }

    #[test]
    fn intentional_disregard_5_pct_specified_statements() {
        let input = Input {
            number_of_failed_statements: 10,
            intentional_disregard: true,
            intentional_disregard_pct_category:
                IntentionalDisregardPctCategory::CertainSpecifiedStatementsFivePercent,
            aggregate_amount_required_to_be_reported_dollars: 100_000_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 5_000_000);
    }

    #[test]
    fn intentional_disregard_no_maximum_cap() {
        let input = Input {
            number_of_failed_statements: 1_000_000,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: 0,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 500_000_000);
        assert_eq!(result.capped_penalty_dollars, 500_000_000);
    }

    #[test]
    fn intentional_disregard_2026_inflation_adjusted_per_statement() {
        let input = Input {
            number_of_failed_statements: 100,
            intentional_disregard: true,
            penalty_amount_version: PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.penalty_per_statement_dollars, 680);
        assert_eq!(result.raw_penalty_dollars, 100_000);
    }

    #[test]
    fn citations_pin_section_6722_tiers_and_stacking() {
        let result = check(&baseline_tier1_small_biz());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 6722(a)"));
        assert!(joined.contains("IRC § 6722(b)(1)"));
        assert!(joined.contains("IRC § 6722(b)(2)"));
        assert!(joined.contains("IRC § 6722(d)"));
        assert!(joined.contains("IRC § 6722(e)"));
        assert!(joined.contains("Tier 1"));
        assert!(joined.contains("Tier 2"));
        assert!(joined.contains("Tier 3"));
        assert!(joined.contains("$50"));
        assert!(joined.contains("$100"));
        assert!(joined.contains("$250"));
        assert!(joined.contains("$500"));
        assert!(joined.contains("$60"));
        assert!(joined.contains("$130"));
        assert!(joined.contains("$340"));
        assert!(joined.contains("$680"));
        assert!(joined.contains("INTENTIONAL DISREGARD"));
        assert!(joined.contains("10 PERCENT"));
        assert!(joined.contains("5 PERCENT"));
        assert!(joined.contains("NO MAXIMUM"));
        assert!(joined.contains("$5,000,000"));
        assert!(joined.contains("Rev. Proc. 2025-32"));
        assert!(joined.contains("IRC § 6724"));
        assert!(joined.contains("Treas. Reg. § 301.6722-1"));
        assert!(joined.contains("IRC § 6721"));
        assert!(joined.contains("§ 6721/§ 6722 Stacking"));
        assert!(joined.contains("IRS IRM 20.1.7"));
    }

    #[test]
    fn constant_pin_base_amounts_and_inflation_adjusted_2026() {
        assert_eq!(IRC_6722_TIER1_PER_STATEMENT_BASE_DOLLARS, 50);
        assert_eq!(IRC_6722_TIER2_PER_STATEMENT_BASE_DOLLARS, 100);
        assert_eq!(IRC_6722_TIER3_PER_STATEMENT_BASE_DOLLARS, 250);
        assert_eq!(IRC_6722_INTENTIONAL_DISREGARD_PER_STATEMENT_BASE_DOLLARS, 500);
        assert_eq!(IRC_6722_INTENTIONAL_DISREGARD_PCT_MOST_BASIS_POINTS, 1_000);
        assert_eq!(IRC_6722_INTENTIONAL_DISREGARD_PCT_SPECIFIED_BASIS_POINTS, 500);
        assert_eq!(IRC_6722_INTENTIONAL_DISREGARD_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_6722_TIER1_PER_STATEMENT_2026_DOLLARS, 60);
        assert_eq!(IRC_6722_TIER2_PER_STATEMENT_2026_DOLLARS, 130);
        assert_eq!(IRC_6722_TIER3_PER_STATEMENT_2026_DOLLARS, 340);
        assert_eq!(IRC_6722_INTENTIONAL_DISREGARD_PER_STATEMENT_2026_DOLLARS, 680);
        assert_eq!(IRC_6722_TIER1_MAX_BASE_DOLLARS, 500_000);
        assert_eq!(IRC_6722_TIER2_MAX_BASE_DOLLARS, 1_500_000);
        assert_eq!(IRC_6722_TIER3_MAX_BASE_DOLLARS, 3_000_000);
        assert_eq!(IRC_6722_TIER3_MAX_2026_DOLLARS, 4_191_500);
        assert_eq!(IRC_6722_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS, 175_000);
        assert_eq!(IRC_6722_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS, 500_000);
        assert_eq!(IRC_6722_TIER3_SMALL_BUSINESS_MAX_BASE_DOLLARS, 1_000_000);
        assert_eq!(IRC_6722_TIER3_SMALL_BUSINESS_MAX_2026_DOLLARS, 1_397_000);
        assert_eq!(IRC_6722_SMALL_BUSINESS_GROSS_RECEIPTS_THRESHOLD_DOLLARS, 5_000_000);
        assert_eq!(IRC_6722_INTENTIONAL_DISREGARD_NO_MAX, u64::MAX);
    }

    #[test]
    fn saturating_overflow_defense_extreme_statements_intentional() {
        let input = Input {
            number_of_failed_statements: u64::MAX,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: u64::MAX,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6722Mode::ViolationIntentionalDisregardPenaltyOwedNoMax
        );
    }
}
