//! IRC § 1252 — Gain from Disposition of Farm Land (Soil and
//! Water Conservation Recapture).
//!
//! Pure-compute recharacterization rule. When a taxpayer disposes
//! of farm land held LESS THAN 10 YEARS for which **§ 175 soil and
//! water conservation expenditures** were deducted, gain
//! recognized is recharacterized as ORDINARY INCOME to the extent
//! of an **APPLICABLE PERCENTAGE × aggregate § 175 deductions**
//! (capped at total gain on disposition). The applicable
//! percentage follows a **sliding scale** that decreases by 20 %
//! per year after the fifth year. Parallel sibling to § 1245
//! (personal property recapture), § 1250 (real property recapture),
//! § 1254 (oil/gas/mineral recapture — iter 622), and § 1255 (§ 126
//! property recapture).
//!
//! Statute (verbatim mapping):
//! - § 1252(a)(1) — GENERAL RULE: if farm land held less than 10
//!   years is disposed of, the LESSER of (A) applicable percentage
//!   of aggregate § 175 deductions OR (B) excess of amount
//!   realized over adjusted basis is treated as ORDINARY INCOME.
//!   The remaining gain (if any) is § 1231 / § 1221 capital gain.
//! - § 1252(a)(1) APPLICABLE PERCENTAGE sliding scale:
//!   - 100 % if disposed of within 5 years after acquisition
//!   - 80 % if within the sixth year
//!   - 60 % if within the seventh year
//!   - 40 % if within the eighth year
//!   - 20 % if within the ninth year
//!   - 0 % if 10+ years (§ 1252 inapplicable; pure § 1231 gain)
//! - § 1252(a)(2) — formerly referenced § 182 (land clearing
//!   expenditures), but § 182 was REPEALED for taxable years
//!   beginning after December 31, 1985 by Tax Reform Act of 1986
//!   (P.L. 99-514). Modern § 1252 applies only to § 175 soil and
//!   water conservation expenditures.
//! - § 1252(b) — RELATIONSHIP TO § 1245: § 1252 applies BEFORE
//!   § 1245 (which typically does not reach farm land).
//! - § 1252(c) — FARM LAND DEFINITION: any land with respect to
//!   which deductions have been allowed under § 175.
//! - Treas. Reg. § 1.1252-1 — general rule for treatment of gain
//!   from disposition of farm land.
//! - Treas. Reg. § 1.1252-2 — special rules for partnerships and
//!   their partners.
//!
//! Trader-critical for: farm/ranch trader investors (LPs and MLPs
//! holding farm real estate); family-office farm portfolios;
//! agricultural REIT exits; farmland conservation easement
//! interactions; estate liquidation of family farms with prior
//! § 175 deductions.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 1252 + Treas. Reg. § 1.1252-1 confirm
//!   statutory text and sliding-scale percentages.
//! - IRS Form 4797 Instructions confirm Part III line 27 farm
//!   land recapture reporting.
//! - Bloomberg Tax IRC § 1252 confirms 10-year holding-period
//!   ceiling and § 182 repeal effective post-1985.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1252_HOLDING_PERIOD_CEILING_YEARS: u32 = 10;
pub const SECTION_1252_100_PCT_THRESHOLD_YEARS: u32 = 5;
pub const SECTION_1252_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1252_YEAR_1_5_PERCENTAGE_BASIS_POINTS: u64 = 10_000;
pub const SECTION_1252_YEAR_6_PERCENTAGE_BASIS_POINTS: u64 = 8_000;
pub const SECTION_1252_YEAR_7_PERCENTAGE_BASIS_POINTS: u64 = 6_000;
pub const SECTION_1252_YEAR_8_PERCENTAGE_BASIS_POINTS: u64 = 4_000;
pub const SECTION_1252_YEAR_9_PERCENTAGE_BASIS_POINTS: u64 = 2_000;
pub const SECTION_182_REPEAL_EFFECTIVE_AFTER_YEAR: u32 = 1985;
pub const SECTION_182_REPEAL_LAW: &str = "P.L. 99-514 (Tax Reform Act of 1986)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldingPeriodBucket {
    LessThan5Years,
    SixthYear,
    SeventhYear,
    EighthYear,
    NinthYear,
    TenthYearOrLater,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacterAsReported {
    CapitalLongTerm,
    Section1231Gain,
    OrdinaryOther,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1252Mode {
    NotApplicableHoldingPeriodOver10Years,
    NotApplicableNoSection175DeductionsAllowed,
    NotApplicableNoGainRecognized,
    CompliantGainRecharacterizedAsOrdinaryFullSlidingScale,
    CompliantSection175DeductionsLessThanGainResidualCapital,
    ViolationGainReportedAsCapitalDespiteSection1252,
    ViolationApplicableSlidingScalePercentageMisapplied,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub holding_period_bucket: HoldingPeriodBucket,
    pub gain_recognized_dollars: u64,
    pub aggregate_section_175_deductions_dollars: u64,
    pub gain_character_as_reported: GainCharacterAsReported,
    pub taxpayer_applied_correct_sliding_scale_percentage: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1252Mode,
    pub applicable_percentage_basis_points: u64,
    pub ordinary_income_recharacterized_dollars: u64,
    pub remaining_capital_or_1231_gain_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1252Input = Input;
pub type Section1252Output = Output;
pub type Section1252Result = Output;

fn applicable_percentage_basis_points(bucket: HoldingPeriodBucket) -> u64 {
    match bucket {
        HoldingPeriodBucket::LessThan5Years => SECTION_1252_YEAR_1_5_PERCENTAGE_BASIS_POINTS,
        HoldingPeriodBucket::SixthYear => SECTION_1252_YEAR_6_PERCENTAGE_BASIS_POINTS,
        HoldingPeriodBucket::SeventhYear => SECTION_1252_YEAR_7_PERCENTAGE_BASIS_POINTS,
        HoldingPeriodBucket::EighthYear => SECTION_1252_YEAR_8_PERCENTAGE_BASIS_POINTS,
        HoldingPeriodBucket::NinthYear => SECTION_1252_YEAR_9_PERCENTAGE_BASIS_POINTS,
        HoldingPeriodBucket::TenthYearOrLater => 0,
    }
}

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_1252_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1252(a)(1) — gain on farm land held < 10 years recharacterized as ordinary income to extent of LESSER of applicable percentage of § 175 deductions OR gain recognized".to_string(),
        "26 U.S.C. § 1252(a)(1) sliding scale — 100 % < 5 years; 80 % 6th year; 60 % 7th year; 40 % 8th year; 20 % 9th year; 0 % at 10+ years".to_string(),
        "26 U.S.C. § 1252(a)(2) — formerly referenced § 182 land clearing; § 182 REPEALED for taxable years beginning after Dec 31, 1985 by Tax Reform Act of 1986 (P.L. 99-514)".to_string(),
        "26 U.S.C. § 1252(b) — § 1252 applies BEFORE § 1245 (which typically does not reach farm land)".to_string(),
        "26 U.S.C. § 1252(c) — farm land = any land with respect to which § 175 deductions allowed".to_string(),
        "26 U.S.C. § 175 — soil and water conservation expenditures deduction (ordinary deduction in year incurred)".to_string(),
        "Treas. Reg. § 1.1252-1 — general rule for treatment of gain from disposition of farm land".to_string(),
        "Treas. Reg. § 1.1252-2 — special rules for partnerships and their partners".to_string(),
        "IRS Form 4797 Part III line 27 — § 1252 farm land recapture reporting".to_string(),
        "Sibling recapture provisions: § 1245 personal property; § 1250 real property; § 1254 oil/gas/mineral; § 1255 § 126 property".to_string(),
    ];

    if input.holding_period_bucket == HoldingPeriodBucket::TenthYearOrLater {
        return Output {
            mode: Section1252Mode::NotApplicableHoldingPeriodOver10Years,
            applicable_percentage_basis_points: 0,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "§ 1252(a)(1) — 10-year holding period ceiling".to_string(),
            notes: format!(
                "§ 1252 inapplicable: holding period 10 years or more; gain of ${} preserves § 1231 / capital character.",
                input.gain_recognized_dollars
            ),
            citations,
        };
    }

    if input.aggregate_section_175_deductions_dollars == 0 {
        return Output {
            mode: Section1252Mode::NotApplicableNoSection175DeductionsAllowed,
            applicable_percentage_basis_points: 0,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "§ 1252(c) — farm land requires § 175 deductions to qualify".to_string(),
            notes: "No § 175 soil and water conservation deductions allowed; land does not satisfy § 1252(c) farm land definition; § 1252 inapplicable.".to_string(),
            citations,
        };
    }

    if input.gain_recognized_dollars == 0 {
        return Output {
            mode: Section1252Mode::NotApplicableNoGainRecognized,
            applicable_percentage_basis_points: 0,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: 0,
            statutory_basis: "§ 1252 inapplicable absent gain".to_string(),
            notes: "No gain recognized on disposition; § 1252 inapplicable.".to_string(),
            citations,
        };
    }

    let pct_bp = applicable_percentage_basis_points(input.holding_period_bucket);
    let applicable_amount = apply_rate(input.aggregate_section_175_deductions_dollars, pct_bp);
    let ordinary = applicable_amount.min(input.gain_recognized_dollars);
    let remaining = input.gain_recognized_dollars.saturating_sub(ordinary);

    if !input.taxpayer_applied_correct_sliding_scale_percentage {
        return Output {
            mode: Section1252Mode::ViolationApplicableSlidingScalePercentageMisapplied,
            applicable_percentage_basis_points: pct_bp,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1252(a)(1) — applicable percentage sliding scale must be correctly applied".to_string(),
            notes: format!(
                "VIOLATION § 1252(a)(1): taxpayer did not apply correct sliding-scale percentage for holding period {:?}; correct percentage = {} basis points ({}%); ordinary income recharacterization = ${}.",
                input.holding_period_bucket,
                pct_bp,
                pct_bp / 100,
                ordinary
            ),
            citations,
        };
    }

    if matches!(
        input.gain_character_as_reported,
        GainCharacterAsReported::CapitalLongTerm | GainCharacterAsReported::Section1231Gain
    ) && ordinary > 0
    {
        return Output {
            mode: Section1252Mode::ViolationGainReportedAsCapitalDespiteSection1252,
            applicable_percentage_basis_points: pct_bp,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1252(a)(1) — gain reported as capital / § 1231 despite § 175 deductions".to_string(),
            notes: format!(
                "VIOLATION § 1252(a)(1): gain of ${} reported as {:?} but § 1252 recharacterizes ${} as ordinary income (lesser of {}-bp × § 175 deductions of ${} = ${} OR gain ${}); residual gain = ${}.",
                input.gain_recognized_dollars,
                input.gain_character_as_reported,
                ordinary,
                pct_bp,
                input.aggregate_section_175_deductions_dollars,
                applicable_amount,
                input.gain_recognized_dollars,
                remaining
            ),
            citations,
        };
    }

    if applicable_amount < input.gain_recognized_dollars {
        return Output {
            mode: Section1252Mode::CompliantSection175DeductionsLessThanGainResidualCapital,
            applicable_percentage_basis_points: pct_bp,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1252(a)(1)(A) — applicable percentage × § 175 deductions less than gain; residual capital".to_string(),
            notes: format!(
                "COMPLIANT § 1252(a)(1): holding period {:?} → {}-bp sliding scale; ordinary recharacterization = ${} (= {} × ${} § 175 deductions); residual § 1231 / capital gain = ${}.",
                input.holding_period_bucket, pct_bp, ordinary, pct_bp / 100, input.aggregate_section_175_deductions_dollars, remaining
            ),
            citations,
        };
    }

    Output {
        mode: Section1252Mode::CompliantGainRecharacterizedAsOrdinaryFullSlidingScale,
        applicable_percentage_basis_points: pct_bp,
        ordinary_income_recharacterized_dollars: ordinary,
        remaining_capital_or_1231_gain_dollars: remaining,
        statutory_basis: "§ 1252(a)(1) — gain fully recharacterized at sliding scale".to_string(),
        notes: format!(
            "COMPLIANT § 1252(a)(1): holding period {:?}; {}-bp sliding scale × § 175 deductions ${} = ${} applicable amount; ordinary income = min(${}, ${}) = ${}; residual = ${}.",
            input.holding_period_bucket,
            pct_bp,
            input.aggregate_section_175_deductions_dollars,
            applicable_amount,
            applicable_amount,
            input.gain_recognized_dollars,
            ordinary,
            remaining
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_post_175_under_5_years() -> Input {
        Input {
            holding_period_bucket: HoldingPeriodBucket::LessThan5Years,
            gain_recognized_dollars: 1_000_000,
            aggregate_section_175_deductions_dollars: 400_000,
            gain_character_as_reported: GainCharacterAsReported::OrdinaryOther,
            taxpayer_applied_correct_sliding_scale_percentage: true,
        }
    }

    #[test]
    fn holding_period_over_10_years_not_applicable() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::TenthYearOrLater,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::NotApplicableHoldingPeriodOver10Years);
    }

    #[test]
    fn no_section_175_deductions_not_applicable() {
        let input = Input {
            aggregate_section_175_deductions_dollars: 0,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::NotApplicableNoSection175DeductionsAllowed);
    }

    #[test]
    fn no_gain_not_applicable() {
        let input = Input {
            gain_recognized_dollars: 0,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::NotApplicableNoGainRecognized);
    }

    #[test]
    fn under_5_years_100_pct_compliant_full_recapture() {
        let result = compute(&baseline_post_175_under_5_years());
        assert_eq!(result.mode, Section1252Mode::CompliantSection175DeductionsLessThanGainResidualCapital);
        assert_eq!(result.applicable_percentage_basis_points, 10_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 400_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 600_000);
    }

    #[test]
    fn sixth_year_80_pct_recharacterization() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::SixthYear,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 8_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 320_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 680_000);
    }

    #[test]
    fn seventh_year_60_pct_recharacterization() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::SeventhYear,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 6_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 240_000);
    }

    #[test]
    fn eighth_year_40_pct_recharacterization() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::EighthYear,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 4_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 160_000);
    }

    #[test]
    fn ninth_year_20_pct_recharacterization() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::NinthYear,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 2_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 80_000);
    }

    #[test]
    fn deductions_exceed_gain_capped_at_gain() {
        let input = Input {
            aggregate_section_175_deductions_dollars: 2_000_000,
            gain_recognized_dollars: 1_000_000,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::CompliantGainRecharacterizedAsOrdinaryFullSlidingScale);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 1_000_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 0);
    }

    #[test]
    fn capital_long_term_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacterAsReported::CapitalLongTerm,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::ViolationGainReportedAsCapitalDespiteSection1252);
    }

    #[test]
    fn section_1231_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacterAsReported::Section1231Gain,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::ViolationGainReportedAsCapitalDespiteSection1252);
    }

    #[test]
    fn incorrect_sliding_scale_percentage_violation() {
        let input = Input {
            taxpayer_applied_correct_sliding_scale_percentage: false,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1252Mode::ViolationApplicableSlidingScalePercentageMisapplied);
    }

    #[test]
    fn at_4_year_4_months_still_in_under_5_year_bucket() {
        let result = compute(&baseline_post_175_under_5_years());
        assert_eq!(result.applicable_percentage_basis_points, 10_000);
    }

    #[test]
    fn citations_pin_section_1252_subsections_and_182_repeal() {
        let result = compute(&baseline_post_175_under_5_years());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1252(a)(1)"));
        assert!(joined.contains("100 % < 5 years"));
        assert!(joined.contains("80 % 6th year"));
        assert!(joined.contains("60 % 7th year"));
        assert!(joined.contains("40 % 8th year"));
        assert!(joined.contains("20 % 9th year"));
        assert!(joined.contains("0 % at 10+ years"));
        assert!(joined.contains("§ 182 REPEALED"));
        assert!(joined.contains("P.L. 99-514"));
        assert!(joined.contains("§ 1252(b)"));
        assert!(joined.contains("§ 1252(c)"));
        assert!(joined.contains("§ 175"));
        assert!(joined.contains("§ 1.1252-1"));
        assert!(joined.contains("§ 1.1252-2"));
        assert!(joined.contains("Form 4797 Part III line 27"));
        assert!(joined.contains("§ 1245"));
        assert!(joined.contains("§ 1250"));
        assert!(joined.contains("§ 1254"));
        assert!(joined.contains("§ 1255"));
    }

    #[test]
    fn constant_pin_sliding_scale_percentages_and_dates() {
        assert_eq!(SECTION_1252_HOLDING_PERIOD_CEILING_YEARS, 10);
        assert_eq!(SECTION_1252_100_PCT_THRESHOLD_YEARS, 5);
        assert_eq!(SECTION_1252_YEAR_1_5_PERCENTAGE_BASIS_POINTS, 10_000);
        assert_eq!(SECTION_1252_YEAR_6_PERCENTAGE_BASIS_POINTS, 8_000);
        assert_eq!(SECTION_1252_YEAR_7_PERCENTAGE_BASIS_POINTS, 6_000);
        assert_eq!(SECTION_1252_YEAR_8_PERCENTAGE_BASIS_POINTS, 4_000);
        assert_eq!(SECTION_1252_YEAR_9_PERCENTAGE_BASIS_POINTS, 2_000);
        assert_eq!(SECTION_182_REPEAL_EFFECTIVE_AFTER_YEAR, 1985);
    }

    #[test]
    fn saturating_overflow_defense_extreme_amounts() {
        let input = Input {
            gain_recognized_dollars: u64::MAX,
            aggregate_section_175_deductions_dollars: u64::MAX,
            ..baseline_post_175_under_5_years()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section1252Mode::CompliantGainRecharacterizedAsOrdinaryFullSlidingScale
                | Section1252Mode::CompliantSection175DeductionsLessThanGainResidualCapital
        ));
    }
}
