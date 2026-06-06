//! IRC § 903 in-lieu-of tax creditability for foreign tax credit.
//!
//! § 903 extends § 901 FTC creditability to foreign taxes that are NOT generally-imposed
//! foreign income/war-profits/excess-profits tax but substitute for such a tax (the
//! "in-lieu-of" branch). Classic application: foreign withholding tax on services/royalties
//! paid by nonresidents where the foreign jurisdiction would otherwise impose income tax
//! on net income but waives that residence-country net-income tax in favor of the gross-basis
//! withholding tax.
//!
//! Treas. Reg. § 1.903-1(c) "substitution" test requires the in-lieu-of tax to substitute
//! for a general income tax that "would otherwise be imposed" on the same taxpayer/income.
//! Treas. Reg. § 1.903-1(c)(1)(iii) "soak-up" rule: foreign tax whose liability depends on
//! availability of FTC against US tax is non-creditable regardless of substitution.
//!
//! TD 9959 (Jan 4 2022) Final Regulations added "attribution requirement" to § 903: the
//! generally-imposed income tax which the levy substitutes for must independently meet
//! attribution rules (sourcing nexus). Notice 2023-55 (Jul 21 2023) granted temporary
//! relief permitting taxpayers to apply former § 1.903-1 without the attribution requirement
//! for tax years ending on or before Dec 31 2023. Notice 2025-23 (per practitioner reports)
//! extended the relief through tax years ending on or before Dec 31 2025.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - rsmus.com/insights/tax-alerts/2023/temporary-relief-granted-foreign-tax-credit.html
//! - ey.com/en_gl/technical/tax-alerts/us-treasury-provides-welcome-temporary-relief-from-controversial
//! - mwe.com/insights/new-attribution-requirement-denies-foreign-tax-credits-for-certain-withholding-taxes-and-other-taxes/
//! - thetaxadviser.com/issues/2024/feb/short-term-relief-for-foreign-tax-credit-woes/
//! - fenwick.com/insights/publications/treasury-finalizes-foreign-tax-credit-regulations-including-novel-jurisdictional-nexus-attribution-rule
//! - federalregister.gov/documents/2022/11/22/2022-25337/guidance-related-to-the-foreign-tax-credit

use serde::{Deserialize, Serialize};

/// Year regime selector controlling whether attribution requirement applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YearRegime {
    /// Tax year ending on or before Dec 31 2023 — Notice 2023-55 deferral active.
    PreOrCalendar2023NoticeDeferralActive,
    /// Tax year ending in 2024 or 2025 — Notice 2025-23 extended deferral active.
    Calendar2024Or2025NoticeDeferralActive,
    /// Tax year ending after Dec 31 2025 — TD 9959 attribution requirement applies fully.
    PostCalendar2025AttributionRequiredFully,
}

/// Whether the foreign levy is a separate levy (gross-basis withholding) or general income tax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevyClass {
    /// Foreign gross-basis withholding tax claimed under § 903 in-lieu-of branch.
    GrossBasisWithholdingInLieuOfBranch,
    /// Foreign net-basis income tax claimed under § 901 (not § 903).
    NetBasisIncomeTaxSection901Branch,
}

/// Whether the in-lieu-of tax substitutes for a generally-imposed foreign income tax
/// (the "substitution" test of Treas. Reg. § 1.903-1(c)(2)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstitutionStatus {
    /// Substitute tax: would-be income tax is waived in favor of the withholding levy.
    SubstitutesForGenerallyImposedIncomeTax,
    /// Levy is additive on top of generally-imposed income tax — fails substitution.
    AdditiveLevyFailsSubstitution,
    /// Foreign jurisdiction imposes no generally-imposed income tax — fails substitution.
    NoGenerallyImposedIncomeTaxFailsSubstitution,
}

/// Whether liability is conditioned on availability of US FTC (soak-up rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoakUpStatus {
    /// Liability independent of US FTC — passes soak-up rule.
    LiabilityIndependentOfUsFtcPasses,
    /// Liability conditioned on US FTC availability — soak-up tax, non-creditable.
    LiabilityConditionedOnUsFtcSoakUpNonCreditable,
}

