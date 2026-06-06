//! IRC § 1471 — Withholdable Payments to Foreign Financial
//! Institutions (FATCA Chapter 4 Withholding).
//!
//! Pure-compute FATCA Chapter 4 withholding obligation under
//! § 1471 (FFIs) and § 1472 (NFFEs). Withholding agent must
//! deduct and withhold **30 % of any withholdable payment** made
//! to a foreign financial institution (FFI) or non-financial
//! foreign entity (NFFE) that does NOT meet Chapter 4
//! documentation and reporting requirements. Withholding agent
//! is statutory withholding agent and personally liable for tax
//! not withheld; remits via Forms 1042 / 1042-S.
//!
//! Statute (verbatim mapping):
//! - § 1471(a) — GENERAL RULE: in the case of any withholdable
//!   payment to an FFI which does not meet the requirements of
//!   subsection (b), the withholding agent shall deduct and
//!   withhold from such payment a tax equal to 30 percent of the
//!   amount of such payment.
//! - § 1471(b) — REPORTING REQUIREMENTS FOR PARTICIPATING FFI:
//!   FFI must enter into FFI Agreement with IRS to identify US
//!   accounts, report annual account info on Form 8966, comply
//!   with verification and due diligence procedures, withhold on
//!   payments to recalcitrant account holders.
//! - § 1471(b)(2) — DEEMED-COMPLIANT FFI: Treasury may treat
//!   certain FFIs as meeting § 1471(b) requirements based on
//!   procedures specified in Treas. Reg. § 1.1471-5(f).
//! - § 1471(c) — INFORMATION REPORTING by participating FFI on
//!   each US account: name + TIN + account number + balance +
//!   gross receipts / withdrawals.
//! - § 1471(d) — DEFINITIONS: FFI = any financial institution
//!   which is a foreign entity; financial institution includes
//!   depository institutions, custodial institutions, investment
//!   entities, and certain insurance companies.
//! - § 1471(e) — EXEMPT BENEFICIAL OWNERS: foreign governments,
//!   international organizations, foreign central banks, and
//!   certain other entities are exempt.
//! - § 1471(f) — EXCEPTION FOR PRE-EXISTING OBLIGATIONS:
//!   payments under obligations outstanding on January 1, 2014.
//! - § 1472 — WITHHOLDING ON NON-FINANCIAL FOREIGN ENTITIES
//!   (NFFE): 30 % withholding on withholdable payments to NFFE
//!   unless NFFE certifies it has no substantial US owners OR
//!   provides identifying info about substantial US owners.
//! - § 1473 — DEFINITIONS: withholdable payment = (A) US-source
//!   FDAP (interest, dividends, rents, royalties, etc.); (B)
//!   gross proceeds from sale of property producing US-source
//!   interest or dividends. Withholding on (B) gross proceeds
//!   was deferred to 2019 then permanently rescinded by Notice
//!   2014-33 and Treas. Reg. § 1.1473-1(a).
//! - § 1474 — SPECIAL RULES: refund procedure + double-tax
//!   recovery + treaty coordination.
//!
//! Web research (verified 2026-06-03):
//! - IRS FATCA page: confirms 30 % rate; W-8BEN-E for FFI/NFFE;
//!   GIIN for Chapter 4 status; Form 1042 annual return.
//! - Cornell LII § 1471 full statutory text.
//! - 26 CFR § 1.1471-0 et seq. — implementing regulations.
//! - U.S. Law Explained FATCA Ultimate Guide.
//!
//! Forms:
//! - W-8BEN-E — foreign entity certification of Chapter 3
//!   beneficial owner status + Chapter 4 FATCA status.
//! - W-8IMY — foreign intermediary (qualified intermediary,
//!   nonqualified intermediary, withholding partnership/trust).
//! - W-8EXP — foreign government / international organization.
//! - W-8ECI — effectively connected income.
//! - W-9 — US person.
//! - Form 1042 — Annual Withholding Tax Return for US Source
//!   Income of Foreign Persons.
//! - Form 1042-S — Foreign Person's US Source Income Subject
//!   to Withholding (per-payee statement).
//! - Form 8966 — FATCA Report (annual reporting by participating
//!   FFI and registered deemed-compliant FFI).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS: u64 = 3_000;
pub const SECTION_1471_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1471_FATCA_ENACTMENT_YEAR: u32 = 2010;
pub const SECTION_1471_PRE_EXISTING_OBLIGATIONS_DATE_YEAR: u32 = 2014;
pub const SECTION_1471_PRE_EXISTING_OBLIGATIONS_DATE_MONTH: u32 = 1;
pub const SECTION_1471_PRE_EXISTING_OBLIGATIONS_DATE_DAY: u32 = 1;
pub const SECTION_1471_FORM_1042_DEADLINE_MONTH: u32 = 3;
pub const SECTION_1471_FORM_1042_DEADLINE_DAY: u32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayeeChapter4Status {
    UsPersonProvidedFormW9,
    ParticipatingFfiWithGiin,
    RegisteredDeemedCompliantFfiWithGiin,
    CertifiedDeemedCompliantFfi,
    ReportingModel1FfiWithGiin,
    ReportingModel2FfiWithGiin,
    OwnerDocumentedFfi,
    ExceptedFfi,
    ExemptBeneficialOwner,
    NonparticipatingFfi,
    ActiveNffe,
    PassiveNffeNoSubstantialUsOwners,
    PassiveNffeWithReportedSubstantialUsOwners,
    PassiveNffeUndocumented,
    RecalcitrantAccountHolder,
    UndocumentedPayee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithholdablePaymentType {
    UsSourceFdapInterest,
    UsSourceFdapDividend,
    UsSourceFdapRents,
    UsSourceFdapRoyalties,
    UsSourceFdapOther,
    GrossProceedsFromSaleOfPropertyRescindedByTreasReg,
    PreExistingObligationOutstandingJanuary1_2014,
    NonUsSourcePayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1471Mode {
    NotApplicableUsPayee,
    NotApplicableNonUsSourcePayment,
    NotApplicableGrossProceedsRescinded,
    NotApplicablePreExistingObligation,
    CompliantParticipatingFfiNoWithholdingRequired,
    CompliantDeemedCompliantFfiNoWithholdingRequired,
    CompliantReportingModel1Or2FfiNoWithholdingRequired,
    CompliantExemptBeneficialOwnerNoWithholdingRequired,
    CompliantActiveNffeNoWithholdingRequired,
    CompliantPassiveNffeWithSubstantialUsOwnersReported,
    CompliantPassiveNffeNoSubstantialUsOwnersCertified,
    CompliantFullWithholdingAppliedToNonparticipatingFfi,
    ViolationWithholdingAgentFailedToWithholdFromNonparticipatingFfi,
    ViolationUndocumentedPayeeNoWithholdingApplied,
    ViolationForm1042NotFiledByMarch15Deadline,
    ViolationForm1042SNotIssuedToForeignPayee,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub payee_chapter4_status: PayeeChapter4Status,
    pub payment_type: WithholdablePaymentType,
    pub payment_amount_dollars: u64,
    pub actual_withheld_dollars: u64,
    pub form_1042_filed_by_march_15_deadline: bool,
    pub form_1042_s_issued_to_foreign_payee: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1471Mode,
    pub required_withholding_dollars: u64,
    pub applicable_rate_basis_points: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1471Input = Input;
pub type Section1471Output = Output;
pub type Section1471Result = Output;

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_1471_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1471(a) — 30 % withholding on withholdable payments to FFI not meeting § 1471(b)".to_string(),
        "26 U.S.C. § 1471(b) — participating FFI: FFI agreement with IRS; report US accounts on Form 8966; withhold on recalcitrant accounts".to_string(),
        "26 U.S.C. § 1471(b)(2) — deemed-compliant FFI per Treas. Reg. § 1.1471-5(f)".to_string(),
        "26 U.S.C. § 1471(c) — information reporting per participating FFI on each US account (name + TIN + account number + balance + gross receipts)".to_string(),
        "26 U.S.C. § 1471(d) — FFI definition includes depository, custodial, investment entity, certain insurance".to_string(),
        "26 U.S.C. § 1471(e) — exempt beneficial owners: foreign governments, international organizations, foreign central banks".to_string(),
        "26 U.S.C. § 1471(f) — pre-existing obligations outstanding January 1, 2014 exempt".to_string(),
        "26 U.S.C. § 1472 — 30 % withholding on withholdable payments to NFFE unless NFFE certifies no substantial US owners OR identifies substantial US owners".to_string(),
        "26 U.S.C. § 1473 — withholdable payment: US-source FDAP interest/dividends/rents/royalties; gross proceeds rescinded by Treas. Reg. § 1.1473-1(a) + Notice 2014-33".to_string(),
        "26 U.S.C. § 1474 — special rules: refund procedure + double-tax recovery + treaty coordination".to_string(),
        "Form W-8BEN-E — foreign entity Chapter 3 + Chapter 4 status certification".to_string(),
        "Form W-8IMY — foreign intermediary status".to_string(),
        "Form W-8EXP — foreign government / international organization".to_string(),
        "Form W-9 — US person".to_string(),
        "Form 1042 — Annual Withholding Tax Return for US Source Income of Foreign Persons; due March 15".to_string(),
        "Form 1042-S — Foreign Person's US Source Income Subject to Withholding (per-payee statement)".to_string(),
        "Form 8966 — FATCA Report (annual)".to_string(),
        "GIIN — Global Intermediary Identification Number established Chapter 4 status".to_string(),
    ];

    if input.payee_chapter4_status == PayeeChapter4Status::UsPersonProvidedFormW9 {
        return Output {
            mode: Section1471Mode::NotApplicableUsPayee,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1471 — Chapter 4 inapplicable to US person who provided Form W-9".to_string(),
            notes: format!(
                "US payee with Form W-9 on file; § 1471 FATCA Chapter 4 withholding inapplicable. Payment amount = ${}.",
                input.payment_amount_dollars
            ),
            citations,
        };
    }

    if input.payment_type == WithholdablePaymentType::NonUsSourcePayment {
        return Output {
            mode: Section1471Mode::NotApplicableNonUsSourcePayment,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1473 — payment is not a withholdable payment (non-US-source)"
                .to_string(),
            notes: "Payment is non-US-source; not a withholdable payment under § 1473.".to_string(),
            citations,
        };
    }

    if input.payment_type
        == WithholdablePaymentType::GrossProceedsFromSaleOfPropertyRescindedByTreasReg
    {
        return Output {
            mode: Section1471Mode::NotApplicableGrossProceedsRescinded,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "Treas. Reg. § 1.1473-1(a) + Notice 2014-33 — gross proceeds withholding permanently rescinded".to_string(),
            notes: "Gross proceeds from sale of property: § 1473 withholding was deferred to 2019 then permanently rescinded by Treas. Reg. § 1.1473-1(a).".to_string(),
            citations,
        };
    }

    if input.payment_type == WithholdablePaymentType::PreExistingObligationOutstandingJanuary1_2014
    {
        return Output {
            mode: Section1471Mode::NotApplicablePreExistingObligation,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1471(f) — pre-existing obligation outstanding January 1, 2014 exempt".to_string(),
            notes: "Payment under obligation outstanding January 1, 2014; § 1471(f) grandfather exemption applies.".to_string(),
            citations,
        };
    }

    let compliant_status = match input.payee_chapter4_status {
        PayeeChapter4Status::ParticipatingFfiWithGiin => {
            Some(Section1471Mode::CompliantParticipatingFfiNoWithholdingRequired)
        }
        PayeeChapter4Status::RegisteredDeemedCompliantFfiWithGiin
        | PayeeChapter4Status::CertifiedDeemedCompliantFfi
        | PayeeChapter4Status::OwnerDocumentedFfi
        | PayeeChapter4Status::ExceptedFfi => {
            Some(Section1471Mode::CompliantDeemedCompliantFfiNoWithholdingRequired)
        }
        PayeeChapter4Status::ReportingModel1FfiWithGiin
        | PayeeChapter4Status::ReportingModel2FfiWithGiin => {
            Some(Section1471Mode::CompliantReportingModel1Or2FfiNoWithholdingRequired)
        }
        PayeeChapter4Status::ExemptBeneficialOwner => {
            Some(Section1471Mode::CompliantExemptBeneficialOwnerNoWithholdingRequired)
        }
        PayeeChapter4Status::ActiveNffe => {
            Some(Section1471Mode::CompliantActiveNffeNoWithholdingRequired)
        }
        PayeeChapter4Status::PassiveNffeNoSubstantialUsOwners => {
            Some(Section1471Mode::CompliantPassiveNffeNoSubstantialUsOwnersCertified)
        }
        PayeeChapter4Status::PassiveNffeWithReportedSubstantialUsOwners => {
            Some(Section1471Mode::CompliantPassiveNffeWithSubstantialUsOwnersReported)
        }
        _ => None,
    };

    if let Some(mode) = compliant_status {
        if !input.form_1042_filed_by_march_15_deadline {
            return Output {
                mode: Section1471Mode::ViolationForm1042NotFiledByMarch15Deadline,
                required_withholding_dollars: 0,
                applicable_rate_basis_points: 0,
                statutory_basis: "Form 1042 due March 15 (15th day of 3rd month after close of calendar year)".to_string(),
                notes: "VIOLATION: payee status itself does not trigger withholding, but Form 1042 annual return not filed by March 15 deadline. Late filing subject to § 6651(a)(1) penalty + interest.".to_string(),
                citations,
            };
        }
        if !input.form_1042_s_issued_to_foreign_payee {
            return Output {
                mode: Section1471Mode::ViolationForm1042SNotIssuedToForeignPayee,
                required_withholding_dollars: 0,
                applicable_rate_basis_points: 0,
                statutory_basis: "Form 1042-S per-payee statement required regardless of zero withholding".to_string(),
                notes: "VIOLATION: Form 1042-S not issued to foreign payee. Per-payee reporting required even when withholding is zero. Subject to § 6722 penalty.".to_string(),
                citations,
            };
        }
        return Output {
            mode,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: format!(
                "§ 1471 / § 1472 — payee Chapter 4 status {:?} satisfies documentation requirements; no withholding required",
                input.payee_chapter4_status
            ),
            notes: format!(
                "COMPLIANT: payee Chapter 4 status = {:?}; payment amount = ${}; no Chapter 4 withholding required. Form 1042 + Form 1042-S filed timely.",
                input.payee_chapter4_status, input.payment_amount_dollars
            ),
            citations,
        };
    }

    let required = apply_rate(
        input.payment_amount_dollars,
        SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS,
    );

    if input.payee_chapter4_status == PayeeChapter4Status::UndocumentedPayee
        && input.actual_withheld_dollars < required
    {
        return Output {
            mode: Section1471Mode::ViolationUndocumentedPayeeNoWithholdingApplied,
            required_withholding_dollars: required,
            applicable_rate_basis_points: SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS,
            statutory_basis: "§ 1471(a) — undocumented payee presumed nonparticipating; 30 % withholding required".to_string(),
            notes: format!(
                "VIOLATION § 1471(a): payee did not provide Form W-8 or W-9 documentation; presumed nonparticipating. 30 % withholding required on payment ${} = ${}. Withholding agent personally liable.",
                input.payment_amount_dollars, required
            ),
            citations,
        };
    }

    if input.actual_withheld_dollars < required {
        return Output {
            mode: Section1471Mode::ViolationWithholdingAgentFailedToWithholdFromNonparticipatingFfi,
            required_withholding_dollars: required,
            applicable_rate_basis_points: SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS,
            statutory_basis: "§ 1471(a) — 30 % withholding required from nonparticipating FFI / undocumented passive NFFE".to_string(),
            notes: format!(
                "VIOLATION § 1471(a): payee status = {:?} requires 30 % FATCA withholding. Payment ${}; required ${}; actual withheld ${}; shortfall ${}. Withholding agent personally liable + § 6651/§ 6656 penalties.",
                input.payee_chapter4_status,
                input.payment_amount_dollars,
                required,
                input.actual_withheld_dollars,
                required.saturating_sub(input.actual_withheld_dollars)
            ),
            citations,
        };
    }

    if !input.form_1042_filed_by_march_15_deadline {
        return Output {
            mode: Section1471Mode::ViolationForm1042NotFiledByMarch15Deadline,
            required_withholding_dollars: required,
            applicable_rate_basis_points: SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS,
            statutory_basis: "Form 1042 due March 15".to_string(),
            notes: format!(
                "VIOLATION: 30 % withholding of ${} computed and remitted, but Form 1042 not filed by March 15. Late filing subject to § 6651(a)(1) penalty + interest.",
                required
            ),
            citations,
        };
    }

    if !input.form_1042_s_issued_to_foreign_payee {
        return Output {
            mode: Section1471Mode::ViolationForm1042SNotIssuedToForeignPayee,
            required_withholding_dollars: required,
            applicable_rate_basis_points: SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS,
            statutory_basis: "Form 1042-S per-payee statement required".to_string(),
            notes: "VIOLATION: Form 1042-S not issued to foreign payee. Subject to § 6722 penalty."
                .to_string(),
            citations,
        };
    }

    Output {
        mode: Section1471Mode::CompliantFullWithholdingAppliedToNonparticipatingFfi,
        required_withholding_dollars: required,
        applicable_rate_basis_points: SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS,
        statutory_basis: "§ 1471(a) — 30 % FATCA withholding applied to nonparticipating payee".to_string(),
        notes: format!(
            "COMPLIANT § 1471: 30 % withholding of ${} applied to payment ${} for nonparticipating payee. Form 1042 + Form 1042-S filed timely.",
            required, input.payment_amount_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_nonparticipating_ffi_compliant() -> Input {
        Input {
            payee_chapter4_status: PayeeChapter4Status::NonparticipatingFfi,
            payment_type: WithholdablePaymentType::UsSourceFdapInterest,
            payment_amount_dollars: 1_000_000,
            actual_withheld_dollars: 300_000,
            form_1042_filed_by_march_15_deadline: true,
            form_1042_s_issued_to_foreign_payee: true,
        }
    }

    fn baseline_participating_ffi() -> Input {
        Input {
            payee_chapter4_status: PayeeChapter4Status::ParticipatingFfiWithGiin,
            payment_type: WithholdablePaymentType::UsSourceFdapDividend,
            payment_amount_dollars: 500_000,
            actual_withheld_dollars: 0,
            form_1042_filed_by_march_15_deadline: true,
            form_1042_s_issued_to_foreign_payee: true,
        }
    }

    #[test]
    fn us_person_w9_not_applicable() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::UsPersonProvidedFormW9,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1471Mode::NotApplicableUsPayee);
    }

    #[test]
    fn non_us_source_payment_not_applicable() {
        let input = Input {
            payment_type: WithholdablePaymentType::NonUsSourcePayment,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::NotApplicableNonUsSourcePayment
        );
    }

    #[test]
    fn gross_proceeds_rescinded_not_applicable() {
        let input = Input {
            payment_type:
                WithholdablePaymentType::GrossProceedsFromSaleOfPropertyRescindedByTreasReg,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::NotApplicableGrossProceedsRescinded
        );
    }

    #[test]
    fn pre_existing_obligation_january_1_2014_grandfathered() {
        let input = Input {
            payment_type: WithholdablePaymentType::PreExistingObligationOutstandingJanuary1_2014,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::NotApplicablePreExistingObligation
        );
    }

    #[test]
    fn participating_ffi_with_giin_compliant_no_withholding() {
        let result = compute(&baseline_participating_ffi());
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantParticipatingFfiNoWithholdingRequired
        );
        assert_eq!(result.required_withholding_dollars, 0);
    }

    #[test]
    fn deemed_compliant_ffi_no_withholding() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::RegisteredDeemedCompliantFfiWithGiin,
            ..baseline_participating_ffi()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantDeemedCompliantFfiNoWithholdingRequired
        );
    }

    #[test]
    fn reporting_model_1_iga_ffi_no_withholding() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::ReportingModel1FfiWithGiin,
            ..baseline_participating_ffi()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantReportingModel1Or2FfiNoWithholdingRequired
        );
    }

    #[test]
    fn exempt_beneficial_owner_no_withholding() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::ExemptBeneficialOwner,
            ..baseline_participating_ffi()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantExemptBeneficialOwnerNoWithholdingRequired
        );
    }

    #[test]
    fn active_nffe_no_withholding() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::ActiveNffe,
            ..baseline_participating_ffi()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantActiveNffeNoWithholdingRequired
        );
    }

    #[test]
    fn passive_nffe_no_us_owners_certified_compliant() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::PassiveNffeNoSubstantialUsOwners,
            ..baseline_participating_ffi()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantPassiveNffeNoSubstantialUsOwnersCertified
        );
    }

    #[test]
    fn passive_nffe_with_reported_us_owners_compliant() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::PassiveNffeWithReportedSubstantialUsOwners,
            ..baseline_participating_ffi()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantPassiveNffeWithSubstantialUsOwnersReported
        );
    }

    #[test]
    fn nonparticipating_ffi_30_pct_withholding_compliant() {
        let result = compute(&baseline_nonparticipating_ffi_compliant());
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantFullWithholdingAppliedToNonparticipatingFfi
        );
        assert_eq!(result.required_withholding_dollars, 300_000);
        assert_eq!(result.applicable_rate_basis_points, 3_000);
    }

    #[test]
    fn nonparticipating_ffi_withholding_failed_violation() {
        let input = Input {
            actual_withheld_dollars: 0,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::ViolationWithholdingAgentFailedToWithholdFromNonparticipatingFfi
        );
    }

    #[test]
    fn nonparticipating_ffi_under_withheld_violation_with_shortfall() {
        let input = Input {
            actual_withheld_dollars: 100_000,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::ViolationWithholdingAgentFailedToWithholdFromNonparticipatingFfi
        );
        assert!(result.notes.contains("shortfall $200000"));
    }

    #[test]
    fn undocumented_payee_no_withholding_violation() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::UndocumentedPayee,
            actual_withheld_dollars: 0,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::ViolationUndocumentedPayeeNoWithholdingApplied
        );
        assert_eq!(result.required_withholding_dollars, 300_000);
    }

    #[test]
    fn passive_nffe_undocumented_30_pct_withholding_required() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::PassiveNffeUndocumented,
            actual_withheld_dollars: 300_000,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantFullWithholdingAppliedToNonparticipatingFfi
        );
        assert_eq!(result.required_withholding_dollars, 300_000);
    }

    #[test]
    fn recalcitrant_account_holder_30_pct_withholding() {
        let input = Input {
            payee_chapter4_status: PayeeChapter4Status::RecalcitrantAccountHolder,
            actual_withheld_dollars: 300_000,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::CompliantFullWithholdingAppliedToNonparticipatingFfi
        );
    }

    #[test]
    fn form_1042_late_filing_violation() {
        let input = Input {
            form_1042_filed_by_march_15_deadline: false,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::ViolationForm1042NotFiledByMarch15Deadline
        );
    }

    #[test]
    fn form_1042_s_not_issued_violation() {
        let input = Input {
            form_1042_s_issued_to_foreign_payee: false,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1471Mode::ViolationForm1042SNotIssuedToForeignPayee
        );
    }

    #[test]
    fn citations_pin_section_1471_subsections_and_forms() {
        let result = compute(&baseline_nonparticipating_ffi_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1471(a)"));
        assert!(joined.contains("§ 1471(b)"));
        assert!(joined.contains("§ 1471(b)(2)"));
        assert!(joined.contains("§ 1471(c)"));
        assert!(joined.contains("§ 1471(d)"));
        assert!(joined.contains("§ 1471(e)"));
        assert!(joined.contains("§ 1471(f)"));
        assert!(joined.contains("§ 1472"));
        assert!(joined.contains("§ 1473"));
        assert!(joined.contains("§ 1474"));
        assert!(joined.contains("W-8BEN-E"));
        assert!(joined.contains("W-8IMY"));
        assert!(joined.contains("W-8EXP"));
        assert!(joined.contains("W-9"));
        assert!(joined.contains("Form 1042"));
        assert!(joined.contains("Form 1042-S"));
        assert!(joined.contains("Form 8966"));
        assert!(joined.contains("GIIN"));
    }

    #[test]
    fn constant_pin_30_pct_rate_and_deadlines() {
        assert_eq!(SECTION_1471_WITHHOLDING_RATE_BASIS_POINTS, 3_000);
        assert_eq!(SECTION_1471_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SECTION_1471_FATCA_ENACTMENT_YEAR, 2010);
        assert_eq!(SECTION_1471_PRE_EXISTING_OBLIGATIONS_DATE_YEAR, 2014);
        assert_eq!(SECTION_1471_PRE_EXISTING_OBLIGATIONS_DATE_MONTH, 1);
        assert_eq!(SECTION_1471_PRE_EXISTING_OBLIGATIONS_DATE_DAY, 1);
        assert_eq!(SECTION_1471_FORM_1042_DEADLINE_MONTH, 3);
        assert_eq!(SECTION_1471_FORM_1042_DEADLINE_DAY, 15);
    }

    #[test]
    fn saturating_overflow_defense_extreme_payment() {
        let input = Input {
            payment_amount_dollars: u64::MAX,
            actual_withheld_dollars: u64::MAX,
            ..baseline_nonparticipating_ffi_compliant()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section1471Mode::CompliantFullWithholdingAppliedToNonparticipatingFfi
                | Section1471Mode::ViolationWithholdingAgentFailedToWithholdFromNonparticipatingFfi
        ));
    }
}
