//! IRC § 357 assumption of liability in tax-free corporate transfers.
//!
//! § 357 governs how the assumption of liabilities by a transferee corporation affects
//! gain recognition by the transferor in § 351 transfers and § 368 reorganizations.
//! Companion to § 358 (iter 560 — shareholder basis side) and § 362 (iter 562 —
//! corporation basis side). § 357 contains the critical rules that determine whether
//! liability assumption preserves non-recognition (§ 357(a)) or triggers gain
//! recognition due to tax-avoidance purpose (§ 357(b)) or liabilities-exceed-basis
//! mechanic (§ 357(c)(1)).
//!
//! § 357(a) GENERAL RULE: assumption of a liability of the transferor by the
//! transferee corporation in a § 351 or § 361 exchange does NOT cause the assumed
//! liability to be treated as money or other property received by the transferor —
//! liability assumption preserves non-recognition.
//!
//! § 357(b) TAX-AVOIDANCE EXCEPTION: § 357(a) does NOT apply if the principal purpose
//! of the transferor with respect to the liability assumption was (1) to avoid
//! federal income tax on the exchange OR (2) not a bona-fide business purpose. If
//! the exception applies, ALL liabilities assumed are treated as money received
//! (full boot). Taxpayer bears burden of proof by clear preponderance of evidence
//! that no tax-avoidance purpose existed.
//!
//! § 357(c)(1) EXCESS-LIABILITY GAIN: even where § 357(a) applies (bona-fide
//! business purpose, no tax-avoidance intent), gain is RECOGNIZED to the extent
//! the aggregate amount of liabilities assumed by transferee EXCEEDS the aggregate
//! adjusted basis of the property transferred. Treas. Reg. § 1.357-2.
//!
//! § 357(c)(2) EXCEPTIONS to § 357(c)(1): excess-liability gain rule does NOT
//! apply to: (A) liability the discharge of which would give rise to a deduction
//! (e.g., accounts payable from cash-basis trade or business), or (B) liability
//! described in § 736(a) (retiring-partner payments).
//!
//! § 357(d) DETERMINATION OF AMOUNT OF LIABILITY ASSUMED: liability is treated as
//! assumed by transferee only to the extent the transferor is RELIEVED of the
//! liability. Liabilities to which property is subject (but transferor not
//! relieved) are NOT § 357 liabilities.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_357
//! - law.cornell.edu/cfr/text/26/1.357-1
//! - law.cornell.edu/cfr/text/26/1.357-2
//! - thetaxadviser.com/issues/2017/jan/tax-impact-shareholder-corporate-assumption-liabilities/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurposeStatus {
    /// Bona-fide business purpose; no tax-avoidance principal purpose.
    BonaFideBusinessPurposeNoTaxAvoidance,
    /// Principal purpose was tax avoidance OR no bona-fide business purpose —
    /// § 357(b) exception applies.
    TaxAvoidanceOrNoBonaFideBusinessPurposeSection357B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiabilityType {
    /// Ordinary acquisition / capital debt (NOT excluded under § 357(c)(2)).
    OrdinaryNonExcludedLiability,
    /// Liability whose discharge would give rise to a deduction (e.g., accounts
    /// payable from cash-basis trade or business) — § 357(c)(2)(A) exception.
    DeductibleDischargeLiabilitySection357C2A,
    /// § 736(a) retiring-partner liability — § 357(c)(2)(B) exception.
    Section736ARetiringPartnerLiability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub purpose_status: PurposeStatus,
    pub liability_type: LiabilityType,
    pub aggregate_adjusted_basis_of_property_transferred_cents: u64,
    pub aggregate_liabilities_assumed_cents: u64,
}

pub type Section357LiabilityAssumptionInput = Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section357ANonRecognitionPreservedNoGain,
    Section357BTaxAvoidanceExceptionFullLiabilityTreatedAsBoot,
    Section357C1ExcessLiabilityGainRecognition,
    Section357C2ExceptionAppliesNoExcessLiabilityGain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub gain_recognized_cents: u64,
    pub liability_treated_as_boot_cents: u64,
    pub note: String,
}

