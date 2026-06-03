//! IRC § 6041 — Information at Source / Information Reporting.
//!
//! Pure-compute check for whether a person engaged in a trade or
//! business must file Form 1099-NEC / 1099-MISC / 1042-S for
//! payments made to a payee, and whether the filing requirements
//! and backup withholding obligations were satisfied.
//!
//! Statute (verbatim mapping):
//! - § 6041(a) — GENERAL RULE: all persons engaged in a trade or
//!   business and making payment in the course of such trade or
//!   business to another person of rent, salaries, wages, premiums,
//!   annuities, compensations, remunerations, emoluments, or other
//!   fixed or determinable gains, profits, and income (other than
//!   payments to which § 6042(a)(1), 6044(a)(1), 6047(e), 6049(a),
//!   or 6050N(a) applies, and other than payments with respect to
//!   which a statement is required under § 6042(a), 6044(b), or
//!   6050N(a)) of $2,000 (post-OBBBA; previously $600) or more in
//!   any taxable year shall render a true and accurate return of
//!   payment to such other person.
//! - § 6041(a) — POST-OBBBA THRESHOLD: P.L. 119-21 (One Big
//!   Beautiful Bill Act of 2025) § 70433 amended § 6041(a) and
//!   § 6041A(a)(2) to RAISE the reporting threshold from $600 to
//!   $2,000 for payments made AFTER December 31, 2025. Effective
//!   for 2026 tax year (1099s filed in early 2027). Threshold
//!   indexed annually for inflation from 2027 with 2025 as base
//!   year, rounded to nearest $100.
//! - § 6041(c) — CORPORATE EXCEPTION: most payments to
//!   corporations are EXEMPT from § 6041 reporting; EXCEPTIONS
//!   include payments to attorneys and corporate medical/health
//!   payments, which must be reported regardless of corporate
//!   status of payee.
//! - § 6041(h) — TIN REQUIREMENT: payee must furnish taxpayer
//!   identification number on Form W-9; failure triggers § 3406
//!   backup withholding at 24 percent.
//! - § 6041A — DIRECT SALES OF $5,000 OR MORE + REMUNERATION FOR
//!   NONEMPLOYEE SERVICES: parallel reporting regime for direct
//!   sales and nonemployee services subject to same OBBBA $2,000
//!   threshold post-2025.
//! - § 3406 — BACKUP WITHHOLDING: 24 percent withholding on
//!   reportable payments when payee fails to provide TIN or IRS
//!   notifies payer of TIN mismatch.
//! - § 6721 — FAILURE TO FILE CORRECT INFORMATION RETURN: penalty
//!   per return varying by lateness (e.g., $60 if corrected ≤ 30
//!   days, $130 if ≤ Aug 1, $330 if after Aug 1; intentional
//!   disregard $660+).
//! - § 6722 — FAILURE TO FURNISH CORRECT PAYEE STATEMENT: parallel
//!   penalty structure on payee statements.
//! - § 6723 — FAILURE TO COMPLY WITH OTHER INFORMATION REPORTING
//!   REQUIREMENTS: $50 per failure.
//!
//! Forms:
//! - **Form 1099-NEC** — Non-Employee Compensation (services
//!   rendered by independent contractors, freelancers, etc.); due
//!   January 31 of year following payment.
//! - **Form 1099-MISC** — Miscellaneous Income (rents, prizes,
//!   awards, other income); due February 28 paper / March 31
//!   electronic.
//! - **Form 1042 + Form 1042-S** — foreign person reporting under
//!   FATCA Chapter 4 + Chapter 3.
//! - **Form W-9** — Request for Taxpayer Identification Number
//!   and Certification (US persons).
//! - **Form W-8BEN / W-8BEN-E** — foreign person beneficial owner
//!   certification.
//!
//! Web research (verified 2026-06-03):
//! - IRS Publication 1099 (2026) General Instructions confirms
//!   OBBBA $2,000 threshold effective for 2026 tax year.
//! - Federal Register 2026-07519 "Increase in Threshold for
//!   Requiring Information Reporting" confirms statutory change.
//! - RSM US "Navigating OBBBA's new changes in U.S. tax reporting
//!   and withholding rules" confirms § 70433 amendments to
//!   § 6041(a) and § 6041A(a)(2).
//! - WhippleWood "$2,000 1099 Threshold for 2026" confirms
//!   inflation indexing from 2027.
//! - Avalara "One Big Beautiful Bill Act changes 1099 thresholds"
//!   confirms effective date Dec 31, 2025.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_6041_OBBBA_POST_2025_THRESHOLD_DOLLARS: u64 = 2_000;
pub const SECTION_6041_PRE_OBBBA_THRESHOLD_DOLLARS: u64 = 600;
pub const SECTION_6041_OBBBA_EFFECTIVE_AFTER_YEAR: u32 = 2025;
pub const SECTION_6041_OBBBA_EFFECTIVE_AFTER_MONTH: u32 = 12;
pub const SECTION_6041_OBBBA_EFFECTIVE_AFTER_DAY: u32 = 31;
pub const SECTION_6041_INFLATION_INDEXING_BASE_YEAR: u32 = 2025;
pub const SECTION_6041_INFLATION_ROUNDING_DOLLARS: u64 = 100;
pub const SECTION_3406_BACKUP_WITHHOLDING_BASIS_POINTS: u64 = 2_400;
pub const SECTION_6041_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_6041_OBBBA_PUBLIC_LAW_NUMBER_MAJOR: u32 = 119;
pub const SECTION_6041_OBBBA_PUBLIC_LAW_NUMBER_MINOR: u32 = 21;
pub const SECTION_6041_OBBBA_SECTION_NUMBER: u32 = 70_433;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayerActivityType {
    EngagedInTradeOrBusiness,
    PersonalConsumerNotTradeOrBusiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayeeClassification {
    IndividualSoleProprietorContractor,
    CorporationGeneralExempt,
    CorporationAttorneyLawFirm,
    CorporationMedicalHealthcareProvider,
    PartnershipOrLlcWithMultipleMembers,
    ForeignPersonW8Provided,
    ForeignPersonNoDocumentation,
    UsTrustOrEstate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    NonemployeeCompensationServices,
    RentMisc1099,
    AttorneyFeesLegal,
    PrizesAwards,
    MedicalHealthcarePayments,
    DirectSalesOf5000OrMore,
    OtherTradeOrBusinessPayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentYearOBBBAStatus {
    PaymentMadeBeforeJanuary1_2026OldThreshold,
    PaymentMadeOnOrAfterJanuary1_2026NewThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6041Mode {
    NotApplicableNotTradeOrBusiness,
    NotApplicableBelowReportingThreshold,
    NotApplicableCorporationGeneralExemption,
    NotApplicableForeignPayeeWithW8DocumentationFatcaApplies,
    CompliantForm1099NecFiledTimelyJanuary31,
    CompliantForm1099MiscFiledTimely,
    CompliantForm1042SFiledForForeignPayeeNoDocumentation,
    CompliantBackupWithholdingAppliedNoW9,
    CompliantAttorneyOrMedicalPaymentReportedDespiteCorpStatus,
    ViolationFailedToFileForm1099Section6721,
    ViolationFailedToFurnishPayeeStatementSection6722,
    ViolationFailedToObtainW9OrApplyBackupWithholding,
    ViolationLateFiledMoreThan30DaysAfterDueDate,
    ViolationLawyerPaymentExceptionMisappliedAsCorpExempt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub payer_activity_type: PayerActivityType,
    pub payee_classification: PayeeClassification,
    pub payment_type: PaymentType,
    pub payment_amount_dollars: u64,
    pub payment_year_obbba_status: PaymentYearOBBBAStatus,
    pub payee_provided_w9_with_tin: bool,
    pub payer_actually_applied_backup_withholding: bool,
    pub actual_backup_withholding_dollars: u64,
    pub form_1099_filed_by_january_31_deadline: bool,
    pub form_1099_or_1042s_furnished_to_payee: bool,
    pub days_late_filing_form_1099: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6041Mode,
    pub applicable_threshold_dollars: u64,
    pub required_backup_withholding_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section6041Input = Input;
pub type Section6041Output = Output;
pub type Section6041Result = Output;

fn applicable_threshold(status: PaymentYearOBBBAStatus) -> u64 {
    match status {
        PaymentYearOBBBAStatus::PaymentMadeBeforeJanuary1_2026OldThreshold => {
            SECTION_6041_PRE_OBBBA_THRESHOLD_DOLLARS
        }
        PaymentYearOBBBAStatus::PaymentMadeOnOrAfterJanuary1_2026NewThreshold => {
            SECTION_6041_OBBBA_POST_2025_THRESHOLD_DOLLARS
        }
    }
}

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_6041_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 6041(a) — trade-or-business persons making payments of $2,000 or more (post-OBBBA) to another person must file information return".to_string(),
        "26 U.S.C. § 6041(c) — corporate exception: most payments to corporations exempt; EXCEPTIONS include attorneys and medical/health payments".to_string(),
        "26 U.S.C. § 6041(h) — payee TIN requirement; W-9 collected before payment".to_string(),
        "26 U.S.C. § 6041A — direct sales of $5,000+ + nonemployee remuneration subject to same OBBBA $2,000 threshold".to_string(),
        "26 U.S.C. § 3406 — backup withholding at 24 % when payee fails to provide TIN or IRS notifies of TIN mismatch".to_string(),
        "26 U.S.C. § 6721 — failure to file correct information return penalty ($60 ≤ 30 days late, $130 ≤ Aug 1, $330 after Aug 1; intentional disregard $660+)".to_string(),
        "26 U.S.C. § 6722 — failure to furnish correct payee statement; parallel penalty structure".to_string(),
        "26 U.S.C. § 6723 — failure to comply with other information reporting requirements; $50 per failure".to_string(),
        "P.L. 119-21 (One Big Beautiful Bill Act of 2025) § 70433 — raised § 6041(a) + § 6041A(a)(2) threshold from $600 to $2,000 effective Jan 1, 2026; inflation indexed from 2027 with 2025 base, rounded to nearest $100".to_string(),
        "Form 1099-NEC — Non-Employee Compensation; due January 31 of year following payment".to_string(),
        "Form 1099-MISC — Miscellaneous Income (rents, prizes, awards); due February 28 paper / March 31 electronic".to_string(),
        "Form W-9 — Request for Taxpayer Identification Number and Certification (US persons)".to_string(),
        "Form W-8BEN / W-8BEN-E — foreign person beneficial owner certification".to_string(),
        "Form 1042 + Form 1042-S — foreign person reporting under FATCA Chapter 4 + Chapter 3".to_string(),
    ];

    if input.payer_activity_type == PayerActivityType::PersonalConsumerNotTradeOrBusiness {
        return Output {
            mode: Section6041Mode::NotApplicableNotTradeOrBusiness,
            applicable_threshold_dollars: 0,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6041(a) — payer not engaged in trade or business".to_string(),
            notes: "Payer is consumer not engaged in trade or business; § 6041 reporting inapplicable.".to_string(),
            citations,
        };
    }

    let threshold = applicable_threshold(input.payment_year_obbba_status);

    if input.payment_amount_dollars < threshold {
        return Output {
            mode: Section6041Mode::NotApplicableBelowReportingThreshold,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: format!("§ 6041(a) — payment below ${} threshold", threshold),
            notes: format!(
                "Payment of ${} is below applicable reporting threshold of ${}; § 6041 information return not required.",
                input.payment_amount_dollars, threshold
            ),
            citations,
        };
    }

    if input.payee_classification == PayeeClassification::ForeignPersonW8Provided
        && input.form_1099_or_1042s_furnished_to_payee
    {
        return Output {
            mode: Section6041Mode::NotApplicableForeignPayeeWithW8DocumentationFatcaApplies,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6041 inapplicable to foreign payee with W-8 documentation; FATCA Chapter 3/4 applies via Form 1042-S".to_string(),
            notes: "Foreign payee with W-8BEN / W-8BEN-E documentation; § 6041 inapplicable. Reporting on Form 1042-S under FATCA Chapter 3 / 4 ( § 1441 / § 1471 ).".to_string(),
            citations,
        };
    }

    if input.payee_classification == PayeeClassification::ForeignPersonNoDocumentation {
        if !input.form_1099_or_1042s_furnished_to_payee {
            return Output {
                mode: Section6041Mode::ViolationFailedToFurnishPayeeStatementSection6722,
                applicable_threshold_dollars: threshold,
                required_backup_withholding_dollars: 0,
                statutory_basis: "§ 6722 — failure to furnish Form 1042-S to undocumented foreign payee".to_string(),
                notes: "VIOLATION: foreign payee provided no W-8 documentation; payer must withhold 30 % under FATCA / Chapter 3 and issue Form 1042-S. Failure to furnish payee statement subject to § 6722 penalty.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section6041Mode::CompliantForm1042SFiledForForeignPayeeNoDocumentation,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6041 + § 1441/1471 — foreign payee reporting on Form 1042-S".to_string(),
            notes: "COMPLIANT: Form 1042-S filed for foreign payee with no W-8 documentation; 30 % FATCA/Chapter 3 withholding applied.".to_string(),
            citations,
        };
    }

    if input.payee_classification == PayeeClassification::CorporationGeneralExempt {
        if matches!(input.payment_type, PaymentType::AttorneyFeesLegal | PaymentType::MedicalHealthcarePayments) {
            return Output {
                mode: Section6041Mode::ViolationLawyerPaymentExceptionMisappliedAsCorpExempt,
                applicable_threshold_dollars: threshold,
                required_backup_withholding_dollars: 0,
                statutory_basis: "§ 6041(c) — attorney and medical payments NOT exempt despite corporate payee status".to_string(),
                notes: format!(
                    "VIOLATION: payee is corporation but payment type {:?} is NOT exempt under § 6041(c) corporate exception. Form 1099 must be filed.",
                    input.payment_type
                ),
                citations,
            };
        }
        return Output {
            mode: Section6041Mode::NotApplicableCorporationGeneralExemption,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6041(c) — corporate exemption applies".to_string(),
            notes: format!(
                "Payee is corporation and payment type {:?} qualifies for § 6041(c) general corporate exemption from information reporting.",
                input.payment_type
            ),
            citations,
        };
    }

    if !input.payee_provided_w9_with_tin {
        let required_bw = apply_rate(input.payment_amount_dollars, SECTION_3406_BACKUP_WITHHOLDING_BASIS_POINTS);
        if !input.payer_actually_applied_backup_withholding
            || input.actual_backup_withholding_dollars < required_bw
        {
            return Output {
                mode: Section6041Mode::ViolationFailedToObtainW9OrApplyBackupWithholding,
                applicable_threshold_dollars: threshold,
                required_backup_withholding_dollars: required_bw,
                statutory_basis: "§ 3406 — backup withholding required at 24 % when payee fails to provide TIN".to_string(),
                notes: format!(
                    "VIOLATION § 3406: payee did not provide W-9 with TIN; required 24 % backup withholding on payment ${} = ${}; payer actual withholding ${}.",
                    input.payment_amount_dollars, required_bw, input.actual_backup_withholding_dollars
                ),
                citations,
            };
        }
        return Output {
            mode: Section6041Mode::CompliantBackupWithholdingAppliedNoW9,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: required_bw,
            statutory_basis: "§ 3406 — 24 % backup withholding properly applied".to_string(),
            notes: format!(
                "COMPLIANT § 3406: payee did not provide W-9; payer applied 24 % backup withholding of ${} on payment ${}.",
                required_bw, input.payment_amount_dollars
            ),
            citations,
        };
    }

    if !input.form_1099_filed_by_january_31_deadline {
        let mode = if input.days_late_filing_form_1099 > 30 {
            Section6041Mode::ViolationLateFiledMoreThan30DaysAfterDueDate
        } else {
            Section6041Mode::ViolationFailedToFileForm1099Section6721
        };
        return Output {
            mode,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6721 — failure to file correct information return".to_string(),
            notes: format!(
                "VIOLATION § 6721: Form 1099 not filed by January 31 deadline; {} days late. Penalty per return varies by lateness ($60 ≤ 30 days, $130 ≤ Aug 1, $330 after Aug 1; intentional disregard $660+).",
                input.days_late_filing_form_1099
            ),
            citations,
        };
    }

    if !input.form_1099_or_1042s_furnished_to_payee {
        return Output {
            mode: Section6041Mode::ViolationFailedToFurnishPayeeStatementSection6722,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6722 — failure to furnish correct payee statement".to_string(),
            notes: "VIOLATION § 6722: Form 1099 filed with IRS but not furnished to payee; payee statement required.".to_string(),
            citations,
        };
    }

    if matches!(input.payment_type, PaymentType::AttorneyFeesLegal | PaymentType::MedicalHealthcarePayments)
        && matches!(
            input.payee_classification,
            PayeeClassification::CorporationAttorneyLawFirm | PayeeClassification::CorporationMedicalHealthcareProvider
        )
    {
        return Output {
            mode: Section6041Mode::CompliantAttorneyOrMedicalPaymentReportedDespiteCorpStatus,
            applicable_threshold_dollars: threshold,
            required_backup_withholding_dollars: 0,
            statutory_basis: "§ 6041(c) — attorney + medical payments reported despite corporate status".to_string(),
            notes: format!(
                "COMPLIANT: payee is corporate {:?} but § 6041(c) requires reporting on Form 1099 despite corporate status. Form 1099 filed and furnished.",
                input.payee_classification
            ),
            citations,
        };
    }

    let mode = match input.payment_type {
        PaymentType::NonemployeeCompensationServices => Section6041Mode::CompliantForm1099NecFiledTimelyJanuary31,
        _ => Section6041Mode::CompliantForm1099MiscFiledTimely,
    };

    Output {
        mode,
        applicable_threshold_dollars: threshold,
        required_backup_withholding_dollars: 0,
        statutory_basis: format!(
            "§ 6041(a) reporting threshold ${} satisfied; Form 1099 filed and furnished timely",
            threshold
        ),
        notes: format!(
            "COMPLIANT § 6041: payment of ${} to {:?} reported on Form 1099 (filed January 31 + furnished to payee).",
            input.payment_amount_dollars, input.payee_classification
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_2026_compliant() -> Input {
        Input {
            payer_activity_type: PayerActivityType::EngagedInTradeOrBusiness,
            payee_classification: PayeeClassification::IndividualSoleProprietorContractor,
            payment_type: PaymentType::NonemployeeCompensationServices,
            payment_amount_dollars: 5_000,
            payment_year_obbba_status: PaymentYearOBBBAStatus::PaymentMadeOnOrAfterJanuary1_2026NewThreshold,
            payee_provided_w9_with_tin: true,
            payer_actually_applied_backup_withholding: false,
            actual_backup_withholding_dollars: 0,
            form_1099_filed_by_january_31_deadline: true,
            form_1099_or_1042s_furnished_to_payee: true,
            days_late_filing_form_1099: 0,
        }
    }

    #[test]
    fn personal_consumer_not_applicable() {
        let input = Input {
            payer_activity_type: PayerActivityType::PersonalConsumerNotTradeOrBusiness,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::NotApplicableNotTradeOrBusiness);
    }

    #[test]
    fn pre_2026_payment_uses_600_threshold() {
        let input = Input {
            payment_year_obbba_status: PaymentYearOBBBAStatus::PaymentMadeBeforeJanuary1_2026OldThreshold,
            payment_amount_dollars: 700,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantForm1099NecFiledTimelyJanuary31);
        assert_eq!(result.applicable_threshold_dollars, 600);
    }

    #[test]
    fn post_2025_payment_uses_2000_threshold() {
        let result = compute(&baseline_2026_compliant());
        assert_eq!(result.mode, Section6041Mode::CompliantForm1099NecFiledTimelyJanuary31);
        assert_eq!(result.applicable_threshold_dollars, 2_000);
    }

    #[test]
    fn post_2025_payment_at_1999_below_threshold() {
        let input = Input {
            payment_amount_dollars: 1_999,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::NotApplicableBelowReportingThreshold);
    }

    #[test]
    fn post_2025_payment_at_exactly_2000_satisfies_threshold() {
        let input = Input {
            payment_amount_dollars: 2_000,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantForm1099NecFiledTimelyJanuary31);
    }

    #[test]
    fn corporate_general_exemption_applies() {
        let input = Input {
            payee_classification: PayeeClassification::CorporationGeneralExempt,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::NotApplicableCorporationGeneralExemption);
    }

    #[test]
    fn attorney_payment_to_corp_still_reportable_compliant() {
        let input = Input {
            payee_classification: PayeeClassification::CorporationAttorneyLawFirm,
            payment_type: PaymentType::AttorneyFeesLegal,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantAttorneyOrMedicalPaymentReportedDespiteCorpStatus);
    }

    #[test]
    fn corp_general_exempt_misapplied_for_attorney_payment_violation() {
        let input = Input {
            payee_classification: PayeeClassification::CorporationGeneralExempt,
            payment_type: PaymentType::AttorneyFeesLegal,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationLawyerPaymentExceptionMisappliedAsCorpExempt);
    }

    #[test]
    fn medical_payment_to_corp_still_reportable_compliant() {
        let input = Input {
            payee_classification: PayeeClassification::CorporationMedicalHealthcareProvider,
            payment_type: PaymentType::MedicalHealthcarePayments,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantAttorneyOrMedicalPaymentReportedDespiteCorpStatus);
    }

    #[test]
    fn no_w9_with_backup_withholding_compliant() {
        let input = Input {
            payee_provided_w9_with_tin: false,
            payer_actually_applied_backup_withholding: true,
            actual_backup_withholding_dollars: 1_200,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantBackupWithholdingAppliedNoW9);
        assert_eq!(result.required_backup_withholding_dollars, 1_200);
    }

    #[test]
    fn no_w9_no_backup_withholding_violation() {
        let input = Input {
            payee_provided_w9_with_tin: false,
            payer_actually_applied_backup_withholding: false,
            actual_backup_withholding_dollars: 0,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationFailedToObtainW9OrApplyBackupWithholding);
    }

    #[test]
    fn no_w9_insufficient_backup_withholding_violation() {
        let input = Input {
            payee_provided_w9_with_tin: false,
            payer_actually_applied_backup_withholding: true,
            actual_backup_withholding_dollars: 500,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationFailedToObtainW9OrApplyBackupWithholding);
    }

    #[test]
    fn form_1099_late_within_30_days_violation() {
        let input = Input {
            form_1099_filed_by_january_31_deadline: false,
            days_late_filing_form_1099: 20,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationFailedToFileForm1099Section6721);
    }

    #[test]
    fn form_1099_late_more_than_30_days_violation() {
        let input = Input {
            form_1099_filed_by_january_31_deadline: false,
            days_late_filing_form_1099: 45,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationLateFiledMoreThan30DaysAfterDueDate);
    }

    #[test]
    fn payee_statement_not_furnished_violation() {
        let input = Input {
            form_1099_or_1042s_furnished_to_payee: false,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationFailedToFurnishPayeeStatementSection6722);
    }

    #[test]
    fn foreign_w8_with_1042s_furnished_not_applicable() {
        let input = Input {
            payee_classification: PayeeClassification::ForeignPersonW8Provided,
            form_1099_or_1042s_furnished_to_payee: true,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::NotApplicableForeignPayeeWithW8DocumentationFatcaApplies);
    }

    #[test]
    fn foreign_undocumented_with_1042s_compliant() {
        let input = Input {
            payee_classification: PayeeClassification::ForeignPersonNoDocumentation,
            form_1099_or_1042s_furnished_to_payee: true,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantForm1042SFiledForForeignPayeeNoDocumentation);
    }

    #[test]
    fn foreign_undocumented_no_1042s_violation() {
        let input = Input {
            payee_classification: PayeeClassification::ForeignPersonNoDocumentation,
            form_1099_or_1042s_furnished_to_payee: false,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::ViolationFailedToFurnishPayeeStatementSection6722);
    }

    #[test]
    fn rent_payment_type_uses_1099_misc() {
        let input = Input {
            payment_type: PaymentType::RentMisc1099,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantForm1099MiscFiledTimely);
    }

    #[test]
    fn citations_pin_section_6041_subsections_and_obbba() {
        let result = compute(&baseline_2026_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 6041(a)"));
        assert!(joined.contains("§ 6041(c)"));
        assert!(joined.contains("§ 6041(h)"));
        assert!(joined.contains("§ 6041A"));
        assert!(joined.contains("§ 3406"));
        assert!(joined.contains("§ 6721"));
        assert!(joined.contains("§ 6722"));
        assert!(joined.contains("§ 6723"));
        assert!(joined.contains("P.L. 119-21"));
        assert!(joined.contains("§ 70433"));
        assert!(joined.contains("Form 1099-NEC"));
        assert!(joined.contains("Form 1099-MISC"));
        assert!(joined.contains("Form W-9"));
        assert!(joined.contains("Form W-8BEN"));
        assert!(joined.contains("Form 1042"));
    }

    #[test]
    fn constant_pin_obbba_thresholds_and_effective_date() {
        assert_eq!(SECTION_6041_OBBBA_POST_2025_THRESHOLD_DOLLARS, 2_000);
        assert_eq!(SECTION_6041_PRE_OBBBA_THRESHOLD_DOLLARS, 600);
        assert_eq!(SECTION_6041_OBBBA_EFFECTIVE_AFTER_YEAR, 2025);
        assert_eq!(SECTION_6041_OBBBA_EFFECTIVE_AFTER_MONTH, 12);
        assert_eq!(SECTION_6041_OBBBA_EFFECTIVE_AFTER_DAY, 31);
        assert_eq!(SECTION_6041_INFLATION_INDEXING_BASE_YEAR, 2025);
        assert_eq!(SECTION_6041_INFLATION_ROUNDING_DOLLARS, 100);
        assert_eq!(SECTION_3406_BACKUP_WITHHOLDING_BASIS_POINTS, 2_400);
        assert_eq!(SECTION_6041_OBBBA_PUBLIC_LAW_NUMBER_MAJOR, 119);
        assert_eq!(SECTION_6041_OBBBA_PUBLIC_LAW_NUMBER_MINOR, 21);
        assert_eq!(SECTION_6041_OBBBA_SECTION_NUMBER, 70_433);
    }

    #[test]
    fn saturating_overflow_defense_extreme_payment() {
        let input = Input {
            payment_amount_dollars: u64::MAX,
            ..baseline_2026_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section6041Mode::CompliantForm1099NecFiledTimelyJanuary31);
    }
}
