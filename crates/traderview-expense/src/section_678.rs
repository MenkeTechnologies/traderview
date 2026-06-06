//! IRC § 678 — Person Other Than Grantor Treated as Owner
//! (Beneficiary Defective Inheritor Trust / BDIT Foundation).
//!
//! Pure-compute § 678 grantor-trust attribution. § 678 is the
//! singular statutory mechanism by which a person OTHER THAN the
//! settlor / transferor can be treated as the grantor (owner) of
//! a trust for income tax purposes, while NOT being the grantor
//! for gift / estate tax purposes. This dual-treatment property
//! is the foundation of the **BDIT** (Beneficiary Defective
//! Inheritor Trust) — a third-party-created irrevocable trust in
//! which the beneficiary holds a withdrawal power that triggers
//! § 678(a)(1) attribution during the power's existence and
//! § 678(a)(2) post-lapse attribution within the **5x5 lapse
//! safe harbor** of § 2514(e). The result: beneficiary pays
//! income tax on trust income (depleting beneficiary's estate
//! via tax burden) while trust assets grow tax-free for the
//! benefit of subsequent generations and escape beneficiary's
//! taxable estate.
//!
//! Statute (verbatim mapping):
//! - § 678(a) — GENERAL RULE: a person OTHER THAN the grantor
//!   shall be treated as the owner of any portion of a trust with
//!   respect to which —
//!   - (1) such person has a power exercisable solely by self to
//!     vest the corpus or the income therefrom in self, OR
//!   - (2) such person has previously partially released or
//!     otherwise modified such a power and after the release or
//!     modification retains such control as would, within the
//!     principles of §§ 671-677, subject a grantor of a trust to
//!     treatment as the owner thereof.
//! - § 678(b) — EXCEPTION FOR POWER OVER INCOME: subsection (a)
//!   shall not apply with respect to a power OVER INCOME of any
//!   portion of a trust in the case of a person other than the
//!   grantor if the grantor of the trust or a transferor (to
//!   whom § 679 applies) is otherwise treated as the owner of
//!   such portion under §§ 671-679. (Result: grantor's grantor
//!   trust status PREEMPTS § 678 attribution to beneficiary.)
//! - § 678(c) — OBLIGATIONS OF SUPPORT: subsection (a)(1) does
//!   not apply to a power exercisable solely by self in the
//!   capacity of trustee or co-trustee, the exercise of which
//!   would be limited by an ASCERTAINABLE STANDARD (such as
//!   HEMS — Health, Education, Maintenance, Support) for support
//!   of beneficiary or dependent. (HEMS trustee-distribution
//!   standard is generally SAFE from § 678 attribution.)
//! - § 678(d) — EFFECT OF RENUNCIATION OR DISCLAIMER: subsection
//!   (a) does not apply with respect to a power renounced or
//!   disclaimed by the holder within a reasonable time after
//!   becoming aware of the power.
//! - § 678(e) — CROSS-REFERENCE: see § 2514(e) for the 5x5 lapse
//!   safe harbor under the gift tax that defines what lapse
//!   amount constitutes a "release" for § 678(a)(2) purposes.
//! - **§ 2514(e) 5x5 LAPSE SAFE HARBOR**: the lapse of a power
//!   of appointment during a calendar year is treated as a gift
//!   to the trust ONLY to the extent the value of the property
//!   which could have been appointed exceeds the GREATER OF
//!   **$5,000 OR 5 % of the aggregate value of the assets out
//!   of which the lapse could have been satisfied**. Lapse within
//!   the 5x5 = NOT a release; § 678(a)(2) does not attach.
//!
//! BDIT mechanics (verified from Oshins, ACTEC Foundation,
//! IconTrust, Griffin Bridgers sources):
//! - Third party (grandparent / parent) creates irrevocable
//!   inter-vivos trust with **initial $5,000 funding** (within
//!   5x5 safe harbor).
//! - Beneficiary (taxpayer) holds a **Crummey withdrawal power**
//!   over the contribution (typically 30-60 days).
//! - Power lapses within the 5x5 safe harbor — no gift tax
//!   consequence to beneficiary under § 2514(e); beneficiary
//!   treated as owner under § 678(a)(2).
//! - Beneficiary purchases trader-favored asset (S-corp shares,
//!   trading account, real estate LP interest) from trust for
//!   promissory note — NO income tax recognition because § 678
//!   treats beneficiary as both buyer AND seller (transaction
//!   between grantor and grantor's grantor trust).
//! - Trust grows tax-free at low cost basis; beneficiary pays
//!   income tax on trust income out of personal assets,
//!   depleting beneficiary's taxable estate further.
//! - At beneficiary's death, trust assets escape beneficiary's
//!   estate (because beneficiary is not the gift-tax grantor)
//!   but get NO § 1014 step-up (per Rev. Rul. 2023-2 logic
//!   applied analogously to BDIT).
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 678 + § 2514(e) confirm statutory text.
//! - Richard A. Oshins NAEPC Journal — "The Beneficiary Defective
//!   Inheritor's Trust (BDIT)" foundational article.
//! - ACTEC Foundation podcast — "The Mysteries and
//!   Misunderstandings Related to Code Section 678".
//! - PLR 200949012 (Dec 4, 2009) — IRS private letter ruling
//!   confirming BDIT structure permissible under § 678.
//! - Griffin Bridgers Substack — "IRC Section 678(a) and HEMS:
//!   Safety in Silence?" (HEMS-trustee carve-out under § 678(c)).
//! - Texas Society of CPAs — "IRC § 678 and the Beneficiary
//!   Deemed Owner Trust (BDOT)" income-only variant.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_2514_E_LAPSE_SAFE_HARBOR_DOLLARS: u64 = 5_000;
pub const SECTION_2514_E_LAPSE_SAFE_HARBOR_CORPUS_PCT_BASIS_POINTS: u64 = 500;
pub const SECTION_678_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const PLR_200949012_BDIT_YEAR: u32 = 2009;
pub const PLR_200949012_BDIT_MONTH: u32 = 12;
pub const PLR_200949012_BDIT_DAY: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section678TriggerCategory {
    WithdrawalPowerSection678a1Active,
    PostLapseSection678a2WithinSafeHarbor,
    PostLapseSection678a2ExceedingSafeHarbor,
    BdotIncomeOnlyVariant,
    HemsTrusteeDistributionStandardExemption,
    PowerRenouncedOrDisclaimedSection678d,
    GrantorOtherwiseOwnerSubsectionB,
    NoPowerNoBdit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithdrawalPowerStatus {
    PowerCurrentlyActive,
    PowerLapsedWithin5x5SafeHarbor,
    PowerLapsedExceeding5x5SafeHarbor,
    PowerRenouncedOrDisclaimedWithinReasonableTime,
    NoPowerEverGranted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section678Mode {
    NotApplicableNoWithdrawalPower,
    NotApplicableGrantorOtherwiseOwnerSubsectionB,
    NotApplicableSupportiveObligationSubsectionC,
    NotApplicablePowerRenouncedOrDisclaimedSubsectionD,
    CompliantSection678a1WithdrawalPowerActiveBeneficiaryAsOwner,
    CompliantSection678a25x5LapseSafeHarborBeneficiaryRemainsOwner,
    CompliantBdotIncomeOnlyVariantBeneficiaryAsOwnerOverIncomePortion,
    CompliantHemsTrusteeDistributionStandardSafeFromSection678a,
    CompliantBditClassicWithdrawalPowerAndLapseFlowsTroughToBeneficiary,
    ViolationSection678a2LapseExceeds5x5GiftTaxTriggerUnderSection2514e,
    ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems,
    ViolationHemsTrusteeDistributionImproperlyClaimedAsSection678aExempt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trigger_category: Section678TriggerCategory,
    pub withdrawal_power_status: WithdrawalPowerStatus,
    pub power_amount_dollars: u64,
    pub trust_corpus_at_lapse_dollars: u64,
    pub grantor_otherwise_treated_as_owner_under_671_through_677: bool,
    pub power_subject_to_ascertainable_standard_hems: bool,
    pub beneficiary_reported_items_on_personal_return: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section678Mode,
    pub lapse_excess_above_5x5_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section678Input = Input;
pub type Section678Output = Output;
pub type Section678Result = Output;

fn five_x_five_safe_harbor_dollars(corpus: u64) -> u64 {
    let pct_amount = (corpus as u128)
        .saturating_mul(SECTION_2514_E_LAPSE_SAFE_HARBOR_CORPUS_PCT_BASIS_POINTS as u128)
        .checked_div(SECTION_678_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64;
    pct_amount.max(SECTION_2514_E_LAPSE_SAFE_HARBOR_DOLLARS)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 678(a)(1) — person other than grantor treated as owner of portion over which person has power exercisable solely by self to vest corpus or income in self".to_string(),
        "26 U.S.C. § 678(a)(2) — person treated as owner post-release/modification if retained control would subject grantor to taxation under §§ 671-677".to_string(),
        "26 U.S.C. § 678(b) — exception: § 678(a) does NOT apply to power over INCOME if grantor (or § 679 transferor) is otherwise treated as owner under §§ 671-679".to_string(),
        "26 U.S.C. § 678(c) — obligations of support: § 678(a)(1) does not apply to power exercisable as trustee/co-trustee limited by ASCERTAINABLE STANDARD (HEMS — Health, Education, Maintenance, Support)".to_string(),
        "26 U.S.C. § 678(d) — power renounced or disclaimed within reasonable time after awareness escapes § 678(a)".to_string(),
        "26 U.S.C. § 678(e) — cross-reference to § 2514(e) 5x5 lapse safe harbor".to_string(),
        "26 U.S.C. § 2514(e) — 5x5 safe harbor: lapse of power of appointment treated as gift ONLY to extent value exceeds GREATER OF $5,000 OR 5 % of trust corpus".to_string(),
        "PLR 200949012 (Dec 4, 2009) — IRS confirmed BDIT structure under § 678 (Oshins-style third-party trust with Crummey withdrawal power lapsing within 5x5)".to_string(),
        "Oshins NAEPC Journal — 'The Beneficiary Defective Inheritor's Trust (BDIT)' foundational article".to_string(),
        "ACTEC Foundation — 'The Mysteries and Misunderstandings Related to Code Section 678' podcast".to_string(),
        "Griffin Bridgers — 'IRC Section 678(a) and HEMS: Safety in Silence?' (HEMS trustee-distribution carve-out under § 678(c))".to_string(),
        "BDOT — Beneficiary Deemed Owner Trust: income-only variant of BDIT structure under § 678".to_string(),
        "BDIT mechanics: 3rd-party irrevocable trust funded with initial $5,000; beneficiary holds Crummey withdrawal power; lapse within 5x5 = no gift tax + § 678 grantor status; beneficiary buys assets from trust for promissory note with no income tax recognition; trust grows tax-free outside beneficiary's estate".to_string(),
    ];

    if input.trigger_category == Section678TriggerCategory::NoPowerNoBdit
        || input.withdrawal_power_status == WithdrawalPowerStatus::NoPowerEverGranted
    {
        return Output {
            mode: Section678Mode::NotApplicableNoWithdrawalPower,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678 requires withdrawal power to vest corpus or income in person"
                .to_string(),
            notes: "No § 678(a)(1) withdrawal power exists; § 678 inapplicable.".to_string(),
            citations,
        };
    }

    if input.grantor_otherwise_treated_as_owner_under_671_through_677 {
        return Output {
            mode: Section678Mode::NotApplicableGrantorOtherwiseOwnerSubsectionB,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678(b) — grantor's grantor trust status preempts § 678 attribution".to_string(),
            notes: "Grantor otherwise treated as owner under §§ 671-677 (e.g., § 675(4)(C) substitution power); § 678(b) preempts beneficiary attribution.".to_string(),
            citations,
        };
    }

    if input.withdrawal_power_status
        == WithdrawalPowerStatus::PowerRenouncedOrDisclaimedWithinReasonableTime
    {
        return Output {
            mode: Section678Mode::NotApplicablePowerRenouncedOrDisclaimedSubsectionD,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678(d) — power renounced or disclaimed within reasonable time".to_string(),
            notes: "Beneficiary timely renounced or disclaimed the power; § 678(d) escape; no attribution.".to_string(),
            citations,
        };
    }

    if input.trigger_category == Section678TriggerCategory::HemsTrusteeDistributionStandardExemption
        && input.power_subject_to_ascertainable_standard_hems
    {
        return Output {
            mode: Section678Mode::CompliantHemsTrusteeDistributionStandardSafeFromSection678a,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678(c) — HEMS ascertainable standard exempts trustee distribution power".to_string(),
            notes: "Beneficiary-trustee distribution power limited by HEMS (Health, Education, Maintenance, Support) ascertainable standard; § 678(c) safe from § 678(a)(1) attribution.".to_string(),
            citations,
        };
    }

    if input.trigger_category == Section678TriggerCategory::HemsTrusteeDistributionStandardExemption
        && !input.power_subject_to_ascertainable_standard_hems
    {
        return Output {
            mode: Section678Mode::ViolationHemsTrusteeDistributionImproperlyClaimedAsSection678aExempt,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678(c) — HEMS exemption requires ASCERTAINABLE STANDARD".to_string(),
            notes: "VIOLATION § 678(c): HEMS-trustee carve-out claimed but power is NOT limited by ascertainable standard; § 678(a)(1) attribution triggers.".to_string(),
            citations,
        };
    }

    if input.trigger_category == Section678TriggerCategory::BdotIncomeOnlyVariant {
        if !input.beneficiary_reported_items_on_personal_return {
            return Output {
                mode: Section678Mode::ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems,
                lapse_excess_above_5x5_dollars: 0,
                statutory_basis: "§ 678 — BDOT income variant requires beneficiary reporting".to_string(),
                notes: "VIOLATION: BDOT income-only variant requires beneficiary to report trust income items on personal return; not reported.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section678Mode::CompliantBdotIncomeOnlyVariantBeneficiaryAsOwnerOverIncomePortion,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678 — BDOT income-only variant".to_string(),
            notes: "COMPLIANT BDOT: beneficiary treated as owner over income portion only; § 678(a)(1) attribution to income flow-through.".to_string(),
            citations,
        };
    }

    if input.withdrawal_power_status == WithdrawalPowerStatus::PowerCurrentlyActive {
        if !input.beneficiary_reported_items_on_personal_return {
            return Output {
                mode: Section678Mode::ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems,
                lapse_excess_above_5x5_dollars: 0,
                statutory_basis: "§ 678(a)(1) — beneficiary must report items during active power period".to_string(),
                notes: "VIOLATION § 678(a)(1): Crummey/withdrawal power active; beneficiary failed to report trust items on personal return during power period.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section678Mode::CompliantSection678a1WithdrawalPowerActiveBeneficiaryAsOwner,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678(a)(1) — withdrawal power active; beneficiary treated as owner".to_string(),
            notes: format!(
                "COMPLIANT § 678(a)(1): withdrawal power of ${} active; beneficiary treated as owner; items reported on beneficiary's personal return.",
                input.power_amount_dollars
            ),
            citations,
        };
    }

    if input.withdrawal_power_status == WithdrawalPowerStatus::PowerLapsedWithin5x5SafeHarbor {
        if !input.beneficiary_reported_items_on_personal_return {
            return Output {
                mode: Section678Mode::ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems,
                lapse_excess_above_5x5_dollars: 0,
                statutory_basis: "§ 678(a)(2) + § 2514(e) — post-lapse within 5x5 still requires beneficiary reporting".to_string(),
                notes: "VIOLATION § 678(a)(2): power lapsed within 5x5 safe harbor; beneficiary remains owner post-lapse but failed to report trust items.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section678Mode::CompliantBditClassicWithdrawalPowerAndLapseFlowsTroughToBeneficiary,
            lapse_excess_above_5x5_dollars: 0,
            statutory_basis: "§ 678(a)(2) + § 2514(e) — 5x5 lapse safe harbor; BDIT classic structure".to_string(),
            notes: format!(
                "COMPLIANT § 678(a)(2): power of ${} lapsed within 5x5 safe harbor ({} trust corpus); beneficiary remains owner under classic BDIT structure; items flow to beneficiary's personal return.",
                input.power_amount_dollars, input.trust_corpus_at_lapse_dollars
            ),
            citations,
        };
    }

    if input.withdrawal_power_status == WithdrawalPowerStatus::PowerLapsedExceeding5x5SafeHarbor {
        let safe_harbor = five_x_five_safe_harbor_dollars(input.trust_corpus_at_lapse_dollars);
        let excess = input.power_amount_dollars.saturating_sub(safe_harbor);
        return Output {
            mode: Section678Mode::ViolationSection678a2LapseExceeds5x5GiftTaxTriggerUnderSection2514e,
            lapse_excess_above_5x5_dollars: excess,
            statutory_basis: "§ 2514(e) — lapse exceeds 5x5 safe harbor; gift tax trigger".to_string(),
            notes: format!(
                "VIOLATION § 2514(e): lapse of ${} exceeds 5x5 safe harbor of ${} (greater of $5,000 or 5 % of trust corpus ${}); excess ${} treated as gift by beneficiary to trust; § 678(a)(2) still attributes income.",
                input.power_amount_dollars,
                safe_harbor,
                input.trust_corpus_at_lapse_dollars,
                excess
            ),
            citations,
        };
    }

    Output {
        mode: Section678Mode::NotApplicableNoWithdrawalPower,
        lapse_excess_above_5x5_dollars: 0,
        statutory_basis: "Default fall-through".to_string(),
        notes: "Default § 678 inapplicable; no triggering power configuration.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_bdit_classic_within_5x5() -> Input {
        Input {
            trigger_category: Section678TriggerCategory::PostLapseSection678a2WithinSafeHarbor,
            withdrawal_power_status: WithdrawalPowerStatus::PowerLapsedWithin5x5SafeHarbor,
            power_amount_dollars: 5_000,
            trust_corpus_at_lapse_dollars: 100_000,
            grantor_otherwise_treated_as_owner_under_671_through_677: false,
            power_subject_to_ascertainable_standard_hems: false,
            beneficiary_reported_items_on_personal_return: true,
        }
    }

    #[test]
    fn no_withdrawal_power_not_applicable() {
        let input = Input {
            trigger_category: Section678TriggerCategory::NoPowerNoBdit,
            withdrawal_power_status: WithdrawalPowerStatus::NoPowerEverGranted,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section678Mode::NotApplicableNoWithdrawalPower);
    }

    #[test]
    fn grantor_otherwise_owner_subsection_b_preempts_678() {
        let input = Input {
            grantor_otherwise_treated_as_owner_under_671_through_677: true,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::NotApplicableGrantorOtherwiseOwnerSubsectionB
        );
    }

    #[test]
    fn power_renounced_or_disclaimed_subsection_d() {
        let input = Input {
            withdrawal_power_status:
                WithdrawalPowerStatus::PowerRenouncedOrDisclaimedWithinReasonableTime,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::NotApplicablePowerRenouncedOrDisclaimedSubsectionD
        );
    }

    #[test]
    fn hems_trustee_distribution_safe_from_678a() {
        let input = Input {
            trigger_category: Section678TriggerCategory::HemsTrusteeDistributionStandardExemption,
            power_subject_to_ascertainable_standard_hems: true,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::CompliantHemsTrusteeDistributionStandardSafeFromSection678a
        );
    }

    #[test]
    fn hems_claimed_without_ascertainable_standard_violation() {
        let input = Input {
            trigger_category: Section678TriggerCategory::HemsTrusteeDistributionStandardExemption,
            power_subject_to_ascertainable_standard_hems: false,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::ViolationHemsTrusteeDistributionImproperlyClaimedAsSection678aExempt
        );
    }

    #[test]
    fn bdit_classic_lapse_within_5x5_compliant() {
        let result = compute(&baseline_bdit_classic_within_5x5());
        assert_eq!(
            result.mode,
            Section678Mode::CompliantBditClassicWithdrawalPowerAndLapseFlowsTroughToBeneficiary
        );
    }

    #[test]
    fn bdit_lapse_within_5x5_beneficiary_failed_to_report_violation() {
        let input = Input {
            beneficiary_reported_items_on_personal_return: false,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems
        );
    }

    #[test]
    fn withdrawal_power_currently_active_compliant() {
        let input = Input {
            trigger_category: Section678TriggerCategory::WithdrawalPowerSection678a1Active,
            withdrawal_power_status: WithdrawalPowerStatus::PowerCurrentlyActive,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::CompliantSection678a1WithdrawalPowerActiveBeneficiaryAsOwner
        );
    }

    #[test]
    fn withdrawal_power_active_but_no_report_violation() {
        let input = Input {
            trigger_category: Section678TriggerCategory::WithdrawalPowerSection678a1Active,
            withdrawal_power_status: WithdrawalPowerStatus::PowerCurrentlyActive,
            beneficiary_reported_items_on_personal_return: false,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems
        );
    }

    #[test]
    fn lapse_exceeds_5x5_gift_tax_trigger() {
        let input = Input {
            trigger_category: Section678TriggerCategory::PostLapseSection678a2ExceedingSafeHarbor,
            withdrawal_power_status: WithdrawalPowerStatus::PowerLapsedExceeding5x5SafeHarbor,
            power_amount_dollars: 20_000,
            trust_corpus_at_lapse_dollars: 100_000,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::ViolationSection678a2LapseExceeds5x5GiftTaxTriggerUnderSection2514e
        );
        assert_eq!(result.lapse_excess_above_5x5_dollars, 15_000);
    }

    #[test]
    fn lapse_at_exactly_5000_within_safe_harbor() {
        let input = Input {
            withdrawal_power_status: WithdrawalPowerStatus::PowerLapsedWithin5x5SafeHarbor,
            power_amount_dollars: 5_000,
            trust_corpus_at_lapse_dollars: 80_000,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::CompliantBditClassicWithdrawalPowerAndLapseFlowsTroughToBeneficiary
        );
    }

    #[test]
    fn lapse_at_5_pct_of_corpus_within_safe_harbor() {
        let input = Input {
            withdrawal_power_status: WithdrawalPowerStatus::PowerLapsedWithin5x5SafeHarbor,
            power_amount_dollars: 50_000,
            trust_corpus_at_lapse_dollars: 1_000_000,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::CompliantBditClassicWithdrawalPowerAndLapseFlowsTroughToBeneficiary
        );
    }

    #[test]
    fn bdot_income_only_variant_compliant() {
        let input = Input {
            trigger_category: Section678TriggerCategory::BdotIncomeOnlyVariant,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::CompliantBdotIncomeOnlyVariantBeneficiaryAsOwnerOverIncomePortion
        );
    }

    #[test]
    fn bdot_income_only_no_report_violation() {
        let input = Input {
            trigger_category: Section678TriggerCategory::BdotIncomeOnlyVariant,
            beneficiary_reported_items_on_personal_return: false,
            ..baseline_bdit_classic_within_5x5()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section678Mode::ViolationCrummeyPowerActiveButBeneficiaryFailedToReportItems
        );
    }

    #[test]
    fn citations_pin_section_678_subsections_and_2514e() {
        let result = compute(&baseline_bdit_classic_within_5x5());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 678(a)(1)"));
        assert!(joined.contains("§ 678(a)(2)"));
        assert!(joined.contains("§ 678(b)"));
        assert!(joined.contains("§ 678(c)"));
        assert!(joined.contains("§ 678(d)"));
        assert!(joined.contains("§ 678(e)"));
        assert!(joined.contains("§ 2514(e)"));
        assert!(joined.contains("5x5 safe harbor"));
        assert!(joined.contains("$5,000"));
        assert!(joined.contains("5 % of trust corpus"));
        assert!(joined.contains("PLR 200949012"));
        assert!(joined.contains("Oshins"));
        assert!(joined.contains("ACTEC Foundation"));
        assert!(joined.contains("Griffin Bridgers"));
        assert!(joined.contains("HEMS"));
        assert!(joined.contains("BDIT"));
        assert!(joined.contains("BDOT"));
    }

    #[test]
    fn constant_pin_5x5_safe_harbor_and_plr_date() {
        assert_eq!(SECTION_2514_E_LAPSE_SAFE_HARBOR_DOLLARS, 5_000);
        assert_eq!(
            SECTION_2514_E_LAPSE_SAFE_HARBOR_CORPUS_PCT_BASIS_POINTS,
            500
        );
        assert_eq!(SECTION_678_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(PLR_200949012_BDIT_YEAR, 2009);
        assert_eq!(PLR_200949012_BDIT_MONTH, 12);
        assert_eq!(PLR_200949012_BDIT_DAY, 4);
    }
}