/// Whether the generally-imposed foreign income tax that the levy substitutes for would
/// independently satisfy the TD 9959 attribution requirement (sourcing nexus).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionStatus {
    /// Generally-imposed income tax satisfies attribution (sourcing nexus present).
    GenerallyImposedTaxSatisfiesAttribution,
    /// Generally-imposed income tax fails attribution (no sourcing nexus).
    GenerallyImposedTaxFailsAttribution,
    /// Not applicable (claim is under § 901 net-basis branch, not § 903).
    NotApplicableSection901Branch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CreditableUnderInLieuOfBranchAttributionDeferred,
    CreditableUnderInLieuOfBranchAttributionMet,
    NonCreditableSoakUpTaxRule,
    NonCreditableFailsSubstitution,
    NonCreditableFailsAttributionPostDeferral,
}

/// Inputs for § 903 in-lieu-of tax creditability test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub year_regime: YearRegime,
    pub levy_class: LevyClass,
    pub substitution_status: SubstitutionStatus,
    pub soak_up_status: SoakUpStatus,
    pub attribution_status: AttributionStatus,
    pub foreign_tax_paid_cents: u64,
}

pub type Section903InLieuOfTaxCreditabilityInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub creditable_amount_cents: u64,
    pub disallowed_amount_cents: u64,
    pub note: String,
}

pub type Section903InLieuOfTaxCreditabilityOutput = Output;