pub type Section357LiabilityAssumptionOutput = Output;
pub type Section357LiabilityAssumptionResult = Output;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.purpose_status,
        PurposeStatus::TaxAvoidanceOrNoBonaFideBusinessPurposeSection357B
    ) {
        return Output {
            severity: Severity::Section357BTaxAvoidanceExceptionFullLiabilityTreatedAsBoot,
            gain_recognized_cents: input.aggregate_liabilities_assumed_cents,
            liability_treated_as_boot_cents: input.aggregate_liabilities_assumed_cents,
            note: format!(
                "§ 357(b) TAX-AVOIDANCE EXCEPTION applies. Principal purpose of liability \
                 assumption was tax avoidance OR no bona-fide business purpose. ALL \
                 liabilities assumed (${}) are treated as money received (full boot). \
                 Taxpayer bears burden of proof by clear preponderance of evidence per \
                 Treas. Reg. § 1.357-1(c). Coordinates with § 358(a)(1) shareholder-basis-\
                 reducing-by-boot mechanic + § 362(a) corporation-basis-carryover. § 357(b) \
                 punishes the transferor harshly: even a single dollar of liabilities \
                 becomes taxable boot.",
                input.aggregate_liabilities_assumed_cents / 100
            ),
        };
    }

    if !matches!(
        input.liability_type,
        LiabilityType::OrdinaryNonExcludedLiability
    ) {
        return Output {
            severity: Severity::Section357C2ExceptionAppliesNoExcessLiabilityGain,
            gain_recognized_cents: 0,
            liability_treated_as_boot_cents: 0,
            note: format!(
                "§ 357(c)(2) EXCEPTION applies. Liability is excluded from § 357(c)(1) \
                 excess-liability gain calculation: {}. No gain recognition regardless of \
                 whether liabilities exceed basis. § 357(a) non-recognition preserved.",
                liability_type_label(input.liability_type)
            ),
        };
    }

    if input.aggregate_liabilities_assumed_cents
        > input.aggregate_adjusted_basis_of_property_transferred_cents
    {
        let gain = input
            .aggregate_liabilities_assumed_cents
            .saturating_sub(input.aggregate_adjusted_basis_of_property_transferred_cents);
        return Output {
            severity: Severity::Section357C1ExcessLiabilityGainRecognition,
            gain_recognized_cents: gain,
            liability_treated_as_boot_cents: 0,
            note: format!(
                "§ 357(c)(1) EXCESS-LIABILITY GAIN RECOGNITION applies. Aggregate \
                 liabilities assumed (${}) EXCEEDS aggregate adjusted basis of property \
                 transferred (${}) by ${}. Gain recognized = ${}. Even bona-fide business \
                 purpose + non-tax-avoidance intent does NOT prevent excess-liability gain. \
                 Treas. Reg. § 1.357-2 example: $20K basis + $30K mortgage → $10K gain. \
                 Coordinates with § 358(a) shareholder basis floor (cannot go below zero) + \
                 § 1245 / § 1250 depreciation recapture as ordinary-income portion. Common \
                 in highly-leveraged real-estate-to-corp transfers + LLC-to-corp conversions \
                 + partnership-to-corp incorporations.",
                input.aggregate_liabilities_assumed_cents / 100,
                input.aggregate_adjusted_basis_of_property_transferred_cents / 100,
                gain / 100,
                gain / 100
            ),
        };
    }

    Output {
        severity: Severity::Section357ANonRecognitionPreservedNoGain,
        gain_recognized_cents: 0,
        liability_treated_as_boot_cents: 0,
        note: format!(
            "§ 357(a) NON-RECOGNITION PRESERVED. Liability assumption (${}) does NOT cause \
             gain because (1) bona-fide business purpose under § 357(b) + (2) liabilities \
             assumed do NOT exceed aggregate basis of property transferred (${}) per \
             § 357(c)(1). Liability is treated as transferor-relieved per § 357(d). \
             Coordinates with § 358(a)(1) shareholder-basis reduction by liability amount + \
             § 362(a) corporation carryover basis + § 351 control-immediately-after \
             requirement (§ 368(c) 80%).",
            input.aggregate_liabilities_assumed_cents / 100,
            input.aggregate_adjusted_basis_of_property_transferred_cents / 100
        ),
    }
}

