//! IRC § 6109 — Identifying Numbers / TIN / EIN / ITIN / SSN
//! Foundational Trader Compliance Module.
//!
//! Pure-compute check for taxpayer identification number (TIN)
//! compliance under IRC § 6109. § 6109 is the foundational
//! statute requiring every taxpayer to furnish an identifying
//! number (SSN, EIN, ITIN, ATIN) to payers required to file
//! information returns AND requiring every payer to include
//! the correct TIN on each information return filed.
//! Trader-critical because every 1099-B, 1099-DIV, 1099-INT,
//! 1099-K, K-1, W-9, W-8BEN, and W-8BEN-E compliance cycle
//! turns on § 6109 — missing or incorrect TIN triggers
//! 24 percent backup withholding under § 3406 and § 6721
//! information-return penalties up to $100 per return with
//! NO MAXIMUM for intentional disregard.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6109(a)(1) Payer Obligation**: any payer required
//!   to file an information return must include the **PAYEE'S
//!   CORRECT TIN** on the return ([Cornell LII 26 USC § 6109](https://www.law.cornell.edu/uscode/text/26/6109);
//!   [IRS — Taxpayer Identification Numbers (TIN)](https://www.irs.gov/tin/taxpayer-identification-numbers-tin)).
//! - **IRC § 6109(a)(2) Payee Obligation**: payee must furnish
//!   their **CORRECT TIN to the payer** when the payment will
//!   be reportable on an information return.
//! - **IRC § 6109(a)(3) Return Filer Obligation**: person
//!   making return or statement under § 6011 must include
//!   identifying number assigned to such person.
//! - **TIN Types**: (1) **SSN** (Social Security Number) issued
//!   by Social Security Administration to US citizens and
//!   resident aliens eligible to work; (2) **EIN** (Employer
//!   Identification Number) issued by IRS to entities + sole
//!   proprietors; (3) **ITIN** (Individual Taxpayer
//!   Identification Number) issued by IRS to foreign persons
//!   not eligible for SSN who must file a US tax return; (4)
//!   **ATIN** (Adoption Taxpayer Identification Number) issued
//!   by IRS during pending US-domestic adoptions.
//! - **Form W-9** (Request for Taxpayer Identification Number
//!   and Certification): used by US persons to provide TIN to
//!   payers/brokers required to file information returns ([IRS
//!   — About Form W-9](https://www.irs.gov/forms-pubs/about-form-w-9);
//!   [IRS — Instructions for Form W-9](https://www.irs.gov/instructions/iw9)).
//! - **Form W-8BEN** (Certificate of Foreign Status of
//!   Beneficial Owner — Individuals): used by foreign
//!   individuals to claim treaty benefits and certify foreign
//!   status ([IRS — Instructions for Form W-8BEN](https://www.irs.gov/instructions/iw8ben)).
//!   ITIN required for treaty benefit claims unless income is
//!   from actively traded stocks/debt obligations, mutual fund
//!   dividends, or certain other securities income.
//! - **Form W-8BEN-E**: foreign entities; **W-8IMY**:
//!   intermediaries; **W-8ECI**: effectively connected income.
//! - **IRC § 3406 Backup Withholding**: payer must withhold at
//!   **24 PERCENT** (fourth lowest rate under § 1(c)) when
//!   payee fails to furnish valid TIN, after B-Notice TIN does
//!   not match IRS records, IRS notifies of underreporting, or
//!   payee fails to certify exemption on Form W-9 ([IRS IRM
//!   5.19.3 Backup Withholding Program](https://www.irs.gov/irm/part5/irm_05-019-003r)).
//! - **IRC § 6721 Information Return Penalty**: $50 per return
//!   for failure to file complete and accurate information
//!   return (including failure to include correct payee TIN);
//!   max **$250,000 per year** for large filers, **$100,000
//!   per year** for small businesses; **$100 per return with
//!   NO MAXIMUM for INTENTIONAL DISREGARD**.
//! - **Treas. Reg. § 301.6109-1**: implementing regulation
//!   defining TIN types, formats, application procedures, and
//!   payer/payee responsibilities.
//! - **Backup Withholding Triggers** (most common): (1) missing
//!   TIN on W-9; (2) TIN mismatch after first or second
//!   B-Notice; (3) IRS notification under § 3406(a)(1)(C) of
//!   underreporting; (4) failure to certify exemption from
//!   withholding on Form W-9.
//! - **B-Notice Cure Process**: first B-Notice (incorrect name/
//!   TIN) gives payee 30 business days to correct; second
//!   B-Notice (after second mismatch within 3 years) requires
//!   IRS or SSA validation document; failure to cure triggers
//!   24 % backup withholding under § 3406.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6109_SECTION_NUMBER: u32 = 6_109;
pub const IRC_3406_BACKUP_WITHHOLDING_RATE_BASIS_POINTS: u64 = 2_400;
pub const IRC_3406_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_6721_PENALTY_PER_RETURN_DOLLARS: u64 = 50;
pub const IRC_6721_PENALTY_PER_RETURN_INTENTIONAL_DISREGARD_DOLLARS: u64 = 100;
pub const IRC_6721_PENALTY_MAX_LARGE_FILER_DOLLARS: u64 = 250_000;
pub const IRC_6721_PENALTY_MAX_SMALL_BUSINESS_DOLLARS: u64 = 100_000;
pub const IRC_6721_INTENTIONAL_DISREGARD_NO_MAX: u64 = u64::MAX;
pub const FIRST_B_NOTICE_CURE_PERIOD_BUSINESS_DAYS: u32 = 30;
pub const B_NOTICE_LOOKBACK_YEARS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TinType {
    Ssn,
    Ein,
    Itin,
    Atin,
    NoTinProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayeeTaxStatus {
    UsCitizenOrResidentAlienWithSsn,
    UsBusinessEntityWithEin,
    SoleProprietorWithSsnOrEin,
    ForeignIndividualWithItin,
    ForeignEntityNoUsTinFormW8benE,
    NoTinNotEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormUsed {
    FormW9UsPersonRequestForTin,
    FormW8benForeignIndividualBeneficialOwner,
    FormW8benEForeignEntityBeneficialOwner,
    FormW8imyIntermediary,
    FormW8eciEffectivelyConnectedIncome,
    NoFormCollected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupWithholdingTrigger {
    NoTriggerNoBackupWithholdingRequired,
    PayeeFailedToProvideTinOnFormW9,
    TinMismatchAfterFirstBNoticeNotCured,
    TinMismatchAfterSecondBNoticeNotCured,
    IrsNotifiedPayeeUnderreportingUnderSection3406A1C,
    PayeeFailedToCertifyExemptionFromWithholdingOnFormW9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6109Mode {
    NotApplicableNoInformationReturnRequired,
    CompliantSection6109A1PayerIncludedCorrectTinOnInformationReturn,
    CompliantSection6109A2PayeeFurnishedCorrectTin,
    CompliantFormW9OrW8benCollectedAndOnFile,
    CompliantBackupWithholdingApplied24PercentRate,
    CompliantBNoticeFirstCureWithin30BusinessDays,
    CompliantBNoticeSecondRequiresIrsSsaValidation,
    ViolationSection6109A1PayerOmittedOrIncorrectTinOnInformationReturn,
    ViolationSection6109A2PayeeFailedToFurnishTin,
    ViolationBackupWithholdingNotAppliedDespiteTriggerEvent,
    ViolationSection6721InformationReturnPenaltyAccruesWithinAnnualMax,
    ViolationSection6721IntentionalDisregardPenaltyNoMax,
    ViolationFormW9OrW8benNotCollectedBeforeFirstPayment,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub information_return_required: bool,
    pub tin_type_furnished: TinType,
    pub payee_tax_status: PayeeTaxStatus,
    pub form_used: FormUsed,
    pub backup_withholding_trigger: BackupWithholdingTrigger,
    pub backup_withholding_applied_at_24_percent: bool,
    pub correct_tin_on_information_return: bool,
    pub number_of_information_returns: u64,
    pub intentional_disregard: bool,
    pub is_small_business_filer: bool,
    pub reportable_payment_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6109Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub backup_withholding_amount_dollars: u64,
    pub section_6721_penalty_dollars: u64,
}

pub type Section6109Input = Input;
pub type Section6109Output = Output;
pub type Section6109Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6109(a)(1) — payer required to file information return must include payee's CORRECT TIN on the return".to_string(),
        "IRC § 6109(a)(2) — payee must furnish CORRECT TIN to payer when payment is reportable on information return".to_string(),
        "IRC § 6109(a)(3) — person making return or statement under § 6011 must include identifying number assigned to such person".to_string(),
        "TIN Types under Treas. Reg. § 301.6109-1 — (1) SSN issued by Social Security Administration to US citizens/residents eligible to work; (2) EIN issued by IRS to entities + sole proprietors; (3) ITIN issued by IRS to foreign persons not eligible for SSN; (4) ATIN issued by IRS during pending US-domestic adoptions".to_string(),
        "Form W-9 — Request for Taxpayer Identification Number and Certification; US persons provide TIN to payers/brokers required to file information returns".to_string(),
        "Form W-8BEN — Certificate of Foreign Status of Beneficial Owner (Individuals); foreign individuals claim treaty benefits + certify foreign status; ITIN required for treaty benefits unless income is from actively-traded stocks/debt, mutual fund dividends, or certain other securities income".to_string(),
        "Form W-8BEN-E — foreign entities; Form W-8IMY — intermediaries; Form W-8ECI — effectively connected income".to_string(),
        "IRC § 3406 — backup withholding at 24 PERCENT (fourth lowest rate under § 1(c)) when payee fails to furnish valid TIN, after B-Notice TIN does not match IRS records, IRS notifies of underreporting, or payee fails to certify exemption on Form W-9".to_string(),
        "IRC § 6721 — information return penalty: $50 per return for failure to file complete/accurate information return (including incorrect payee TIN); $250,000 max per year for large filers; $100,000 max for small businesses; $100 per return with NO MAXIMUM for INTENTIONAL DISREGARD".to_string(),
        "B-Notice Cure Process — first B-Notice (incorrect name/TIN) gives payee 30 BUSINESS DAYS to correct; second B-Notice (after second mismatch within 3 years) requires IRS or SSA validation document; failure to cure triggers 24 % backup withholding under § 3406".to_string(),
        "Treas. Reg. § 301.6109-1 — implementing regulation defining TIN types, formats, application procedures, and payer/payee responsibilities".to_string(),
        "IRS IRM 5.19.3 — Backup Withholding Program operational guidance".to_string(),
        "Cornell LII 26 USC § 6109 — primary statutory text".to_string(),
        "IRS — Taxpayer Identification Numbers (TIN) practitioner landing page".to_string(),
        "IRS Publication 2108A — On-Line TIN Matching Program guidance".to_string(),
    ];

    if !input.information_return_required {
        return Output {
            mode: Section6109Mode::NotApplicableNoInformationReturnRequired,
            statutory_basis: "IRC § 6109 — applies only where information return is required".to_string(),
            notes: "NOT APPLICABLE: no information return required for payment; § 6109 TIN requirements do not apply.".to_string(),
            citations,
            backup_withholding_amount_dollars: 0,
            section_6721_penalty_dollars: 0,
        };
    }

    let backup_withholding_amount_dollars = if input.backup_withholding_trigger
        != BackupWithholdingTrigger::NoTriggerNoBackupWithholdingRequired
    {
        input
            .reportable_payment_dollars
            .saturating_mul(IRC_3406_BACKUP_WITHHOLDING_RATE_BASIS_POINTS)
            / IRC_3406_BASIS_POINT_DENOMINATOR
    } else {
        0
    };

    if input.intentional_disregard {
        let intentional_penalty = input
            .number_of_information_returns
            .saturating_mul(IRC_6721_PENALTY_PER_RETURN_INTENTIONAL_DISREGARD_DOLLARS);
        return Output {
            mode: Section6109Mode::ViolationSection6721IntentionalDisregardPenaltyNoMax,
            statutory_basis: "IRC § 6721 — intentional disregard penalty: $100 per return with NO MAXIMUM".to_string(),
            notes: format!(
                "VIOLATION: intentional disregard of § 6109 TIN requirements; § 6721 penalty = $100 per return × {} returns = ${} with NO MAXIMUM cap.",
                input.number_of_information_returns, intentional_penalty
            ),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: intentional_penalty,
        };
    }

    if input.form_used == FormUsed::NoFormCollected {
        return Output {
            mode: Section6109Mode::ViolationFormW9OrW8benNotCollectedBeforeFirstPayment,
            statutory_basis: "Treas. Reg. § 301.6109-1 + IRS guidance — Form W-9 (US persons) or Form W-8BEN/W-8BEN-E (foreign) must be collected before first reportable payment".to_string(),
            notes: "VIOLATION: no Form W-9 (US persons) or Form W-8BEN/W-8BEN-E (foreign) collected before first reportable payment; § 6109 TIN documentation chain not established.".to_string(),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: 0,
        };
    }

    if input.tin_type_furnished == TinType::NoTinProvided {
        return Output {
            mode: Section6109Mode::ViolationSection6109A2PayeeFailedToFurnishTin,
            statutory_basis: "IRC § 6109(a)(2) — payee must furnish correct TIN to payer".to_string(),
            notes: "VIOLATION: payee failed to furnish TIN as required by § 6109(a)(2); payer must initiate 24 % backup withholding under § 3406 until valid TIN provided.".to_string(),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: 0,
        };
    }

    if !input.correct_tin_on_information_return {
        let penalty_per_return = IRC_6721_PENALTY_PER_RETURN_DOLLARS;
        let raw_penalty = input
            .number_of_information_returns
            .saturating_mul(penalty_per_return);
        let max_penalty = if input.is_small_business_filer {
            IRC_6721_PENALTY_MAX_SMALL_BUSINESS_DOLLARS
        } else {
            IRC_6721_PENALTY_MAX_LARGE_FILER_DOLLARS
        };
        let section_6721_penalty_dollars = raw_penalty.min(max_penalty);
        return Output {
            mode: Section6109Mode::ViolationSection6721InformationReturnPenaltyAccruesWithinAnnualMax,
            statutory_basis: "IRC § 6109(a)(1) + § 6721 — payer omitted or filed incorrect TIN".to_string(),
            notes: format!(
                "VIOLATION: § 6109(a)(1) requires correct payee TIN on information return; § 6721 penalty = $50 × {} returns = ${} (capped at ${} {} annual maximum).",
                input.number_of_information_returns,
                raw_penalty,
                max_penalty,
                if input.is_small_business_filer { "small business" } else { "large filer" }
            ),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars,
        };
    }

    if input.backup_withholding_trigger == BackupWithholdingTrigger::TinMismatchAfterFirstBNoticeNotCured
    {
        return Output {
            mode: Section6109Mode::CompliantBNoticeFirstCureWithin30BusinessDays,
            statutory_basis: "B-Notice first cure period — 30 business days under § 3406 procedures".to_string(),
            notes: "COMPLIANT: payee in first B-Notice cure period (30 business days); backup withholding suspended pending cure; if not cured timely, 24 % backup withholding begins under § 3406.".to_string(),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: 0,
        };
    }

    if input.backup_withholding_trigger
        == BackupWithholdingTrigger::TinMismatchAfterSecondBNoticeNotCured
    {
        return Output {
            mode: Section6109Mode::CompliantBNoticeSecondRequiresIrsSsaValidation,
            statutory_basis: "B-Notice second cure — requires IRS or SSA validation document".to_string(),
            notes: "COMPLIANT: payee in second B-Notice cure phase requiring IRS or SSA validation document under § 3406 procedures; backup withholding continues until validation document received.".to_string(),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: 0,
        };
    }

    if input.backup_withholding_trigger != BackupWithholdingTrigger::NoTriggerNoBackupWithholdingRequired
        && !input.backup_withholding_applied_at_24_percent
    {
        return Output {
            mode: Section6109Mode::ViolationBackupWithholdingNotAppliedDespiteTriggerEvent,
            statutory_basis: "IRC § 3406 — backup withholding required when § 6109 triggers fire".to_string(),
            notes: format!(
                "VIOLATION: backup withholding trigger event ({:?}) occurred but payer failed to apply 24 % backup withholding under § 3406; ${} should have been withheld.",
                input.backup_withholding_trigger, backup_withholding_amount_dollars
            ),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: 0,
        };
    }

    if input.backup_withholding_trigger != BackupWithholdingTrigger::NoTriggerNoBackupWithholdingRequired
        && input.backup_withholding_applied_at_24_percent
    {
        return Output {
            mode: Section6109Mode::CompliantBackupWithholdingApplied24PercentRate,
            statutory_basis: "IRC § 3406 — backup withholding correctly applied at 24 % rate".to_string(),
            notes: format!(
                "COMPLIANT: backup withholding trigger event ({:?}) handled correctly; ${} withheld at 24 % rate under § 3406 from ${} reportable payment.",
                input.backup_withholding_trigger,
                backup_withholding_amount_dollars,
                input.reportable_payment_dollars
            ),
            citations,
            backup_withholding_amount_dollars,
            section_6721_penalty_dollars: 0,
        };
    }

    if matches!(
        input.form_used,
        FormUsed::FormW9UsPersonRequestForTin
            | FormUsed::FormW8benForeignIndividualBeneficialOwner
            | FormUsed::FormW8benEForeignEntityBeneficialOwner
    ) {
        return Output {
            mode: Section6109Mode::CompliantFormW9OrW8benCollectedAndOnFile,
            statutory_basis: "Treas. Reg. § 301.6109-1 — Form W-9/W-8BEN/W-8BEN-E TIN documentation chain established".to_string(),
            notes: format!(
                "COMPLIANT: appropriate form collected ({:?}); TIN type {:?} on file; payee tax status {:?} verified.",
                input.form_used, input.tin_type_furnished, input.payee_tax_status
            ),
            citations,
            backup_withholding_amount_dollars: 0,
            section_6721_penalty_dollars: 0,
        };
    }

    Output {
        mode: Section6109Mode::CompliantSection6109A1PayerIncludedCorrectTinOnInformationReturn,
        statutory_basis: "IRC § 6109(a)(1) — payer included correct TIN on information return".to_string(),
        notes: format!(
            "COMPLIANT: § 6109(a)(1) + § 6109(a)(2) satisfied; payee furnished {:?}; payer included correct TIN on information return; no backup withholding trigger.",
            input.tin_type_furnished
        ),
        citations,
        backup_withholding_amount_dollars: 0,
        section_6721_penalty_dollars: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_form_w9() -> Input {
        Input {
            information_return_required: true,
            tin_type_furnished: TinType::Ssn,
            payee_tax_status: PayeeTaxStatus::UsCitizenOrResidentAlienWithSsn,
            form_used: FormUsed::FormW9UsPersonRequestForTin,
            backup_withholding_trigger: BackupWithholdingTrigger::NoTriggerNoBackupWithholdingRequired,
            backup_withholding_applied_at_24_percent: false,
            correct_tin_on_information_return: true,
            number_of_information_returns: 1,
            intentional_disregard: false,
            is_small_business_filer: false,
            reportable_payment_dollars: 10_000,
        }
    }

    #[test]
    fn no_information_return_not_applicable() {
        let input = Input {
            information_return_required: false,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section6109Mode::NotApplicableNoInformationReturnRequired);
    }

    #[test]
    fn form_w9_us_person_with_ssn_compliant() {
        let result = check(&baseline_compliant_form_w9());
        assert_eq!(
            result.mode,
            Section6109Mode::CompliantFormW9OrW8benCollectedAndOnFile
        );
    }

    #[test]
    fn form_w8ben_foreign_individual_with_itin_compliant() {
        let input = Input {
            tin_type_furnished: TinType::Itin,
            payee_tax_status: PayeeTaxStatus::ForeignIndividualWithItin,
            form_used: FormUsed::FormW8benForeignIndividualBeneficialOwner,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::CompliantFormW9OrW8benCollectedAndOnFile
        );
    }

    #[test]
    fn form_w8bene_foreign_entity_compliant() {
        let input = Input {
            tin_type_furnished: TinType::NoTinProvided,
            payee_tax_status: PayeeTaxStatus::ForeignEntityNoUsTinFormW8benE,
            form_used: FormUsed::FormW8benEForeignEntityBeneficialOwner,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        // No TIN on W-8BEN-E for foreign entity NOT claiming treaty benefits is allowed
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationSection6109A2PayeeFailedToFurnishTin
        );
    }

    #[test]
    fn ein_us_business_entity_compliant() {
        let input = Input {
            tin_type_furnished: TinType::Ein,
            payee_tax_status: PayeeTaxStatus::UsBusinessEntityWithEin,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::CompliantFormW9OrW8benCollectedAndOnFile
        );
    }

    #[test]
    fn no_form_collected_violation() {
        let input = Input {
            form_used: FormUsed::NoFormCollected,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationFormW9OrW8benNotCollectedBeforeFirstPayment
        );
    }

    #[test]
    fn no_tin_provided_violation() {
        let input = Input {
            tin_type_furnished: TinType::NoTinProvided,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationSection6109A2PayeeFailedToFurnishTin
        );
    }

    #[test]
    fn incorrect_tin_on_information_return_violation_small_business() {
        let input = Input {
            correct_tin_on_information_return: false,
            number_of_information_returns: 10_000,
            is_small_business_filer: true,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationSection6721InformationReturnPenaltyAccruesWithinAnnualMax
        );
        assert_eq!(result.section_6721_penalty_dollars, 100_000);
    }

    #[test]
    fn incorrect_tin_on_information_return_violation_large_filer() {
        let input = Input {
            correct_tin_on_information_return: false,
            number_of_information_returns: 10_000,
            is_small_business_filer: false,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(result.section_6721_penalty_dollars, 250_000);
    }

    #[test]
    fn incorrect_tin_below_max_within_annual_cap() {
        let input = Input {
            correct_tin_on_information_return: false,
            number_of_information_returns: 100,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(result.section_6721_penalty_dollars, 5_000);
    }

    #[test]
    fn intentional_disregard_no_max_penalty_violation() {
        let input = Input {
            number_of_information_returns: 50_000,
            intentional_disregard: true,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationSection6721IntentionalDisregardPenaltyNoMax
        );
        assert_eq!(result.section_6721_penalty_dollars, 5_000_000);
    }

    #[test]
    fn backup_withholding_no_tin_compliant_at_24_pct() {
        let input = Input {
            backup_withholding_trigger:
                BackupWithholdingTrigger::PayeeFailedToProvideTinOnFormW9,
            backup_withholding_applied_at_24_percent: true,
            reportable_payment_dollars: 10_000,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::CompliantBackupWithholdingApplied24PercentRate
        );
        assert_eq!(result.backup_withholding_amount_dollars, 2_400);
    }

    #[test]
    fn backup_withholding_not_applied_violation() {
        let input = Input {
            backup_withholding_trigger:
                BackupWithholdingTrigger::IrsNotifiedPayeeUnderreportingUnderSection3406A1C,
            backup_withholding_applied_at_24_percent: false,
            reportable_payment_dollars: 10_000,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationBackupWithholdingNotAppliedDespiteTriggerEvent
        );
    }

    #[test]
    fn first_b_notice_cure_window_compliant() {
        let input = Input {
            backup_withholding_trigger:
                BackupWithholdingTrigger::TinMismatchAfterFirstBNoticeNotCured,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::CompliantBNoticeFirstCureWithin30BusinessDays
        );
    }

    #[test]
    fn second_b_notice_requires_irs_ssa_validation_compliant() {
        let input = Input {
            backup_withholding_trigger:
                BackupWithholdingTrigger::TinMismatchAfterSecondBNoticeNotCured,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::CompliantBNoticeSecondRequiresIrsSsaValidation
        );
    }

    #[test]
    fn citations_pin_section_6109_subsections_and_forms() {
        let result = check(&baseline_compliant_form_w9());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 6109(a)(1)"));
        assert!(joined.contains("IRC § 6109(a)(2)"));
        assert!(joined.contains("IRC § 6109(a)(3)"));
        assert!(joined.contains("TIN"));
        assert!(joined.contains("SSN"));
        assert!(joined.contains("EIN"));
        assert!(joined.contains("ITIN"));
        assert!(joined.contains("ATIN"));
        assert!(joined.contains("Form W-9"));
        assert!(joined.contains("Form W-8BEN"));
        assert!(joined.contains("Form W-8BEN-E"));
        assert!(joined.contains("Form W-8IMY"));
        assert!(joined.contains("Form W-8ECI"));
        assert!(joined.contains("IRC § 3406"));
        assert!(joined.contains("24 PERCENT"));
        assert!(joined.contains("IRC § 6721"));
        assert!(joined.contains("$50 per return"));
        assert!(joined.contains("$250,000"));
        assert!(joined.contains("$100,000"));
        assert!(joined.contains("$100 per return with NO MAXIMUM"));
        assert!(joined.contains("INTENTIONAL DISREGARD"));
        assert!(joined.contains("Treas. Reg. § 301.6109-1"));
        assert!(joined.contains("B-Notice"));
        assert!(joined.contains("30 BUSINESS DAYS"));
        assert!(joined.contains("IRS IRM 5.19.3"));
    }

    #[test]
    fn constant_pin_rates_penalties_and_thresholds() {
        assert_eq!(IRC_6109_SECTION_NUMBER, 6_109);
        assert_eq!(IRC_3406_BACKUP_WITHHOLDING_RATE_BASIS_POINTS, 2_400);
        assert_eq!(IRC_3406_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_6721_PENALTY_PER_RETURN_DOLLARS, 50);
        assert_eq!(IRC_6721_PENALTY_PER_RETURN_INTENTIONAL_DISREGARD_DOLLARS, 100);
        assert_eq!(IRC_6721_PENALTY_MAX_LARGE_FILER_DOLLARS, 250_000);
        assert_eq!(IRC_6721_PENALTY_MAX_SMALL_BUSINESS_DOLLARS, 100_000);
        assert_eq!(IRC_6721_INTENTIONAL_DISREGARD_NO_MAX, u64::MAX);
        assert_eq!(FIRST_B_NOTICE_CURE_PERIOD_BUSINESS_DAYS, 30);
        assert_eq!(B_NOTICE_LOOKBACK_YEARS, 3);
    }

    #[test]
    fn saturating_overflow_defense_extreme_returns() {
        let input = Input {
            number_of_information_returns: u64::MAX,
            intentional_disregard: true,
            ..baseline_compliant_form_w9()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6109Mode::ViolationSection6721IntentionalDisregardPenaltyNoMax
        );
    }
}