const NOTICE_2023_55_PUBLICATION: &str = "Notice 2023-55 (Jul 21 2023)";
const NOTICE_2025_23_EXTENSION: &str = "Notice 2025-23 (extending Notice 2023-55 through 2025)";
const TD_9959_EFFECTIVE_DATE: &str = "TD 9959 (Jan 4 2022)";

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.levy_class,
        LevyClass::NetBasisIncomeTaxSection901Branch
    ) {
        return Output {
            severity: Severity::NotApplicable,
            creditable_amount_cents: 0,
            disallowed_amount_cents: 0,
            note: "Levy is net-basis income tax under § 901, not § 903 in-lieu-of branch. \
                   Apply § 901(b) net-gain / realization / non-confiscatory tests directly. \
                   § 903 substitution test inapplicable."
                .to_string(),
        };
    }

    if matches!(
        input.soak_up_status,
        SoakUpStatus::LiabilityConditionedOnUsFtcSoakUpNonCreditable
    ) {
        return Output {
            severity: Severity::NonCreditableSoakUpTaxRule,
            creditable_amount_cents: 0,
            disallowed_amount_cents: input.foreign_tax_paid_cents,
            note: format!(
                "Soak-up tax non-creditable per Treas. Reg. § 1.903-1(c)(1)(iii). Foreign tax \
                 liability is conditioned on availability of US foreign tax credit, so the levy \
                 absorbs the credit that would otherwise be allowed. ${} disallowed in full. \
                 Soak-up tax disqualifies regardless of substitution or attribution facts; \
                 substitution analysis not reached.",
                input.foreign_tax_paid_cents / 100
            ),
        };
    }

    if !matches!(
        input.substitution_status,
        SubstitutionStatus::SubstitutesForGenerallyImposedIncomeTax
    ) {
        let reason = match input.substitution_status {
            SubstitutionStatus::AdditiveLevyFailsSubstitution => {
                "Levy is additive on top of generally-imposed foreign income tax rather than \
                 substituting for it. Treas. Reg. § 1.903-1(c)(2) substitution test fails because \
                 the would-be income tax remains imposed alongside the in-lieu-of levy."
            }
            SubstitutionStatus::NoGenerallyImposedIncomeTaxFailsSubstitution => {
                "Foreign jurisdiction imposes no generally-imposed income tax for the levy to \
                 substitute for. § 903 in-lieu-of branch requires an underlying general income \
                 tax that would otherwise apply; without one, the substitution test fails."
            }
            SubstitutionStatus::SubstitutesForGenerallyImposedIncomeTax => unreachable!(),
        };
        return Output {
            severity: Severity::NonCreditableFailsSubstitution,
            creditable_amount_cents: 0,
            disallowed_amount_cents: input.foreign_tax_paid_cents,
            note: format!(
                "Non-creditable under § 903 in-lieu-of branch. {reason} ${} disallowed in full. \
                 Consider § 164(a)(3) deduction in lieu of credit.",
                input.foreign_tax_paid_cents / 100
            ),
        };
    }

    match input.year_regime {
        YearRegime::PreOrCalendar2023NoticeDeferralActive => Output {
            severity: Severity::CreditableUnderInLieuOfBranchAttributionDeferred,
            creditable_amount_cents: input.foreign_tax_paid_cents,
            disallowed_amount_cents: 0,
            note: format!(
                "Creditable under § 903 in-lieu-of branch. {NOTICE_2023_55_PUBLICATION} deferral \
                 active: attribution requirement under {TD_9959_EFFECTIVE_DATE} not applied for \
                 tax years ending on or before Dec 31 2023. Substitution test (Treas. Reg. \
                 § 1.903-1(c)(2)) and soak-up rule (Treas. Reg. § 1.903-1(c)(1)(iii)) both met. \
                 ${} fully creditable subject to § 904 limitation.",
                input.foreign_tax_paid_cents / 100
            ),
        },
        YearRegime::Calendar2024Or2025NoticeDeferralActive => Output {
            severity: Severity::CreditableUnderInLieuOfBranchAttributionDeferred,
            creditable_amount_cents: input.foreign_tax_paid_cents,
            disallowed_amount_cents: 0,
            note: format!(
                "Creditable under § 903 in-lieu-of branch. {NOTICE_2025_23_EXTENSION} active: \
                 attribution requirement deferred for tax years ending in 2024 and 2025. \
                 Substitution and soak-up tests both met. ${} fully creditable subject to § 904 \
                 limitation. Confirm Notice 2025-23 effective dates against IRS published \
                 guidance before relying.",
                input.foreign_tax_paid_cents / 100
            ),
        },
        YearRegime::PostCalendar2025AttributionRequiredFully => match input.attribution_status {
            AttributionStatus::GenerallyImposedTaxSatisfiesAttribution => Output {
                severity: Severity::CreditableUnderInLieuOfBranchAttributionMet,
                creditable_amount_cents: input.foreign_tax_paid_cents,
                disallowed_amount_cents: 0,
                note: format!(
                    "Creditable under § 903 in-lieu-of branch. Post-Notice 2025-23 deferral, \
                     {TD_9959_EFFECTIVE_DATE} attribution requirement applies and the underlying \
                     generally-imposed foreign income tax independently satisfies the sourcing \
                     nexus attribution rule. ${} fully creditable subject to § 904 limitation.",
                    input.foreign_tax_paid_cents / 100
                ),
            },
            AttributionStatus::GenerallyImposedTaxFailsAttribution => Output {
                severity: Severity::NonCreditableFailsAttributionPostDeferral,
                creditable_amount_cents: 0,
                disallowed_amount_cents: input.foreign_tax_paid_cents,
                note: format!(
                    "Non-creditable under § 903. Post-Notice 2025-23 deferral, the underlying \
                     generally-imposed foreign income tax fails the {TD_9959_EFFECTIVE_DATE} \
                     attribution (sourcing nexus) requirement. ${} disallowed in full. Common \
                     trap: gross-basis withholding tax on services performed entirely outside \
                     the imposing jurisdiction without residence, source, or activity nexus. \
                     Consider § 164(a)(3) deduction in lieu of credit.",
                    input.foreign_tax_paid_cents / 100
                ),
            },
            AttributionStatus::NotApplicableSection901Branch => Output {
                severity: Severity::NonCreditableFailsAttributionPostDeferral,
                creditable_amount_cents: 0,
                disallowed_amount_cents: input.foreign_tax_paid_cents,
                note: "Attribution status reported as not-applicable / § 901 branch, but levy \
                       class indicates § 903 in-lieu-of branch. Inconsistent input — supply \
                       attribution status for the underlying generally-imposed foreign income \
                       tax."
                    .to_string(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            year_regime: YearRegime::Calendar2024Or2025NoticeDeferralActive,
            levy_class: LevyClass::GrossBasisWithholdingInLieuOfBranch,
            substitution_status: SubstitutionStatus::SubstitutesForGenerallyImposedIncomeTax,
            soak_up_status: SoakUpStatus::LiabilityIndependentOfUsFtcPasses,
            attribution_status: AttributionStatus::GenerallyImposedTaxSatisfiesAttribution,
            foreign_tax_paid_cents: 100_000_00,
        }
    }

    #[test]
    fn net_basis_section_901_branch_returns_not_applicable() {
        let mut input = base();
        input.levy_class = LevyClass::NetBasisIncomeTaxSection901Branch;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NotApplicable);
        assert!(output.note.contains("§ 901"));
        assert!(output.note.contains("§ 903"));
    }

    #[test]
    fn soak_up_tax_non_creditable_regardless_of_other_facts() {
        let mut input = base();
        input.soak_up_status = SoakUpStatus::LiabilityConditionedOnUsFtcSoakUpNonCreditable;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NonCreditableSoakUpTaxRule);
        assert_eq!(output.creditable_amount_cents, 0);
        assert_eq!(output.disallowed_amount_cents, 100_000_00);
        assert!(output.note.contains("§ 1.903-1(c)(1)(iii)"));
    }

    #[test]
    fn additive_levy_fails_substitution_test() {
        let mut input = base();
        input.substitution_status = SubstitutionStatus::AdditiveLevyFailsSubstitution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NonCreditableFailsSubstitution);
        assert_eq!(output.disallowed_amount_cents, 100_000_00);
        assert!(output.note.contains("§ 1.903-1(c)(2)"));
        assert!(output.note.contains("§ 164(a)(3)"));
    }

    #[test]
    fn no_generally_imposed_income_tax_fails_substitution() {
        let mut input = base();
        input.substitution_status =
            SubstitutionStatus::NoGenerallyImposedIncomeTaxFailsSubstitution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NonCreditableFailsSubstitution);
        assert!(output.note.contains("no generally-imposed income tax"));
    }

    #[test]
    fn pre_2024_deferral_grants_credit_without_attribution_test() {
        let mut input = base();
        input.year_regime = YearRegime::PreOrCalendar2023NoticeDeferralActive;
        input.attribution_status = AttributionStatus::GenerallyImposedTaxFailsAttribution;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CreditableUnderInLieuOfBranchAttributionDeferred
        );
        assert_eq!(output.creditable_amount_cents, 100_000_00);
        assert!(output.note.contains("Notice 2023-55"));
        assert!(output.note.contains("§ 904"));
    }

    #[test]
    fn calendar_2024_2025_deferral_grants_credit_without_attribution_test() {
        let mut input = base();
        input.year_regime = YearRegime::Calendar2024Or2025NoticeDeferralActive;
        input.attribution_status = AttributionStatus::GenerallyImposedTaxFailsAttribution;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CreditableUnderInLieuOfBranchAttributionDeferred
        );
        assert_eq!(output.creditable_amount_cents, 100_000_00);
        assert!(output.note.contains("Notice 2025-23"));
    }

    #[test]
    fn post_2025_attribution_satisfied_grants_credit() {
        let mut input = base();
        input.year_regime = YearRegime::PostCalendar2025AttributionRequiredFully;
        input.attribution_status = AttributionStatus::GenerallyImposedTaxSatisfiesAttribution;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CreditableUnderInLieuOfBranchAttributionMet
        );
        assert_eq!(output.creditable_amount_cents, 100_000_00);
        assert!(output.note.contains("TD 9959"));
        assert!(output.note.contains("§ 904"));
    }

    #[test]
    fn post_2025_attribution_fails_disallows_credit() {
        let mut input = base();
        input.year_regime = YearRegime::PostCalendar2025AttributionRequiredFully;
        input.attribution_status = AttributionStatus::GenerallyImposedTaxFailsAttribution;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NonCreditableFailsAttributionPostDeferral
        );
        assert_eq!(output.disallowed_amount_cents, 100_000_00);
        assert!(output.note.contains("sourcing nexus"));
        assert!(output.note.contains("§ 164(a)(3)"));
    }

    #[test]
    fn soak_up_takes_priority_over_substitution_failure() {
        let mut input = base();
        input.soak_up_status = SoakUpStatus::LiabilityConditionedOnUsFtcSoakUpNonCreditable;
        input.substitution_status = SubstitutionStatus::AdditiveLevyFailsSubstitution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NonCreditableSoakUpTaxRule);
        assert!(output.note.contains("Soak-up tax"));
    }

    #[test]
    fn section_901_branch_overrides_soak_up_substitution_and_attribution() {
        let mut input = base();
        input.levy_class = LevyClass::NetBasisIncomeTaxSection901Branch;
        input.soak_up_status = SoakUpStatus::LiabilityConditionedOnUsFtcSoakUpNonCreditable;
        input.substitution_status = SubstitutionStatus::AdditiveLevyFailsSubstitution;
        input.attribution_status = AttributionStatus::GenerallyImposedTaxFailsAttribution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NotApplicable);
        assert!(output.note.contains("§ 901(b)"));
    }

    #[test]
    fn post_2025_inconsistent_input_section_901_attribution_with_section_903_levy() {
        let mut input = base();
        input.year_regime = YearRegime::PostCalendar2025AttributionRequiredFully;
        input.attribution_status = AttributionStatus::NotApplicableSection901Branch;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NonCreditableFailsAttributionPostDeferral
        );
        assert!(output.note.contains("Inconsistent input"));
    }

    #[test]
    fn substitution_failure_note_pins_treas_reg_1_903_1_c_2() {
        let mut input = base();
        input.substitution_status = SubstitutionStatus::AdditiveLevyFailsSubstitution;
        let output = check(&input);
        assert!(output.note.contains("Treas. Reg. § 1.903-1(c)(2)"));
    }

    #[test]
    fn soak_up_failure_note_pins_treas_reg_1_903_1_c_1_iii() {
        let mut input = base();
        input.soak_up_status = SoakUpStatus::LiabilityConditionedOnUsFtcSoakUpNonCreditable;
        let output = check(&input);
        assert!(output.note.contains("Treas. Reg. § 1.903-1(c)(1)(iii)"));
    }

    #[test]
    fn post_2025_attribution_fail_note_describes_gross_basis_withholding_trap() {
        let mut input = base();
        input.year_regime = YearRegime::PostCalendar2025AttributionRequiredFully;
        input.attribution_status = AttributionStatus::GenerallyImposedTaxFailsAttribution;
        let output = check(&input);
        assert!(output
            .note
            .contains("gross-basis withholding tax on services"));
    }

    #[test]
    fn zero_foreign_tax_paid_creditable_amount_is_zero_when_substitution_met() {
        let mut input = base();
        input.foreign_tax_paid_cents = 0;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CreditableUnderInLieuOfBranchAttributionDeferred
        );
        assert_eq!(output.creditable_amount_cents, 0);
        assert_eq!(output.disallowed_amount_cents, 0);
    }

    #[test]
    fn very_large_foreign_tax_no_overflow_in_disallowed_amount() {
        let mut input = base();
        input.foreign_tax_paid_cents = u64::MAX;
        input.soak_up_status = SoakUpStatus::LiabilityConditionedOnUsFtcSoakUpNonCreditable;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NonCreditableSoakUpTaxRule);
        assert_eq!(output.disallowed_amount_cents, u64::MAX);
    }

    #[test]
    fn note_pins_section_904_limitation_for_creditable_outputs() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 904 limitation"));
    }

    #[test]
    fn note_pins_section_164_a_3_deduction_alternative_for_disallowed() {
        let mut input = base();
        input.substitution_status = SubstitutionStatus::AdditiveLevyFailsSubstitution;
        let output = check(&input);
        assert!(output
            .note
            .contains("§ 164(a)(3) deduction in lieu of credit"));
    }

    #[test]
    fn td_9959_pinned_in_post_deferral_attribution_satisfied_note() {
        let mut input = base();
        input.year_regime = YearRegime::PostCalendar2025AttributionRequiredFully;
        let output = check(&input);
        assert!(output.note.contains("TD 9959"));
        assert!(output.note.contains("sourcing nexus"));
    }

    #[test]
    fn note_2025_23_pinned_in_2024_2025_deferral_note() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("Notice 2025-23"));
    }

    #[test]
    fn note_2023_55_pinned_in_pre_2024_deferral_note() {
        let mut input = base();
        input.year_regime = YearRegime::PreOrCalendar2023NoticeDeferralActive;
        let output = check(&input);
        assert!(output.note.contains("Notice 2023-55"));
        assert!(output.note.contains("Dec 31 2023"));
    }

    #[test]
    fn pre_2024_deferral_disallowance_zero_when_creditable() {
        let mut input = base();
        input.year_regime = YearRegime::PreOrCalendar2023NoticeDeferralActive;
        let output = check(&input);
        assert_eq!(output.disallowed_amount_cents, 0);
        assert_eq!(output.creditable_amount_cents, 100_000_00);
    }
}