fn liability_type_label(t: LiabilityType) -> &'static str {
    match t {
        LiabilityType::DeductibleDischargeLiabilitySection357C2A => {
            "§ 357(c)(2)(A) — liability the discharge of which would give rise to a \
             deduction (e.g., accounts payable from cash-basis trade or business)"
        }
        LiabilityType::Section736ARetiringPartnerLiability => {
            "§ 357(c)(2)(B) — § 736(a) retiring-partner liability"
        }
        LiabilityType::OrdinaryNonExcludedLiability => "ordinary acquisition or capital debt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            purpose_status: PurposeStatus::BonaFideBusinessPurposeNoTaxAvoidance,
            liability_type: LiabilityType::OrdinaryNonExcludedLiability,
            aggregate_adjusted_basis_of_property_transferred_cents: 100_000_00,
            aggregate_liabilities_assumed_cents: 50_000_00,
        }
    }

    #[test]
    fn section_357a_non_recognition_preserved_when_basis_exceeds_liabilities() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357ANonRecognitionPreservedNoGain
        );
        assert_eq!(output.gain_recognized_cents, 0);
        assert!(output.note.contains("§ 357(a)"));
        assert!(output.note.contains("§ 358(a)(1)"));
        assert!(output.note.contains("§ 362(a)"));
    }

    #[test]
    fn section_357b_tax_avoidance_full_liability_as_boot() {
        let mut input = base();
        input.purpose_status =
            PurposeStatus::TaxAvoidanceOrNoBonaFideBusinessPurposeSection357B;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357BTaxAvoidanceExceptionFullLiabilityTreatedAsBoot
        );
        // ALL $50K liability treated as boot
        assert_eq!(output.gain_recognized_cents, 50_000_00);
        assert_eq!(output.liability_treated_as_boot_cents, 50_000_00);
        assert!(output.note.contains("§ 357(b)"));
        assert!(output.note.contains("clear preponderance"));
        assert!(output.note.contains("Treas. Reg. § 1.357-1(c)"));
    }

    #[test]
    fn section_357c1_excess_liability_gain_when_liabilities_exceed_basis() {
        let mut input = base();
        input.aggregate_liabilities_assumed_cents = 130_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357C1ExcessLiabilityGainRecognition
        );
        // $130K - $100K = $30K gain
        assert_eq!(output.gain_recognized_cents, 30_000_00);
        assert!(output.note.contains("§ 357(c)(1)"));
        assert!(output.note.contains("Treas. Reg. § 1.357-2"));
        assert!(output.note.contains("§ 1245"));
        assert!(output.note.contains("§ 1250"));
    }

    #[test]
    fn section_357c1_classic_example_20k_basis_30k_mortgage_10k_gain() {
        let mut input = base();
        input.aggregate_adjusted_basis_of_property_transferred_cents = 20_000_00;
        input.aggregate_liabilities_assumed_cents = 30_000_00;
        let output = check(&input);
        // Treas. Reg. § 1.357-2 classic example
        assert_eq!(output.gain_recognized_cents, 10_000_00);
    }

    #[test]
    fn section_357c2a_deductible_discharge_exception_no_gain() {
        let mut input = base();
        input.liability_type =
            LiabilityType::DeductibleDischargeLiabilitySection357C2A;
        input.aggregate_liabilities_assumed_cents = 200_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357C2ExceptionAppliesNoExcessLiabilityGain
        );
        assert_eq!(output.gain_recognized_cents, 0);
        assert!(output.note.contains("§ 357(c)(2)(A)"));
        assert!(output.note.contains("accounts payable from cash-basis"));
    }

    #[test]
    fn section_357c2b_section_736a_retiring_partner_no_gain() {
        let mut input = base();
        input.liability_type = LiabilityType::Section736ARetiringPartnerLiability;
        input.aggregate_liabilities_assumed_cents = 200_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357C2ExceptionAppliesNoExcessLiabilityGain
        );
        assert!(output.note.contains("§ 357(c)(2)(B)"));
        assert!(output.note.contains("§ 736(a)"));
    }

    #[test]
    fn section_357b_takes_priority_over_357c() {
        let mut input = base();
        input.purpose_status =
            PurposeStatus::TaxAvoidanceOrNoBonaFideBusinessPurposeSection357B;
        input.aggregate_liabilities_assumed_cents = 200_000_00;
        // Tax-avoidance: all $200K treated as boot (not the $100K excess gain)
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357BTaxAvoidanceExceptionFullLiabilityTreatedAsBoot
        );
        assert_eq!(output.gain_recognized_cents, 200_000_00);
    }

    #[test]
    fn section_357c2_exception_overrides_357c1_excess_gain() {
        let mut input = base();
        input.liability_type =
            LiabilityType::DeductibleDischargeLiabilitySection357C2A;
        input.aggregate_liabilities_assumed_cents = 500_000_00;
        // Even with $400K excess, § 357(c)(2)(A) exception → no gain
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section357C2ExceptionAppliesNoExcessLiabilityGain
        );
        assert_eq!(output.gain_recognized_cents, 0);
    }

    #[test]
    fn very_large_liabilities_no_overflow() {
        let mut input = base();
        input.aggregate_liabilities_assumed_cents = u64::MAX;
        let output = check(&input);
        // saturating_sub prevents overflow
        assert_eq!(
            output.severity,
            Severity::Section357C1ExcessLiabilityGainRecognition
        );
        assert!(output.gain_recognized_cents > 0);
    }

    #[test]
    fn zero_liability_no_gain() {
        let mut input = base();
        input.aggregate_liabilities_assumed_cents = 0;
        let output = check(&input);
        assert_eq!(output.gain_recognized_cents, 0);
    }

    #[test]
    fn zero_basis_full_liability_excess_gain() {
        let mut input = base();
        input.aggregate_adjusted_basis_of_property_transferred_cents = 0;
        input.aggregate_liabilities_assumed_cents = 50_000_00;
        let output = check(&input);
        // $50K - $0 = $50K excess gain
        assert_eq!(output.gain_recognized_cents, 50_000_00);
    }

    #[test]
    fn note_pins_section_358_companion_for_357a() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 358"));
    }

    #[test]
    fn note_pins_section_362_companion_for_357a() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 362"));
    }

    #[test]
    fn note_pins_section_357d_relieved_of_liability() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 357(d)"));
        assert!(output.note.contains("relieved"));
    }

    #[test]
    fn note_pins_section_368c_80_pct_control() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 368(c)"));
    }

    #[test]
    fn note_pins_treas_reg_1_357_2_classic_example_in_excess_gain() {
        let mut input = base();
        input.aggregate_liabilities_assumed_cents = 130_000_00;
        let output = check(&input);
        assert!(output.note.contains("Treas. Reg. § 1.357-2"));
    }

    #[test]
    fn boundary_liability_equals_basis_no_gain() {
        let mut input = base();
        input.aggregate_liabilities_assumed_cents = 100_000_00;
        let output = check(&input);
        // Exactly equal → no excess
        assert_eq!(
            output.severity,
            Severity::Section357ANonRecognitionPreservedNoGain
        );
    }
}
