//! IRC § 6721 — Failure to File Correct Information Returns
//! Tiered Penalty Module.
//!
//! Pure-compute check for IRC § 6721 information-return penalty
//! tiers. § 6721 imposes per-return penalties on payers that
//! fail to timely file complete and accurate information
//! returns (Forms 1099, 1098, W-2, etc.) with the IRS. Penalty
//! amount depends on **how long after the required filing date
//! the failure is corrected**, with three tiers culminating in
//! an UNCAPPED intentional-disregard penalty. Trader-critical
//! because every 1099-B, 1099-DIV, 1099-INT, 1099-K, 1099-NEC,
//! 1099-MISC, K-1 filed by a trader-payor (trading shop, family
//! LP, family office GP) is subject to § 6721 penalty exposure;
//! companion to § 6109 (TIN requirements; built iter 656) and
//! § 6041 (information reporting; built iter 620).
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6721(a) General Rule**: any person required to file
//!   an information return who **fails to file such return on
//!   or before the required filing date, fails to include all
//!   required information, or includes incorrect information**
//!   shall pay a penalty per return for each such failure
//!   ([Cornell LII 26 USC § 6721](https://www.law.cornell.edu/uscode/text/26/6721);
//!   [Bloomberg Tax Sec. 6721](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6721)).
//! - **IRC § 6721(b)(1) Tier 1 — Corrected Within 30 Days**:
//!   penalty reduced to **$50 per return** (base statutory;
//!   $60 per return inflation-adjusted for tax year 2026 under
//!   Rev. Proc. 2025-32) with maximum **$500,000 per calendar
//!   year** (base; inflation-adjusted higher).
//! - **IRC § 6721(b)(2) Tier 2 — Corrected After 30 Days But By
//!   August 1**: penalty reduced to **$100 per return** (base;
//!   $130 per return for 2026) with maximum **$1,500,000 per
//!   calendar year** (base; inflation-adjusted higher).
//! - **IRC § 6721(a)(1) Tier 3 — Not Corrected By August 1 (or
//!   never)**: full penalty **$250 per return** (base; $340
//!   per return for 2026) with maximum **$3,000,000 per
//!   calendar year** (base; inflation-adjusted higher).
//! - **IRC § 6721(d) Small Business Exception**: for any person
//!   with **average annual gross receipts of $5,000,000 or
//!   less** for the most recent 3 taxable years, the per-year
//!   maximums are reduced (Tier 1: $175,000; Tier 2: $500,000;
//!   Tier 3: $1,000,000 base statutory; inflation-adjusted
//!   higher for 2026).
//! - **IRC § 6721(e) Intentional Disregard**: if failures are
//!   due to **INTENTIONAL DISREGARD** of filing requirement or
//!   correct-information requirement, **§ 6721(b), (c), and (d)
//!   shall NOT apply**, and penalty is **GREATER OF $500 per
//!   return ($680 for 2026) OR 10 PERCENT of aggregate amount
//!   required to be reported correctly**, with **NO MAXIMUM**
//!   dollar limitation ([IRS IRM 20.1.7 Information Return
//!   Penalties](https://www.irs.gov/irm/part20/irm_20-001-007r)).
//! - **Inflation Adjustment** under Rev. Proc. 2025-32 (for
//!   returns filed in 2027 covering 2026 reporting): general
//!   per-return penalty **$340**; small-business maximum
//!   **$1,397,000**; large-corporation maximum **$4,191,500**
//!   ([IRS Rev. Proc. 2025-32](https://www.irs.gov/pub/irs-drop/rp-25-32.pdf);
//!   [Current Federal Tax Developments — 2026 Inflation
//!   Adjustments](https://www.currentfederaltaxdevelopments.com/blog/2025/10/9/2026-inflation-adjustments-for-tax-professionals-revenue-procedure-2025-32-analysis)).
//! - **Treas. Reg. § 301.6721-1**: implementing regulation
//!   defining "information return" by reference to § 6724(d)(1),
//!   "required filing date," correction procedures, and
//!   reasonable-cause exception.
//! - **§ 6724 Reasonable Cause Waiver**: penalty waived if
//!   payer can show failure was due to reasonable cause AND
//!   not willful neglect; must establish that failure was due
//!   to events beyond payer's control AND payer acted in a
//!   responsible manner before and after failure.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6721_TIER1_PER_RETURN_BASE_DOLLARS: u64 = 50;
pub const IRC_6721_TIER2_PER_RETURN_BASE_DOLLARS: u64 = 100;
pub const IRC_6721_TIER3_PER_RETURN_BASE_DOLLARS: u64 = 250;
pub const IRC_6721_INTENTIONAL_DISREGARD_PER_RETURN_BASE_DOLLARS: u64 = 500;
pub const IRC_6721_INTENTIONAL_DISREGARD_PCT_BASIS_POINTS: u64 = 1_000;
pub const IRC_6721_INTENTIONAL_DISREGARD_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_6721_TIER1_PER_RETURN_2026_DOLLARS: u64 = 60;
pub const IRC_6721_TIER2_PER_RETURN_2026_DOLLARS: u64 = 130;
pub const IRC_6721_TIER3_PER_RETURN_2026_DOLLARS: u64 = 340;
pub const IRC_6721_INTENTIONAL_DISREGARD_PER_RETURN_2026_DOLLARS: u64 = 680;
pub const IRC_6721_TIER1_MAX_BASE_DOLLARS: u64 = 500_000;
pub const IRC_6721_TIER2_MAX_BASE_DOLLARS: u64 = 1_500_000;
pub const IRC_6721_TIER3_MAX_BASE_DOLLARS: u64 = 3_000_000;
pub const IRC_6721_TIER3_MAX_2026_DOLLARS: u64 = 4_191_500;
pub const IRC_6721_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS: u64 = 175_000;
pub const IRC_6721_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS: u64 = 500_000;
pub const IRC_6721_TIER3_SMALL_BUSINESS_MAX_BASE_DOLLARS: u64 = 1_000_000;
pub const IRC_6721_TIER3_SMALL_BUSINESS_MAX_2026_DOLLARS: u64 = 1_397_000;
pub const IRC_6721_SMALL_BUSINESS_GROSS_RECEIPTS_THRESHOLD_DOLLARS: u64 = 5_000_000;
pub const IRC_6721_INTENTIONAL_DISREGARD_NO_MAX: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionTiming {
    NoFailureFiledOnTime,
    CorrectedWithin30DaysAfterRequiredFilingDate,
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
pub enum Section6721Mode {
    NotApplicableNoFailureFiledOnTime,
    CompliantTier1Within30DaysCorrection,
    CompliantTier2AfterDay30ButByAugust1Correction,
    CompliantTier3NotCorrectedByAugust1FullPenalty,
    CompliantIntentionalDisregardGreaterOfPerReturnOr10PercentAggregateNoMax,
    CompliantReasonableCauseWaiverUnderSection6724,
    ViolationPayerFailedToFileButCorrectedWithin30Days,
    ViolationPayerFailedToFileAndCorrectedAfter30DaysButByAugust1,
    ViolationPayerFailedToFileAndDidNotCorrectByAugust1,
    ViolationIntentionalDisregardPenaltyOwedNoMax,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub correction_timing: CorrectionTiming,
    pub filer_size: FilerSize,
    pub penalty_amount_version: PenaltyAmountVersion,
    pub number_of_failed_returns: u64,
    pub intentional_disregard: bool,
    pub aggregate_amount_required_to_be_reported_dollars: u64,
    pub reasonable_cause_waiver_granted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6721Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub penalty_per_return_dollars: u64,
    pub raw_penalty_dollars: u64,
    pub capped_penalty_dollars: u64,
}

pub type Section6721Input = Input;
pub type Section6721Output = Output;
pub type Section6721Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6721(a) — failure to file correct information return on or before required filing date, failure to include all required information, or including incorrect information triggers per-return penalty".to_string(),
        "IRC § 6721(b)(1) — Tier 1 reduced penalty when corrected within 30 days: $50 per return base ($60 for 2026); maximum $500,000 base ($664,500 for 2026)".to_string(),
        "IRC § 6721(b)(2) — Tier 2 reduced penalty when corrected after 30 days but by August 1: $100 per return base ($130 for 2026); maximum $1,500,000 base ($1,993,500 for 2026)".to_string(),
        "IRC § 6721(a)(1) — Tier 3 full penalty when not corrected by August 1: $250 per return base ($340 for 2026); maximum $3,000,000 base ($4,191,500 for 2026)".to_string(),
        "IRC § 6721(d) — small business exception: average annual gross receipts ≤ $5,000,000 for most recent 3 taxable years; reduced per-year maximums (Tier 1 $175,000; Tier 2 $500,000; Tier 3 $1,000,000 base / $1,397,000 for 2026 Tier 3)".to_string(),
        "IRC § 6721(e) — INTENTIONAL DISREGARD: § 6721(b)/(c)/(d) NOT apply; penalty is GREATER OF $500 per return base ($680 for 2026) OR 10 PERCENT of aggregate amount required to be reported correctly; NO MAXIMUM".to_string(),
        "Rev. Proc. 2025-32 — 2026 inflation-adjusted amounts (returns filed 2027 covering 2026 reporting): general per-return penalty $340; small-business maximum $1,397,000; large-corporation maximum $4,191,500".to_string(),
        "IRC § 6724 — reasonable cause waiver; penalty waived if payer shows failure due to events beyond control AND payer acted responsibly before and after failure; not willful neglect".to_string(),
        "Treas. Reg. § 301.6721-1 — implementing regulation defining 'information return' by reference to § 6724(d)(1), 'required filing date,' correction procedures, and reasonable-cause exception".to_string(),
        "IRC § 6722 — parallel penalty for failure to furnish correct payee statement (recipient copy); same tier structure".to_string(),
        "IRC § 6723 — $50 per other reporting failure with $100,000 annual maximum".to_string(),
        "IRS IRM 20.1.7 — Information Return Penalties operational guidance".to_string(),
        "Cornell LII 26 USC § 6721 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6721 — comprehensive code commentary".to_string(),
    ];

    if input.correction_timing == CorrectionTiming::NoFailureFiledOnTime
        && input.number_of_failed_returns == 0
    {
        return Output {
            mode: Section6721Mode::NotApplicableNoFailureFiledOnTime,
            statutory_basis: "IRC § 6721 — no failure occurred; returns filed on time and accurately".to_string(),
            notes: "NOT APPLICABLE: no failure to file or correct information return; § 6721 penalty does not apply.".to_string(),
            citations,
            penalty_per_return_dollars: 0,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    if input.reasonable_cause_waiver_granted {
        return Output {
            mode: Section6721Mode::CompliantReasonableCauseWaiverUnderSection6724,
            statutory_basis: "IRC § 6724 — reasonable cause waiver granted".to_string(),
            notes: "COMPLIANT: § 6724 reasonable cause waiver granted; payer established failure was due to events beyond control and acted in responsible manner before and after; § 6721 penalty waived.".to_string(),
            citations,
            penalty_per_return_dollars: 0,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    if input.intentional_disregard {
        let per_return = match input.penalty_amount_version {
            PenaltyAmountVersion::BaseStatutoryAmounts => {
                IRC_6721_INTENTIONAL_DISREGARD_PER_RETURN_BASE_DOLLARS
            }
            PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532 => {
                IRC_6721_INTENTIONAL_DISREGARD_PER_RETURN_2026_DOLLARS
            }
        };
        let per_return_penalty_total = input
            .number_of_failed_returns
            .saturating_mul(per_return);
        let ten_percent_aggregate = input
            .aggregate_amount_required_to_be_reported_dollars
            .saturating_mul(IRC_6721_INTENTIONAL_DISREGARD_PCT_BASIS_POINTS)
            / IRC_6721_INTENTIONAL_DISREGARD_BASIS_POINT_DENOMINATOR;
        let raw_penalty_dollars = per_return_penalty_total.max(ten_percent_aggregate);
        return Output {
            mode: Section6721Mode::ViolationIntentionalDisregardPenaltyOwedNoMax,
            statutory_basis: "IRC § 6721(e) — intentional disregard penalty NO MAXIMUM".to_string(),
            notes: format!(
                "VIOLATION: intentional disregard of § 6721 filing requirement; penalty = GREATER OF ${} per return × {} = ${} OR 10 % of aggregate ${} = ${}; NO MAXIMUM; payer owes ${}.",
                per_return,
                input.number_of_failed_returns,
                per_return_penalty_total,
                input.aggregate_amount_required_to_be_reported_dollars,
                ten_percent_aggregate,
                raw_penalty_dollars
            ),
            citations,
            penalty_per_return_dollars: per_return,
            raw_penalty_dollars,
            capped_penalty_dollars: raw_penalty_dollars,
        };
    }

    let (per_return, max_general, max_small_biz, mode_violation, mode_compliant) =
        match (input.correction_timing, input.penalty_amount_version) {
            (CorrectionTiming::CorrectedWithin30DaysAfterRequiredFilingDate, PenaltyAmountVersion::BaseStatutoryAmounts) => (
                IRC_6721_TIER1_PER_RETURN_BASE_DOLLARS,
                IRC_6721_TIER1_MAX_BASE_DOLLARS,
                IRC_6721_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6721Mode::ViolationPayerFailedToFileButCorrectedWithin30Days,
                Section6721Mode::CompliantTier1Within30DaysCorrection,
            ),
            (CorrectionTiming::CorrectedWithin30DaysAfterRequiredFilingDate, PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532) => (
                IRC_6721_TIER1_PER_RETURN_2026_DOLLARS,
                IRC_6721_TIER1_MAX_BASE_DOLLARS,
                IRC_6721_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6721Mode::ViolationPayerFailedToFileButCorrectedWithin30Days,
                Section6721Mode::CompliantTier1Within30DaysCorrection,
            ),
            (CorrectionTiming::CorrectedAfter30DaysButByAugust1, PenaltyAmountVersion::BaseStatutoryAmounts) => (
                IRC_6721_TIER2_PER_RETURN_BASE_DOLLARS,
                IRC_6721_TIER2_MAX_BASE_DOLLARS,
                IRC_6721_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6721Mode::ViolationPayerFailedToFileAndCorrectedAfter30DaysButByAugust1,
                Section6721Mode::CompliantTier2AfterDay30ButByAugust1Correction,
            ),
            (CorrectionTiming::CorrectedAfter30DaysButByAugust1, PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532) => (
                IRC_6721_TIER2_PER_RETURN_2026_DOLLARS,
                IRC_6721_TIER2_MAX_BASE_DOLLARS,
                IRC_6721_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6721Mode::ViolationPayerFailedToFileAndCorrectedAfter30DaysButByAugust1,
                Section6721Mode::CompliantTier2AfterDay30ButByAugust1Correction,
            ),
            (CorrectionTiming::NotCorrectedByAugust1OrNotCorrected, PenaltyAmountVersion::BaseStatutoryAmounts) => (
                IRC_6721_TIER3_PER_RETURN_BASE_DOLLARS,
                IRC_6721_TIER3_MAX_BASE_DOLLARS,
                IRC_6721_TIER3_SMALL_BUSINESS_MAX_BASE_DOLLARS,
                Section6721Mode::ViolationPayerFailedToFileAndDidNotCorrectByAugust1,
                Section6721Mode::CompliantTier3NotCorrectedByAugust1FullPenalty,
            ),
            (CorrectionTiming::NotCorrectedByAugust1OrNotCorrected, PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532) => (
                IRC_6721_TIER3_PER_RETURN_2026_DOLLARS,
                IRC_6721_TIER3_MAX_2026_DOLLARS,
                IRC_6721_TIER3_SMALL_BUSINESS_MAX_2026_DOLLARS,
                Section6721Mode::ViolationPayerFailedToFileAndDidNotCorrectByAugust1,
                Section6721Mode::CompliantTier3NotCorrectedByAugust1FullPenalty,
            ),
            (CorrectionTiming::NoFailureFiledOnTime, _) => {
                return Output {
                    mode: Section6721Mode::NotApplicableNoFailureFiledOnTime,
                    statutory_basis: "IRC § 6721 — no failure occurred".to_string(),
                    notes: "NOT APPLICABLE: no failure to file information return; § 6721 penalty does not apply.".to_string(),
                    citations,
                    penalty_per_return_dollars: 0,
                    raw_penalty_dollars: 0,
                    capped_penalty_dollars: 0,
                };
            }
        };

    let raw_penalty_dollars = input.number_of_failed_returns.saturating_mul(per_return);
    let cap = match input.filer_size {
        FilerSize::SmallBusinessAverageGrossReceiptsAtOrBelow5MillionFor3YearLookback => max_small_biz,
        FilerSize::LargeCorporationOrLargeFilerAboveSmallBusinessThreshold => max_general,
    };
    let capped_penalty_dollars = raw_penalty_dollars.min(cap);

    if raw_penalty_dollars > cap {
        return Output {
            mode: mode_violation,
            statutory_basis: format!(
                "IRC § 6721 — {} per-return penalty; capped at {} annual maximum",
                per_return, cap
            ),
            notes: format!(
                "VIOLATION: {} failed returns × ${} per return = ${} raw penalty; capped at ${} annual maximum for {:?}.",
                input.number_of_failed_returns,
                per_return,
                raw_penalty_dollars,
                cap,
                input.filer_size
            ),
            citations,
            penalty_per_return_dollars: per_return,
            raw_penalty_dollars,
            capped_penalty_dollars,
        };
    }

    Output {
        mode: mode_compliant,
        statutory_basis: format!(
            "IRC § 6721 — {} per-return penalty within annual maximum",
            per_return
        ),
        notes: format!(
            "COMPLIANT: payer accepted § 6721 penalty assessment of ${} per return × {} returns = ${}; below ${} annual maximum for {:?}.",
            per_return,
            input.number_of_failed_returns,
            raw_penalty_dollars,
            cap,
            input.filer_size
        ),
        citations,
        penalty_per_return_dollars: per_return,
        raw_penalty_dollars,
        capped_penalty_dollars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_tier1_small_biz() -> Input {
        Input {
            correction_timing: CorrectionTiming::CorrectedWithin30DaysAfterRequiredFilingDate,
            filer_size: FilerSize::SmallBusinessAverageGrossReceiptsAtOrBelow5MillionFor3YearLookback,
            penalty_amount_version: PenaltyAmountVersion::BaseStatutoryAmounts,
            number_of_failed_returns: 100,
            intentional_disregard: false,
            aggregate_amount_required_to_be_reported_dollars: 1_000_000,
            reasonable_cause_waiver_granted: false,
        }
    }

    #[test]
    fn no_failure_not_applicable() {
        let input = Input {
            correction_timing: CorrectionTiming::NoFailureFiledOnTime,
            number_of_failed_returns: 0,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section6721Mode::NotApplicableNoFailureFiledOnTime);
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
            Section6721Mode::CompliantReasonableCauseWaiverUnderSection6724
        );
    }

    #[test]
    fn tier1_30_day_correction_compliant_within_cap() {
        let result = check(&baseline_tier1_small_biz());
        assert_eq!(
            result.mode,
            Section6721Mode::CompliantTier1Within30DaysCorrection
        );
        assert_eq!(result.penalty_per_return_dollars, 50);
        assert_eq!(result.raw_penalty_dollars, 5_000);
        assert_eq!(result.capped_penalty_dollars, 5_000);
    }

    #[test]
    fn tier1_30_day_correction_small_biz_above_cap_capped() {
        let input = Input {
            number_of_failed_returns: 5_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6721Mode::ViolationPayerFailedToFileButCorrectedWithin30Days
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
            Section6721Mode::CompliantTier2AfterDay30ButByAugust1Correction
        );
        assert_eq!(result.penalty_per_return_dollars, 100);
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
            Section6721Mode::CompliantTier3NotCorrectedByAugust1FullPenalty
        );
        assert_eq!(result.penalty_per_return_dollars, 250);
        assert_eq!(result.raw_penalty_dollars, 25_000);
    }

    #[test]
    fn tier3_not_corrected_above_small_biz_cap_capped() {
        let input = Input {
            correction_timing: CorrectionTiming::NotCorrectedByAugust1OrNotCorrected,
            number_of_failed_returns: 10_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6721Mode::ViolationPayerFailedToFileAndDidNotCorrectByAugust1
        );
        assert_eq!(result.raw_penalty_dollars, 2_500_000);
        assert_eq!(result.capped_penalty_dollars, 1_000_000);
    }

    #[test]
    fn tier3_not_corrected_large_filer_above_cap_capped() {
        let input = Input {
            correction_timing: CorrectionTiming::NotCorrectedByAugust1OrNotCorrected,
            filer_size: FilerSize::LargeCorporationOrLargeFilerAboveSmallBusinessThreshold,
            number_of_failed_returns: 20_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 5_000_000);
        assert_eq!(result.capped_penalty_dollars, 3_000_000);
    }

    #[test]
    fn tier3_2026_inflation_adjusted_large_filer_above_cap() {
        let input = Input {
            correction_timing: CorrectionTiming::NotCorrectedByAugust1OrNotCorrected,
            penalty_amount_version: PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532,
            filer_size: FilerSize::LargeCorporationOrLargeFilerAboveSmallBusinessThreshold,
            number_of_failed_returns: 20_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.penalty_per_return_dollars, 340);
        assert_eq!(result.raw_penalty_dollars, 6_800_000);
        assert_eq!(result.capped_penalty_dollars, 4_191_500);
    }

    #[test]
    fn intentional_disregard_per_return_greater_than_10_pct_aggregate() {
        let input = Input {
            number_of_failed_returns: 1_000,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: 100_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6721Mode::ViolationIntentionalDisregardPenaltyOwedNoMax
        );
        // per return: 1000 × $500 = $500,000 vs 10% × $100,000 = $10,000 → $500,000 wins
        assert_eq!(result.raw_penalty_dollars, 500_000);
        assert_eq!(result.capped_penalty_dollars, 500_000);
    }

    #[test]
    fn intentional_disregard_10_pct_aggregate_greater_than_per_return() {
        let input = Input {
            number_of_failed_returns: 10,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: 100_000_000,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6721Mode::ViolationIntentionalDisregardPenaltyOwedNoMax
        );
        // per return: 10 × $500 = $5,000 vs 10% × $100M = $10,000,000 → $10M wins
        assert_eq!(result.raw_penalty_dollars, 10_000_000);
    }

    #[test]
    fn intentional_disregard_no_maximum_cap() {
        let input = Input {
            number_of_failed_returns: 1_000_000,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: 0,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 500_000_000);
        assert_eq!(result.capped_penalty_dollars, 500_000_000);
    }

    #[test]
    fn intentional_disregard_2026_inflation_adjusted_per_return() {
        let input = Input {
            number_of_failed_returns: 100,
            intentional_disregard: true,
            penalty_amount_version: PenaltyAmountVersion::InflationAdjusted2026UnderRevProc202532,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(result.penalty_per_return_dollars, 680);
        assert_eq!(result.raw_penalty_dollars, 100_000); // 10% of $1M aggregate > 100 × $680
    }

    #[test]
    fn citations_pin_section_6721_tiers_and_inflation_adjustment() {
        let result = check(&baseline_tier1_small_biz());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 6721(a)"));
        assert!(joined.contains("IRC § 6721(b)(1)"));
        assert!(joined.contains("IRC § 6721(b)(2)"));
        assert!(joined.contains("IRC § 6721(d)"));
        assert!(joined.contains("IRC § 6721(e)"));
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
        assert!(joined.contains("NO MAXIMUM"));
        assert!(joined.contains("$5,000,000"));
        assert!(joined.contains("Rev. Proc. 2025-32"));
        assert!(joined.contains("IRC § 6724"));
        assert!(joined.contains("Treas. Reg. § 301.6721-1"));
        assert!(joined.contains("IRC § 6722"));
        assert!(joined.contains("IRC § 6723"));
        assert!(joined.contains("IRS IRM 20.1.7"));
    }

    #[test]
    fn constant_pin_base_amounts_and_inflation_adjusted_2026() {
        assert_eq!(IRC_6721_TIER1_PER_RETURN_BASE_DOLLARS, 50);
        assert_eq!(IRC_6721_TIER2_PER_RETURN_BASE_DOLLARS, 100);
        assert_eq!(IRC_6721_TIER3_PER_RETURN_BASE_DOLLARS, 250);
        assert_eq!(IRC_6721_INTENTIONAL_DISREGARD_PER_RETURN_BASE_DOLLARS, 500);
        assert_eq!(IRC_6721_INTENTIONAL_DISREGARD_PCT_BASIS_POINTS, 1_000);
        assert_eq!(IRC_6721_INTENTIONAL_DISREGARD_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_6721_TIER1_PER_RETURN_2026_DOLLARS, 60);
        assert_eq!(IRC_6721_TIER2_PER_RETURN_2026_DOLLARS, 130);
        assert_eq!(IRC_6721_TIER3_PER_RETURN_2026_DOLLARS, 340);
        assert_eq!(IRC_6721_INTENTIONAL_DISREGARD_PER_RETURN_2026_DOLLARS, 680);
        assert_eq!(IRC_6721_TIER1_MAX_BASE_DOLLARS, 500_000);
        assert_eq!(IRC_6721_TIER2_MAX_BASE_DOLLARS, 1_500_000);
        assert_eq!(IRC_6721_TIER3_MAX_BASE_DOLLARS, 3_000_000);
        assert_eq!(IRC_6721_TIER3_MAX_2026_DOLLARS, 4_191_500);
        assert_eq!(IRC_6721_TIER1_SMALL_BUSINESS_MAX_BASE_DOLLARS, 175_000);
        assert_eq!(IRC_6721_TIER2_SMALL_BUSINESS_MAX_BASE_DOLLARS, 500_000);
        assert_eq!(IRC_6721_TIER3_SMALL_BUSINESS_MAX_BASE_DOLLARS, 1_000_000);
        assert_eq!(IRC_6721_TIER3_SMALL_BUSINESS_MAX_2026_DOLLARS, 1_397_000);
        assert_eq!(IRC_6721_SMALL_BUSINESS_GROSS_RECEIPTS_THRESHOLD_DOLLARS, 5_000_000);
        assert_eq!(IRC_6721_INTENTIONAL_DISREGARD_NO_MAX, u64::MAX);
    }

    #[test]
    fn saturating_overflow_defense_extreme_returns_intentional() {
        let input = Input {
            number_of_failed_returns: u64::MAX,
            intentional_disregard: true,
            aggregate_amount_required_to_be_reported_dollars: u64::MAX,
            ..baseline_tier1_small_biz()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6721Mode::ViolationIntentionalDisregardPenaltyOwedNoMax
        );
    }
}
