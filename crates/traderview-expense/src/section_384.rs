//! IRC § 384 limitation on use of preacquisition losses to offset built-in gain.
//!
//! § 384 prevents a "loss corporation" from offsetting "recognized built-in gain" of an
//! acquired profitable corporation (the "gain corporation") during a 5-year recognition
//! period. The provision plugs the converse hole that § 382 leaves open: § 382 stops a
//! loss-corp from absorbing the profitable-corp's POST-acquisition income; § 384 stops
//! the loss-corp from absorbing the profitable-corp's PRE-acquisition built-in gain that
//! happens to be recognized during the 5-year window.
//!
//! § 384(a) GENERAL RULE: if a corporation acquires control of another corporation, OR
//! the assets of a corporation are acquired in a § 368 reorganization, AND either
//! corporation is a "gain corporation," income for any recognition-period taxable year
//! attributable to RECOGNIZED BUILT-IN GAIN cannot be offset by any "preacquisition loss"
//! (other than a preacquisition loss of the gain corporation itself).
//!
//! § 384(a)(1) RECOGNITION PERIOD: 5 years beginning on the acquisition date. Built-in
//! gains recognized after year 5 are NOT subject to § 384.
//!
//! § 384(b)(2) CONTROL THRESHOLD: § 1504(a)(2) standard — at least 80% of total voting
//! power AND at least 80% of total value (the same standard used for affiliated-group
//! filings under § 1504).
//!
//! § 384(b)(3) COMMON-CONTROL EXCEPTION: § 384 does NOT apply where the loss corp and
//! the gain corp were members of the same controlled group at all times during the 5-year
//! period ending on the acquisition date. § 1563(a) controlled-group standard, but with
//! "more than 50%" substituted for "at least 80%" for purposes of § 384.
//!
//! § 384(c)(2) BUILT-IN-GAIN DEFINITION: net unrealized built-in gain that exists on
//! the acquisition date and is recognized within the 5-year recognition period via sale,
//! exchange, distribution, or other recognition event.
//!
//! § 384(c)(8) PREACQUISITION-LOSS DEFINITION: NOL carryforward to the taxable year of
//! acquisition, NOL for the year of acquisition allocable to the pre-acquisition portion
//! of that year, AND (per § 384(d)) capital loss carryover, general business credit
//! carryforward, foreign tax credit carryover, and similar attributes.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/384
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_384
//! - thetaxadviser.com/issues/2023/feb/issues-in-allocating-income-under-sec-384/
//! - taxnotes.com/research/federal/usc26/384

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionType {
    /// Stock acquisition satisfying § 1504(a)(2) 80% vote AND 80% value control.
    StockAcquisitionSection1504A2Control,
    /// § 368 reorganization asset acquisition.
    Section368ReorganizationAssetAcquisition,
    /// No qualifying acquisition — § 384 inapplicable.
    NoQualifyingAcquisition,
}

/// Whether the common-control exception under § 384(b)(3) applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommonControlExceptionStatus {
    /// Loss corp and gain corp NOT in same controlled group for 5 years pre-acquisition.
    NotInSameControlledGroupExceptionInapplicable,
    /// Loss corp and gain corp WERE in same controlled group (>50% per § 384(b)(3)) at
    /// all times during 5-year pre-acquisition period — § 384 inapplicable.
    InSameControlledGroupFiveYearsPreAcquisitionExceptionApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognitionTimingStatus {
    /// Built-in gain recognized within 5-year recognition period.
    BuiltInGainRecognizedWithinFiveYearRecognitionPeriod,
    /// Built-in gain recognized AFTER 5-year recognition period — outside § 384.
    BuiltInGainRecognizedAfterFiveYearWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoQualifyingAcquisitionSection384Inapplicable,
    CommonControlExceptionAppliesSection384bThreeNoDisallowance,
    BuiltInGainRecognizedAfterFiveYearWindowNoDisallowance,
    NoPreacquisitionLossNoDisallowance,
    Section384APreacquisitionLossOffsetDisallowed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub acquisition_type: AcquisitionType,
    pub common_control_exception_status: CommonControlExceptionStatus,
    pub recognition_timing_status: RecognitionTimingStatus,
    pub recognized_built_in_gain_cents: u64,
    pub preacquisition_loss_carryover_cents: u64,
    pub gain_corp_own_preacquisition_loss_cents: u64,
}

pub type Section384PreacquisitionLossDisallowanceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub disallowed_offset_cents: u64,
    pub allowed_offset_cents: u64,
    pub note: String,
}

pub type Section384PreacquisitionLossDisallowanceOutput = Output;
pub type Section384PreacquisitionLossDisallowanceResult = Output;

const RECOGNITION_PERIOD_YEARS: u32 = 5;
const CONTROL_THRESHOLD_PERCENT: u32 = 80;
const COMMON_CONTROL_EXCEPTION_THRESHOLD_PERCENT: u32 = 50;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.acquisition_type, AcquisitionType::NoQualifyingAcquisition) {
        return Output {
            severity: Severity::NoQualifyingAcquisitionSection384Inapplicable,
            disallowed_offset_cents: 0,
            allowed_offset_cents: input.preacquisition_loss_carryover_cents.min(
                input.recognized_built_in_gain_cents,
            ),
            note: format!(
                "§ 384 inapplicable: no qualifying acquisition. § 384 requires either \
                 stock-acquisition control (§ 1504(a)(2) {CONTROL_THRESHOLD_PERCENT}% vote AND \
                 value) OR § 368 reorganization asset acquisition. Preacquisition loss may \
                 offset built-in gain subject to other limitations (§ 382 annual NOL cap, \
                 § 383 credit cap, § 269 discretionary disallowance)."
            ),
        };
    }

    if matches!(
        input.common_control_exception_status,
        CommonControlExceptionStatus::InSameControlledGroupFiveYearsPreAcquisitionExceptionApplies
    ) {
        return Output {
            severity: Severity::CommonControlExceptionAppliesSection384bThreeNoDisallowance,
            disallowed_offset_cents: 0,
            allowed_offset_cents: input.preacquisition_loss_carryover_cents.min(
                input.recognized_built_in_gain_cents,
            ),
            note: format!(
                "§ 384(b)(3) common-control exception applies. Loss corp and gain corp were \
                 members of the same controlled group (more than \
                 {COMMON_CONTROL_EXCEPTION_THRESHOLD_PERCENT}% per § 384(b)(3) modified \
                 § 1563(a) standard) at all times during the 5-year period ending on the \
                 acquisition date. Preacquisition loss may offset recognized built-in gain \
                 without § 384 disallowance."
            ),
        };
    }

    if matches!(
        input.recognition_timing_status,
        RecognitionTimingStatus::BuiltInGainRecognizedAfterFiveYearWindow
    ) {
        return Output {
            severity: Severity::BuiltInGainRecognizedAfterFiveYearWindowNoDisallowance,
            disallowed_offset_cents: 0,
            allowed_offset_cents: input.preacquisition_loss_carryover_cents.min(
                input.recognized_built_in_gain_cents,
            ),
            note: format!(
                "Built-in gain recognized AFTER the {RECOGNITION_PERIOD_YEARS}-year § 384 \
                 recognition period. § 384 does NOT apply to gain recognized outside the \
                 5-year window. Preacquisition loss may offset the gain subject to other \
                 limitations (§ 382 + § 383 + § 269)."
            ),
        };
    }

    if input.preacquisition_loss_carryover_cents == 0 {
        return Output {
            severity: Severity::NoPreacquisitionLossNoDisallowance,
            disallowed_offset_cents: 0,
            allowed_offset_cents: 0,
            note: "No preacquisition loss available — § 384 disallowance has no effect. \
                   Verify NOL carryforward, capital loss carryover, general business credit \
                   carryforward, and foreign tax credit balances were correctly identified \
                   as of the acquisition date."
                .to_string(),
        };
    }

    // Gain corp's own preacquisition loss may offset its own recognized BIG (carveout
    // under § 384(a) "other than a preacquisition loss of the gain corporation").
    let gain_corp_own_offset = input
        .gain_corp_own_preacquisition_loss_cents
        .min(input.recognized_built_in_gain_cents);
    let remaining_big = input
        .recognized_built_in_gain_cents
        .saturating_sub(gain_corp_own_offset);

    let loss_corp_attempted_offset = input
        .preacquisition_loss_carryover_cents
        .saturating_sub(input.gain_corp_own_preacquisition_loss_cents);
    let disallowed = loss_corp_attempted_offset.min(remaining_big);

    Output {
        severity: Severity::Section384APreacquisitionLossOffsetDisallowed,
        disallowed_offset_cents: disallowed,
        allowed_offset_cents: gain_corp_own_offset,
        note: format!(
            "§ 384(a) preacquisition-loss offset DISALLOWED. Qualifying acquisition (§ 1504(a)(2) \
             {CONTROL_THRESHOLD_PERCENT}% control or § 368 reorganization) + built-in gain \
             recognized within {RECOGNITION_PERIOD_YEARS}-year window + common-control exception \
             inapplicable. Recognized built-in gain ${} cannot be offset by loss corp's \
             preacquisition NOL/capital-loss/credit carryforward; gain corp's OWN preacquisition \
             loss (${}) may offset its own recognized built-in gain per § 384(a) carveout. Net \
             disallowed offset ${}; allowed offset ${}. Coordinates with § 382 (annual NOL cap \
             post-ownership-change), § 383 (general-business-credit cap), § 269 (discretionary \
             disallowance on principal-purpose-of-tax-avoidance), § 381 (carryover-attribute \
             transferee rules).",
            input.recognized_built_in_gain_cents / 100,
            input.gain_corp_own_preacquisition_loss_cents / 100,
            disallowed / 100,
            gain_corp_own_offset / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            acquisition_type: AcquisitionType::StockAcquisitionSection1504A2Control,
            common_control_exception_status:
                CommonControlExceptionStatus::NotInSameControlledGroupExceptionInapplicable,
            recognition_timing_status:
                RecognitionTimingStatus::BuiltInGainRecognizedWithinFiveYearRecognitionPeriod,
            recognized_built_in_gain_cents: 10_000_000_00,
            preacquisition_loss_carryover_cents: 15_000_000_00,
            gain_corp_own_preacquisition_loss_cents: 2_000_000_00,
        }
    }

    #[test]
    fn no_qualifying_acquisition_section_384_inapplicable() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::NoQualifyingAcquisition;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoQualifyingAcquisitionSection384Inapplicable
        );
        assert_eq!(output.disallowed_offset_cents, 0);
        assert!(output.note.contains("80%"));
        assert!(output.note.contains("§ 382"));
    }

    #[test]
    fn common_control_exception_applies_no_disallowance() {
        let mut input = base();
        input.common_control_exception_status =
            CommonControlExceptionStatus::InSameControlledGroupFiveYearsPreAcquisitionExceptionApplies;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CommonControlExceptionAppliesSection384bThreeNoDisallowance
        );
        assert_eq!(output.disallowed_offset_cents, 0);
        assert!(output.note.contains("§ 384(b)(3)"));
        assert!(output.note.contains("§ 1563(a)"));
    }

    #[test]
    fn built_in_gain_after_5_year_window_no_disallowance() {
        let mut input = base();
        input.recognition_timing_status =
            RecognitionTimingStatus::BuiltInGainRecognizedAfterFiveYearWindow;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BuiltInGainRecognizedAfterFiveYearWindowNoDisallowance
        );
        assert_eq!(output.disallowed_offset_cents, 0);
        assert!(output.note.contains("5-year window"));
    }

    #[test]
    fn no_preacquisition_loss_no_disallowance() {
        let mut input = base();
        input.preacquisition_loss_carryover_cents = 0;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoPreacquisitionLossNoDisallowance);
        assert_eq!(output.disallowed_offset_cents, 0);
    }

    #[test]
    fn section_384a_disallows_loss_corp_offset_of_gain_corp_big() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section384APreacquisitionLossOffsetDisallowed
        );
        // Gain corp own offset = min($2M, $10M) = $2M
        // Remaining BIG = $10M - $2M = $8M
        // Loss corp attempted = $15M - $2M = $13M
        // Disallowed = min($13M, $8M) = $8M
        assert_eq!(output.disallowed_offset_cents, 8_000_000_00);
        assert_eq!(output.allowed_offset_cents, 2_000_000_00);
        assert!(output.note.contains("§ 384(a)"));
        assert!(output.note.contains("§ 382"));
        assert!(output.note.contains("§ 383"));
    }

    #[test]
    fn gain_corp_own_preacquisition_loss_carveout_offsets_own_big() {
        let mut input = base();
        // Gain corp's own preacquisition loss covers half the BIG
        input.gain_corp_own_preacquisition_loss_cents = 5_000_000_00;
        let output = check(&input);
        // Remaining BIG = $10M - $5M = $5M
        // Loss corp attempted = $15M - $5M = $10M
        // Disallowed = min($10M, $5M) = $5M
        assert_eq!(output.disallowed_offset_cents, 5_000_000_00);
        assert_eq!(output.allowed_offset_cents, 5_000_000_00);
    }

    #[test]
    fn gain_corp_full_offset_zero_disallowance() {
        let mut input = base();
        // Gain corp's own preacquisition loss covers the entire BIG
        input.gain_corp_own_preacquisition_loss_cents = 10_000_000_00;
        let output = check(&input);
        // Remaining BIG = 0; disallowance = 0
        assert_eq!(output.disallowed_offset_cents, 0);
        assert_eq!(output.allowed_offset_cents, 10_000_000_00);
    }

    #[test]
    fn loss_smaller_than_big_disallowance_capped_at_loss_amount() {
        let mut input = base();
        input.preacquisition_loss_carryover_cents = 3_000_000_00;
        input.gain_corp_own_preacquisition_loss_cents = 0;
        let output = check(&input);
        // Loss corp attempted = $3M - $0 = $3M
        // Remaining BIG = $10M
        // Disallowed = min($3M, $10M) = $3M
        assert_eq!(output.disallowed_offset_cents, 3_000_000_00);
    }

    #[test]
    fn section_368_reorganization_triggers_section_384_analysis() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::Section368ReorganizationAssetAcquisition;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section384APreacquisitionLossOffsetDisallowed
        );
    }

    #[test]
    fn recognition_period_constant_pins_5_years() {
        assert_eq!(RECOGNITION_PERIOD_YEARS, 5);
    }

    #[test]
    fn control_threshold_constant_pins_80_percent() {
        assert_eq!(CONTROL_THRESHOLD_PERCENT, 80);
    }

    #[test]
    fn common_control_exception_threshold_pins_50_percent() {
        assert_eq!(COMMON_CONTROL_EXCEPTION_THRESHOLD_PERCENT, 50);
    }

    #[test]
    fn very_large_built_in_gain_no_overflow() {
        let mut input = base();
        input.recognized_built_in_gain_cents = u64::MAX;
        input.preacquisition_loss_carryover_cents = u64::MAX;
        input.gain_corp_own_preacquisition_loss_cents = 0;
        let output = check(&input);
        // saturating arithmetic prevents panic
        assert_eq!(output.disallowed_offset_cents, u64::MAX);
    }

    #[test]
    fn zero_built_in_gain_no_disallowance() {
        let mut input = base();
        input.recognized_built_in_gain_cents = 0;
        let output = check(&input);
        // Remaining BIG = 0; min(loss, 0) = 0
        assert_eq!(output.disallowed_offset_cents, 0);
        assert_eq!(output.allowed_offset_cents, 0);
    }

    #[test]
    fn note_pins_section_381_carryover_attribute_transferee_rules() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 381"));
    }

    #[test]
    fn note_pins_section_269_discretionary_disallowance() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 269"));
    }

    #[test]
    fn note_pins_section_1504_a_2_control_standard() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::NoQualifyingAcquisition;
        let output = check(&input);
        assert!(output.note.contains("§ 1504(a)(2)"));
    }

    #[test]
    fn note_pins_5_year_recognition_period() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("5-year"));
    }

    #[test]
    fn common_control_exception_overrides_qualifying_acquisition_disallowance() {
        let mut input = base();
        input.common_control_exception_status =
            CommonControlExceptionStatus::InSameControlledGroupFiveYearsPreAcquisitionExceptionApplies;
        // Even with qualifying acquisition + BIG within window, common control rebuts
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CommonControlExceptionAppliesSection384bThreeNoDisallowance
        );
    }

    #[test]
    fn no_qualifying_acquisition_takes_priority_over_common_control() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::NoQualifyingAcquisition;
        input.common_control_exception_status =
            CommonControlExceptionStatus::InSameControlledGroupFiveYearsPreAcquisitionExceptionApplies;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoQualifyingAcquisitionSection384Inapplicable
        );
    }
}
