//! IRC § 1255 — Gain from Disposition of § 126 Property
//! (Conservation Cost-Sharing Payment Recapture).
//!
//! Pure-compute recharacterization rule. When a taxpayer disposes
//! of § 126 property — property acquired, improved, or modified
//! by the application of payments EXCLUDED from gross income under
//! § 126 (federal conservation cost-sharing payments) — gain
//! recognized is recharacterized as ORDINARY INCOME to the extent
//! of an APPLICABLE PERCENTAGE × aggregate § 126 excluded
//! payments. The applicable percentage follows a 20-year sliding
//! cliff: 100 % through year 10, then -10 % per full year
//! thereafter, reaching 0 % at year 20. Completes the recapture
//! quartet with § 1245 (personal property) + § 1250 (real
//! property) + § 1252 (farm land — iter 634) + § 1254 (oil/gas/
//! mineral — iter 622).
//!
//! Statute (verbatim mapping):
//! - § 1255(a)(1) — GENERAL RULE: if § 126 property is disposed
//!   of, the LESSER of (A) applicable percentage × aggregate
//!   payments excluded under § 126 OR (B) excess of amount
//!   realized over adjusted basis is treated as ORDINARY INCOME.
//!   Remaining gain (if any) is § 1231 / capital character.
//! - § 1255(a)(2) APPLICABLE PERCENTAGE sliding scale:
//!   - 100 % if disposed within 10 years
//!   - REDUCED BY 10 % for each full year beyond 10 years
//!   - 90 % in 11th year
//!   - 80 % in 12th year
//!   - 70 % in 13th year
//!   - ... declining 10 % per year
//!   - 0 % at 20+ years (§ 1255 inapplicable; pure § 1231 gain)
//! - § 1255(b) — SPECIAL RULES: cross-reference to § 126 for
//!   definition of cost-sharing payment.
//! - **§ 126 EXCLUDED PAYMENTS**: federal conservation cost-
//!   sharing payments excluded from gross income under § 126,
//!   including: USDA Agricultural Conservation Program payments;
//!   USDA Conservation Reserve Program (CRP) cost-share; USDA
//!   Environmental Quality Incentives Program (EQIP) cost-share;
//!   Forest Service Forestry Incentives Program; certain state
//!   conservation programs designated by Treasury.
//! - Treas. Reg. § 16A.1255-1 — temporary regulations relating
//!   to partial exclusion for certain conservation cost-sharing
//!   payments (treatment of § 126 property disposition).
//!
//! Trader-critical for: farm/ranch trader investors and family
//! offices holding conservation-cost-sharing-funded property;
//! agricultural REIT exits with prior § 126 payments; estate
//! liquidation of conservation-encumbered family farms; trader-
//! landlord cross-portfolios with rural property holdings.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 1255 + Treas. Reg. § 16A.1255-1 confirm
//!   statutory text and 100 % / -10 % sliding scale.
//! - IRS Form 4797 Instructions confirm Part III line 27 § 1255
//!   reporting.
//! - National Timber Tax Section 1255 confirms anti-double-benefit
//!   policy (preventing taxpayer from excluding payment AND
//!   getting capital gains on improved property).
//! - IRS Publication 544 (Sales and Other Dispositions of Assets)
//!   confirms § 1255 reporting workflow.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1255_FULL_RECAPTURE_THRESHOLD_YEARS: u32 = 10;
pub const SECTION_1255_RECAPTURE_CLIFF_YEARS: u32 = 20;
pub const SECTION_1255_REDUCTION_PER_YEAR_BASIS_POINTS: u64 = 1_000;
pub const SECTION_1255_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1255_FULL_PERCENTAGE_BASIS_POINTS: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldingPeriodBucket {
    LessThan11Years,
    ElevenYears,
    TwelveYears,
    ThirteenYears,
    FourteenYears,
    FifteenYears,
    SixteenYears,
    SeventeenYears,
    EighteenYears,
    NineteenYears,
    TwentyYearsOrLater,
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
pub enum Section126PaymentSource {
    UsdaAgriculturalConservationProgram,
    UsdaConservationReserveProgram,
    UsdaEnvironmentalQualityIncentivesProgram,
    ForestServiceForestryIncentivesProgram,
    StateConservationProgramDesignated,
    NoSection126Payments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1255Mode {
    NotApplicableHoldingPeriodOver20Years,
    NotApplicableNoSection126ExcludedPayments,
    NotApplicableNoGainRecognized,
    CompliantGainRecharacterizedAsOrdinaryFullSlidingScale,
    CompliantSection126PaymentsLessThanGainResidualCapital,
    ViolationGainReportedAsCapitalDespiteSection1255,
    ViolationApplicableSlidingScalePercentageMisapplied,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub holding_period_bucket: HoldingPeriodBucket,
    pub gain_recognized_dollars: u64,
    pub aggregate_section_126_excluded_payments_dollars: u64,
    pub payment_source: Section126PaymentSource,
    pub gain_character_as_reported: GainCharacterAsReported,
    pub taxpayer_applied_correct_sliding_scale_percentage: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1255Mode,
    pub applicable_percentage_basis_points: u64,
    pub ordinary_income_recharacterized_dollars: u64,
    pub remaining_capital_or_1231_gain_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1255Input = Input;
pub type Section1255Output = Output;
pub type Section1255Result = Output;

fn applicable_percentage_basis_points(bucket: HoldingPeriodBucket) -> u64 {
    match bucket {
        HoldingPeriodBucket::LessThan11Years => 10_000,
        HoldingPeriodBucket::ElevenYears => 9_000,
        HoldingPeriodBucket::TwelveYears => 8_000,
        HoldingPeriodBucket::ThirteenYears => 7_000,
        HoldingPeriodBucket::FourteenYears => 6_000,
        HoldingPeriodBucket::FifteenYears => 5_000,
        HoldingPeriodBucket::SixteenYears => 4_000,
        HoldingPeriodBucket::SeventeenYears => 3_000,
        HoldingPeriodBucket::EighteenYears => 2_000,
        HoldingPeriodBucket::NineteenYears => 1_000,
        HoldingPeriodBucket::TwentyYearsOrLater => 0,
    }
}

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_1255_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1255(a)(1) — gain on disposition of § 126 property recharacterized as ordinary income to extent of LESSER of applicable percentage × aggregate § 126 excluded payments OR gain recognized".to_string(),
        "26 U.S.C. § 1255(a)(2) — applicable percentage sliding scale: 100 % through 10 years; -10 % per full year thereafter; 0 % at 20+ years".to_string(),
        "26 U.S.C. § 1255(b) — special rules; cross-reference to § 126".to_string(),
        "26 U.S.C. § 126 — partial exclusion from gross income for certain conservation cost-sharing payments (USDA ACP / CRP / EQIP; Forest Service FIP; designated state programs)".to_string(),
        "Treas. Reg. § 16A.1255-1 — general rule for treatment of gain from disposition of § 126 property (temporary regulations)".to_string(),
        "26 C.F.R. Part 16A — Temporary Income Tax Regulations Relating to Partial Exclusion for Conservation Cost-Sharing Payments".to_string(),
        "IRS Form 4797 Part III line 27 — § 1255 recapture reporting".to_string(),
        "IRS Publication 544 (Sales and Other Dispositions of Assets) — § 1255 disposition workflow".to_string(),
        "Sibling recapture provisions: § 1245 personal property; § 1250 real property; § 1252 farm land (iter 634); § 1254 oil/gas/mineral (iter 622)".to_string(),
        "Anti-double-benefit policy — prevents taxpayer from excluding § 126 payment from income AND getting capital gains on improved property".to_string(),
    ];

    if input.holding_period_bucket == HoldingPeriodBucket::TwentyYearsOrLater {
        return Output {
            mode: Section1255Mode::NotApplicableHoldingPeriodOver20Years,
            applicable_percentage_basis_points: 0,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "§ 1255(a)(2) — 20-year sliding-cliff ceiling".to_string(),
            notes: format!(
                "§ 1255 inapplicable: holding period 20+ years (applicable percentage = 0 %); gain of ${} preserves § 1231 / capital character.",
                input.gain_recognized_dollars
            ),
            citations,
        };
    }

    if input.aggregate_section_126_excluded_payments_dollars == 0
        || input.payment_source == Section126PaymentSource::NoSection126Payments
    {
        return Output {
            mode: Section1255Mode::NotApplicableNoSection126ExcludedPayments,
            applicable_percentage_basis_points: 0,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "§ 126 — no excluded conservation cost-sharing payments".to_string(),
            notes: "No § 126 excluded conservation cost-sharing payments; property does not satisfy § 126 property definition; § 1255 inapplicable.".to_string(),
            citations,
        };
    }

    if input.gain_recognized_dollars == 0 {
        return Output {
            mode: Section1255Mode::NotApplicableNoGainRecognized,
            applicable_percentage_basis_points: 0,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: 0,
            statutory_basis: "§ 1255 inapplicable absent gain".to_string(),
            notes: "No gain recognized on disposition; § 1255 inapplicable.".to_string(),
            citations,
        };
    }

    let pct_bp = applicable_percentage_basis_points(input.holding_period_bucket);
    let applicable_amount = apply_rate(
        input.aggregate_section_126_excluded_payments_dollars,
        pct_bp,
    );
    let ordinary = applicable_amount.min(input.gain_recognized_dollars);
    let remaining = input.gain_recognized_dollars.saturating_sub(ordinary);

    if !input.taxpayer_applied_correct_sliding_scale_percentage {
        return Output {
            mode: Section1255Mode::ViolationApplicableSlidingScalePercentageMisapplied,
            applicable_percentage_basis_points: pct_bp,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1255(a)(2) — applicable percentage sliding scale must be correctly applied".to_string(),
            notes: format!(
                "VIOLATION § 1255(a)(2): taxpayer did not apply correct sliding-scale percentage for holding period {:?}; correct percentage = {} basis points ({}%); ordinary income recharacterization = ${}.",
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
            mode: Section1255Mode::ViolationGainReportedAsCapitalDespiteSection1255,
            applicable_percentage_basis_points: pct_bp,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1255(a)(1) — gain reported as capital / § 1231 despite § 126 excluded payments".to_string(),
            notes: format!(
                "VIOLATION § 1255(a)(1): gain of ${} reported as {:?} but § 1255 recharacterizes ${} as ordinary income (lesser of {}-bp × § 126 payments of ${} = ${} OR gain ${}); residual gain = ${}.",
                input.gain_recognized_dollars,
                input.gain_character_as_reported,
                ordinary,
                pct_bp,
                input.aggregate_section_126_excluded_payments_dollars,
                applicable_amount,
                input.gain_recognized_dollars,
                remaining
            ),
            citations,
        };
    }

    if applicable_amount < input.gain_recognized_dollars {
        return Output {
            mode: Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital,
            applicable_percentage_basis_points: pct_bp,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1255(a)(1)(A) — applicable percentage × § 126 payments less than gain; residual capital".to_string(),
            notes: format!(
                "COMPLIANT § 1255(a)(1): holding period {:?} → {}-bp sliding scale; ordinary recharacterization = ${} (= {}% × ${} § 126 payments); residual § 1231 / capital gain = ${}.",
                input.holding_period_bucket,
                pct_bp,
                ordinary,
                pct_bp / 100,
                input.aggregate_section_126_excluded_payments_dollars,
                remaining
            ),
            citations,
        };
    }

    Output {
        mode: Section1255Mode::CompliantGainRecharacterizedAsOrdinaryFullSlidingScale,
        applicable_percentage_basis_points: pct_bp,
        ordinary_income_recharacterized_dollars: ordinary,
        remaining_capital_or_1231_gain_dollars: remaining,
        statutory_basis: "§ 1255(a)(1) — gain fully recharacterized at sliding scale".to_string(),
        notes: format!(
            "COMPLIANT § 1255(a)(1): holding period {:?}; {}-bp sliding scale × § 126 payments ${} = ${} applicable amount; ordinary income = min(${}, ${}) = ${}; residual = ${}.",
            input.holding_period_bucket,
            pct_bp,
            input.aggregate_section_126_excluded_payments_dollars,
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

    fn baseline_section_126_under_10_years() -> Input {
        Input {
            holding_period_bucket: HoldingPeriodBucket::LessThan11Years,
            gain_recognized_dollars: 500_000,
            aggregate_section_126_excluded_payments_dollars: 200_000,
            payment_source: Section126PaymentSource::UsdaConservationReserveProgram,
            gain_character_as_reported: GainCharacterAsReported::OrdinaryOther,
            taxpayer_applied_correct_sliding_scale_percentage: true,
        }
    }

    #[test]
    fn holding_period_over_20_years_not_applicable() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::TwentyYearsOrLater,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::NotApplicableHoldingPeriodOver20Years);
    }

    #[test]
    fn no_section_126_payments_not_applicable() {
        let input = Input {
            aggregate_section_126_excluded_payments_dollars: 0,
            payment_source: Section126PaymentSource::NoSection126Payments,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::NotApplicableNoSection126ExcludedPayments);
    }

    #[test]
    fn no_gain_not_applicable() {
        let input = Input {
            gain_recognized_dollars: 0,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::NotApplicableNoGainRecognized);
    }

    #[test]
    fn under_11_years_100_pct_compliant() {
        let result = compute(&baseline_section_126_under_10_years());
        assert_eq!(result.mode, Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital);
        assert_eq!(result.applicable_percentage_basis_points, 10_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 200_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 300_000);
    }

    #[test]
    fn eleven_years_90_pct() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::ElevenYears,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 9_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 180_000);
    }

    #[test]
    fn twelve_years_80_pct() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::TwelveYears,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 8_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 160_000);
    }

    #[test]
    fn fifteen_years_50_pct() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::FifteenYears,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 5_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 100_000);
    }

    #[test]
    fn nineteen_years_10_pct() {
        let input = Input {
            holding_period_bucket: HoldingPeriodBucket::NineteenYears,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_percentage_basis_points, 1_000);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 20_000);
    }

    #[test]
    fn payments_exceed_gain_capped_at_gain() {
        let input = Input {
            aggregate_section_126_excluded_payments_dollars: 1_000_000,
            gain_recognized_dollars: 400_000,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::CompliantGainRecharacterizedAsOrdinaryFullSlidingScale);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 400_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 0);
    }

    #[test]
    fn capital_long_term_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacterAsReported::CapitalLongTerm,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::ViolationGainReportedAsCapitalDespiteSection1255);
    }

    #[test]
    fn section_1231_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacterAsReported::Section1231Gain,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::ViolationGainReportedAsCapitalDespiteSection1255);
    }

    #[test]
    fn incorrect_sliding_scale_percentage_violation() {
        let input = Input {
            taxpayer_applied_correct_sliding_scale_percentage: false,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::ViolationApplicableSlidingScalePercentageMisapplied);
    }

    #[test]
    fn usda_acp_payment_source_compliant() {
        let input = Input {
            payment_source: Section126PaymentSource::UsdaAgriculturalConservationProgram,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital);
    }

    #[test]
    fn usda_eqip_payment_source_compliant() {
        let input = Input {
            payment_source: Section126PaymentSource::UsdaEnvironmentalQualityIncentivesProgram,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital);
    }

    #[test]
    fn forest_service_fip_payment_source_compliant() {
        let input = Input {
            payment_source: Section126PaymentSource::ForestServiceForestryIncentivesProgram,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital);
    }

    #[test]
    fn state_conservation_program_payment_source_compliant() {
        let input = Input {
            payment_source: Section126PaymentSource::StateConservationProgramDesignated,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital);
    }

    #[test]
    fn citations_pin_section_1255_subsections_and_treas_regs() {
        let result = compute(&baseline_section_126_under_10_years());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1255(a)(1)"));
        assert!(joined.contains("§ 1255(a)(2)"));
        assert!(joined.contains("100 % through 10 years"));
        assert!(joined.contains("-10 % per full year"));
        assert!(joined.contains("0 % at 20+ years"));
        assert!(joined.contains("§ 1255(b)"));
        assert!(joined.contains("§ 126"));
        assert!(joined.contains("USDA ACP"));
        assert!(joined.contains("CRP"));
        assert!(joined.contains("EQIP"));
        assert!(joined.contains("Forest Service FIP"));
        assert!(joined.contains("§ 16A.1255-1"));
        assert!(joined.contains("Form 4797 Part III line 27"));
        assert!(joined.contains("Publication 544"));
        assert!(joined.contains("§ 1245"));
        assert!(joined.contains("§ 1250"));
        assert!(joined.contains("§ 1252"));
        assert!(joined.contains("§ 1254"));
        assert!(joined.contains("Anti-double-benefit"));
    }

    #[test]
    fn constant_pin_sliding_scale_thresholds() {
        assert_eq!(SECTION_1255_FULL_RECAPTURE_THRESHOLD_YEARS, 10);
        assert_eq!(SECTION_1255_RECAPTURE_CLIFF_YEARS, 20);
        assert_eq!(SECTION_1255_REDUCTION_PER_YEAR_BASIS_POINTS, 1_000);
        assert_eq!(SECTION_1255_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SECTION_1255_FULL_PERCENTAGE_BASIS_POINTS, 10_000);
    }

    #[test]
    fn saturating_overflow_defense_extreme_amounts() {
        let input = Input {
            gain_recognized_dollars: u64::MAX,
            aggregate_section_126_excluded_payments_dollars: u64::MAX,
            ..baseline_section_126_under_10_years()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section1255Mode::CompliantGainRecharacterizedAsOrdinaryFullSlidingScale
                | Section1255Mode::CompliantSection126PaymentsLessThanGainResidualCapital
        ));
    }
}
