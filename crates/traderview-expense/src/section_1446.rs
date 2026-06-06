//! IRC § 1446 — Withholding of Tax on Foreign Partners' Share of
//! Effectively Connected Income (Partnership Withholding).
//!
//! Pure-compute § 1446 withholding obligation. A partnership
//! (foreign or domestic) that has income effectively connected
//! with a U.S. trade or business — or income treated as
//! effectively connected — must pay a withholding tax on the
//! effectively connected taxable income (ECTI) allocable to its
//! foreign partners. The partnership is the statutory withholding
//! agent and is personally liable for tax not withheld.
//!
//! Statute (verbatim mapping):
//! - § 1446(a) — GENERAL RULE: a partnership with ECTI allocable
//!   to a foreign partner must pay a withholding tax determined
//!   under § 1446(b).
//! - § 1446(b)(1) — RATE: the highest rate of tax in effect under
//!   § 1 (currently 37 %) for noncorporate partners; the highest
//!   rate under § 11 (currently 21 %) for corporate partners. For
//!   qualifying income items the partnership may withhold at lower
//!   rates with appropriate documentation: 20 % LTCG, 25 %
//!   unrecaptured § 1250 gain, 28 % collectibles gain.
//! - § 1446(c) — DEFINITIONS: foreign partner = any partner who
//!   is not a United States person within the meaning of § 7701(a).
//! - § 1446(d) — TREATMENT: tax treated as paid by the foreign
//!   partner and creditable against partner's U.S. tax liability
//!   per Form 8805.
//! - § 1446(e) — INSTALLMENT PAYMENTS: partnership must pay the
//!   IRS portions of the annual withholding tax by the 15th day
//!   of the 4th, 6th, 9th, and 12th months of its tax year on
//!   Form 8813 voucher.
//! - § 1446(f) — TRANSFER OF PARTNERSHIP INTEREST BY FOREIGN
//!   PERSON: added by TCJA 2017; transferee withholds 10 % of
//!   amount realized on transfer of a partnership interest if a
//!   portion of the gain would be ECI; permanent 10 % rate per
//!   Treas. Reg. § 1.1446(f)-1 et seq. (final regs Oct 7, 2020).
//!
//! Form 8804 — Annual Return for Partnership Withholding Tax;
//!   due 15th day of 3rd month after close of partnership tax year
//!   (or 6th month if § 6081 extension).
//! Form 8805 — Foreign Partner's Information Statement of § 1446
//!   Withholding Tax; one per foreign partner with allocated ECTI.
//! Form 8813 — Partnership Withholding Tax Payment Voucher;
//!   quarterly installment.
//! Form 8804-C — Certificate by foreign partner reducing § 1446
//!   withholding via partner-level deductions/losses.
//!
//! Web research (verified 2026-06-03):
//! - IRS Partnership Withholding page: confirms 37 % noncorporate /
//!   21 % corporate rates; Form 8804/8805/8813.
//! - Cornell LII § 1446 full statutory text.
//! - Gordon Law Group: Forms 8804/8805 + § 1446 Withholding guide.
//! - Treas. Reg. § 1.1446(f)-1 (PTP/partnership interest transfer
//!   withholding final regs Oct 7, 2020) confirms 10 % rate.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1446_NONCORPORATE_RATE_BASIS_POINTS: u64 = 3_700;
pub const SECTION_1446_CORPORATE_RATE_BASIS_POINTS: u64 = 2_100;
pub const SECTION_1446_LTCG_RATE_BASIS_POINTS: u64 = 2_000;
pub const SECTION_1446_UNRECAPTURED_1250_RATE_BASIS_POINTS: u64 = 2_500;
pub const SECTION_1446_COLLECTIBLES_RATE_BASIS_POINTS: u64 = 2_800;
pub const SECTION_1446_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1446_F_TRANSFER_WITHHOLDING_RATE_BASIS_POINTS: u64 = 1_000;
pub const SECTION_1446_QUARTERLY_INSTALLMENT_MONTHS: [u32; 4] = [4, 6, 9, 12];
pub const SECTION_1446_QUARTERLY_INSTALLMENT_DAY: u32 = 15;
pub const SECTION_1446_FORM_8804_ANNUAL_DEADLINE_MONTH: u32 = 3;
pub const SECTION_1446_FORM_8804_ANNUAL_DEADLINE_DAY: u32 = 15;
pub const SECTION_1446_F_TCJA_ENACTMENT_YEAR: u32 = 2017;
pub const SECTION_1446_F_FINAL_REGS_YEAR: u32 = 2020;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PartnerType {
    UsPersonNotForeign,
    NoncorporateForeignPartner,
    CorporateForeignPartner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EctiIncomeType {
    OrdinaryEcti,
    LongTermCapitalGain,
    UnrecapturedSection1250Gain,
    CollectiblesGain,
    QualifiedDividends,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithholdingScenario {
    AnnualEctiWithholding,
    Section1446fPartnershipInterestTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1446Mode {
    NotApplicableNoForeignPartnerOrNoEcti,
    CompliantQuarterlyInstallmentsPaidAndForm8804Filed,
    CompliantSection1446fTenPercentTransferWithholdingApplied,
    CompliantPartnerLevelCertificateFiledForm8804C,
    ViolationPartnershipFailedToWithholdFromForeignPartner,
    ViolationQuarterlyInstallmentMissed,
    ViolationForm8804NotFiledAtAnnualDeadline,
    ViolationForm8805NotIssuedToForeignPartner,
    ViolationWithheldAtIncorrectRate,
    ViolationSection1446fTransferWithholdingFailed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub scenario: WithholdingScenario,
    pub partner_type: PartnerType,
    pub ecti_income_type: EctiIncomeType,
    pub allocable_ecti_or_amount_realized_dollars: u64,
    pub partnership_actual_withheld_dollars: u64,
    pub quarterly_installments_paid_count: u32,
    pub form_8804_filed_by_annual_deadline: bool,
    pub form_8805_issued_to_each_foreign_partner: bool,
    pub form_8804_c_partner_level_certificate_filed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1446Mode,
    pub required_withholding_dollars: u64,
    pub applicable_rate_basis_points: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1446Input = Input;
pub type Section1446Output = Output;
pub type Section1446Result = Output;

fn rate_for(partner: PartnerType, income: EctiIncomeType) -> u64 {
    match (partner, income) {
        (PartnerType::CorporateForeignPartner, _) => SECTION_1446_CORPORATE_RATE_BASIS_POINTS,
        (PartnerType::NoncorporateForeignPartner, EctiIncomeType::LongTermCapitalGain) => {
            SECTION_1446_LTCG_RATE_BASIS_POINTS
        }
        (PartnerType::NoncorporateForeignPartner, EctiIncomeType::UnrecapturedSection1250Gain) => {
            SECTION_1446_UNRECAPTURED_1250_RATE_BASIS_POINTS
        }
        (PartnerType::NoncorporateForeignPartner, EctiIncomeType::CollectiblesGain) => {
            SECTION_1446_COLLECTIBLES_RATE_BASIS_POINTS
        }
        (PartnerType::NoncorporateForeignPartner, _) => SECTION_1446_NONCORPORATE_RATE_BASIS_POINTS,
        (PartnerType::UsPersonNotForeign, _) => 0,
    }
}

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_1446_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1446(a) — partnership with ECTI allocable to foreign partner must pay withholding tax".to_string(),
        "26 U.S.C. § 1446(b)(1) — rate: highest § 1 rate (37 %) for noncorporate; highest § 11 rate (21 %) for corporate".to_string(),
        "26 U.S.C. § 1446(c) — foreign partner = any partner not a US person under § 7701(a)".to_string(),
        "26 U.S.C. § 1446(d) — tax treated as paid by foreign partner; creditable on partner's U.S. return".to_string(),
        "26 U.S.C. § 1446(e) — quarterly installment payments due 15th of 4th/6th/9th/12th months on Form 8813".to_string(),
        "26 U.S.C. § 1446(f) — TCJA 2017: transferee withholds 10 % of amount realized on foreign partner's transfer of partnership interest if portion of gain would be ECI".to_string(),
        "Form 8804 — Annual Return for Partnership Withholding Tax; due 15th day of 3rd month after close of partnership tax year".to_string(),
        "Form 8805 — Foreign Partner's Information Statement of § 1446 Withholding Tax".to_string(),
        "Form 8813 — Partnership Withholding Tax Payment Voucher (quarterly installment)".to_string(),
        "Form 8804-C — Certificate by foreign partner reducing § 1446 withholding via partner-level deductions/losses".to_string(),
        "Treas. Reg. § 1.1446-3 — quarterly installment procedure".to_string(),
        "Treas. Reg. § 1.1446(f)-1 et seq. — partnership interest transfer withholding final regs effective Oct 7, 2020".to_string(),
        "TCJA 2017 (P.L. 115-97) § 13501 — added § 1446(f) for partnership interest transfers".to_string(),
        "Reduced rates for LTCG (20 %), unrecaptured § 1250 (25 %), collectibles (28 %) available with appropriate documentation".to_string(),
    ];

    if input.partner_type == PartnerType::UsPersonNotForeign
        || input.allocable_ecti_or_amount_realized_dollars == 0
    {
        return Output {
            mode: Section1446Mode::NotApplicableNoForeignPartnerOrNoEcti,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1446(a) — no foreign partner with ECTI or no amount realized".to_string(),
            notes: format!(
                "No § 1446 withholding required. Partner = {:?}; allocable ECTI / amount realized = ${}.",
                input.partner_type, input.allocable_ecti_or_amount_realized_dollars
            ),
            citations,
        };
    }

    if input.scenario == WithholdingScenario::Section1446fPartnershipInterestTransfer {
        let required = apply_rate(
            input.allocable_ecti_or_amount_realized_dollars,
            SECTION_1446_F_TRANSFER_WITHHOLDING_RATE_BASIS_POINTS,
        );
        if input.partnership_actual_withheld_dollars < required {
            return Output {
                mode: Section1446Mode::ViolationSection1446fTransferWithholdingFailed,
                required_withholding_dollars: required,
                applicable_rate_basis_points: SECTION_1446_F_TRANSFER_WITHHOLDING_RATE_BASIS_POINTS,
                statutory_basis: "§ 1446(f) — transferee withholds 10 % of amount realized on foreign-partner partnership interest transfer".to_string(),
                notes: format!(
                    "VIOLATION § 1446(f): transferee withheld ${} on transfer with amount realized = ${}; required withholding at 10 % = ${} (shortfall ${}). Transferee personally liable for unwithheld tax under TCJA 2017.",
                    input.partnership_actual_withheld_dollars,
                    input.allocable_ecti_or_amount_realized_dollars,
                    required,
                    required.saturating_sub(input.partnership_actual_withheld_dollars)
                ),
                citations,
            };
        }
        return Output {
            mode: Section1446Mode::CompliantSection1446fTenPercentTransferWithholdingApplied,
            required_withholding_dollars: required,
            applicable_rate_basis_points: SECTION_1446_F_TRANSFER_WITHHOLDING_RATE_BASIS_POINTS,
            statutory_basis: "§ 1446(f) — 10 % transferee withholding satisfied".to_string(),
            notes: format!(
                "COMPLIANT § 1446(f): transferee withheld ${} on amount realized = ${} at 10 % rate; required = ${}.",
                input.partnership_actual_withheld_dollars,
                input.allocable_ecti_or_amount_realized_dollars,
                required
            ),
            citations,
        };
    }

    if input.form_8804_c_partner_level_certificate_filed {
        return Output {
            mode: Section1446Mode::CompliantPartnerLevelCertificateFiledForm8804C,
            required_withholding_dollars: input.partnership_actual_withheld_dollars,
            applicable_rate_basis_points: 0,
            statutory_basis: "Form 8804-C — partner-level certificate reducing § 1446 withholding accepted".to_string(),
            notes: format!(
                "COMPLIANT: foreign partner filed Form 8804-C certificate establishing partner-level deductions/losses; partnership reduced withholding to ${} on ECTI of ${} per certificate.",
                input.partnership_actual_withheld_dollars, input.allocable_ecti_or_amount_realized_dollars
            ),
            citations,
        };
    }

    let rate_bp = rate_for(input.partner_type, input.ecti_income_type);
    let required = apply_rate(input.allocable_ecti_or_amount_realized_dollars, rate_bp);

    if input.partnership_actual_withheld_dollars == 0 && required > 0 {
        return Output {
            mode: Section1446Mode::ViolationPartnershipFailedToWithholdFromForeignPartner,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "§ 1446(a) — partnership is statutory withholding agent personally liable".to_string(),
            notes: format!(
                "VIOLATION § 1446(a): partnership failed to withhold on ${} of ECTI allocable to {:?}. Required at {} basis points = ${}. Partnership personally liable + § 6651/§ 6656 penalties + interest.",
                input.allocable_ecti_or_amount_realized_dollars,
                input.partner_type,
                rate_bp,
                required
            ),
            citations,
        };
    }

    if input.partnership_actual_withheld_dollars < required {
        return Output {
            mode: Section1446Mode::ViolationWithheldAtIncorrectRate,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "§ 1446(b) — actual withholding less than required rate".to_string(),
            notes: format!(
                "VIOLATION § 1446(b): partnership withheld ${} (less than required ${}) on ECTI ${}. Income type = {:?}; required rate = {} basis points. Shortfall = ${}.",
                input.partnership_actual_withheld_dollars,
                required,
                input.allocable_ecti_or_amount_realized_dollars,
                input.ecti_income_type,
                rate_bp,
                required.saturating_sub(input.partnership_actual_withheld_dollars)
            ),
            citations,
        };
    }

    if input.quarterly_installments_paid_count < 4 {
        return Output {
            mode: Section1446Mode::ViolationQuarterlyInstallmentMissed,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "§ 1446(e) — quarterly installment payment deadline missed".to_string(),
            notes: format!(
                "VIOLATION § 1446(e): partnership paid {} of 4 required quarterly installments (due 15th of 4th/6th/9th/12th months on Form 8813). Underpayment subject to § 6655-style estimated-tax penalty + interest.",
                input.quarterly_installments_paid_count
            ),
            citations,
        };
    }

    if !input.form_8804_filed_by_annual_deadline {
        return Output {
            mode: Section1446Mode::ViolationForm8804NotFiledAtAnnualDeadline,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "Form 8804 annual deadline missed (15th of 3rd month after close)".to_string(),
            notes: "VIOLATION: Form 8804 not filed by 15th day of 3rd month after close of partnership tax year. Late filing subject to § 6651(a)(1) failure-to-file penalty + interest.".to_string(),
            citations,
        };
    }

    if !input.form_8805_issued_to_each_foreign_partner {
        return Output {
            mode: Section1446Mode::ViolationForm8805NotIssuedToForeignPartner,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "Form 8805 not issued to foreign partner".to_string(),
            notes: "VIOLATION: Form 8805 (Foreign Partner's Information Statement) not issued to each foreign partner. Foreign partner cannot claim credit on Form 1040-NR / 1120-F without Form 8805. Partnership subject to § 6722 penalty.".to_string(),
            citations,
        };
    }

    Output {
        mode: Section1446Mode::CompliantQuarterlyInstallmentsPaidAndForm8804Filed,
        required_withholding_dollars: required,
        applicable_rate_basis_points: rate_bp,
        statutory_basis: format!(
            "§ 1446(b)(1) — {} basis points rate for {:?} on {:?} income",
            rate_bp, input.partner_type, input.ecti_income_type
        ),
        notes: format!(
            "COMPLIANT § 1446: ECTI = ${}; required withholding = ${}; actual withholding = ${}; 4 quarterly installments paid; Form 8804 filed timely; Form 8805 issued.",
            input.allocable_ecti_or_amount_realized_dollars, required, input.partnership_actual_withheld_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_noncorporate_foreign_partner_compliant() -> Input {
        Input {
            scenario: WithholdingScenario::AnnualEctiWithholding,
            partner_type: PartnerType::NoncorporateForeignPartner,
            ecti_income_type: EctiIncomeType::OrdinaryEcti,
            allocable_ecti_or_amount_realized_dollars: 1_000_000,
            partnership_actual_withheld_dollars: 370_000,
            quarterly_installments_paid_count: 4,
            form_8804_filed_by_annual_deadline: true,
            form_8805_issued_to_each_foreign_partner: true,
            form_8804_c_partner_level_certificate_filed: false,
        }
    }

    #[test]
    fn us_person_partner_not_applicable() {
        let input = Input {
            partner_type: PartnerType::UsPersonNotForeign,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::NotApplicableNoForeignPartnerOrNoEcti
        );
    }

    #[test]
    fn zero_ecti_not_applicable() {
        let input = Input {
            allocable_ecti_or_amount_realized_dollars: 0,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::NotApplicableNoForeignPartnerOrNoEcti
        );
    }

    #[test]
    fn noncorporate_foreign_37_pct_compliant() {
        let result = compute(&baseline_noncorporate_foreign_partner_compliant());
        assert_eq!(
            result.mode,
            Section1446Mode::CompliantQuarterlyInstallmentsPaidAndForm8804Filed
        );
        assert_eq!(result.required_withholding_dollars, 370_000);
        assert_eq!(result.applicable_rate_basis_points, 3_700);
    }

    #[test]
    fn corporate_foreign_21_pct_compliant() {
        let input = Input {
            partner_type: PartnerType::CorporateForeignPartner,
            partnership_actual_withheld_dollars: 210_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::CompliantQuarterlyInstallmentsPaidAndForm8804Filed
        );
        assert_eq!(result.required_withholding_dollars, 210_000);
        assert_eq!(result.applicable_rate_basis_points, 2_100);
    }

    #[test]
    fn noncorporate_ltcg_20_pct_rate() {
        let input = Input {
            ecti_income_type: EctiIncomeType::LongTermCapitalGain,
            partnership_actual_withheld_dollars: 200_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_rate_basis_points, 2_000);
        assert_eq!(result.required_withholding_dollars, 200_000);
    }

    #[test]
    fn noncorporate_unrecaptured_1250_25_pct_rate() {
        let input = Input {
            ecti_income_type: EctiIncomeType::UnrecapturedSection1250Gain,
            partnership_actual_withheld_dollars: 250_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_rate_basis_points, 2_500);
        assert_eq!(result.required_withholding_dollars, 250_000);
    }

    #[test]
    fn noncorporate_collectibles_28_pct_rate() {
        let input = Input {
            ecti_income_type: EctiIncomeType::CollectiblesGain,
            partnership_actual_withheld_dollars: 280_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.applicable_rate_basis_points, 2_800);
        assert_eq!(result.required_withholding_dollars, 280_000);
    }

    #[test]
    fn partnership_failed_to_withhold_violation() {
        let input = Input {
            partnership_actual_withheld_dollars: 0,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::ViolationPartnershipFailedToWithholdFromForeignPartner
        );
        assert_eq!(result.required_withholding_dollars, 370_000);
    }

    #[test]
    fn under_withheld_violation_with_shortfall_computed() {
        let input = Input {
            partnership_actual_withheld_dollars: 100_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::ViolationWithheldAtIncorrectRate
        );
        assert!(result.notes.contains("Shortfall = $270000"));
    }

    #[test]
    fn missed_quarterly_installments_violation() {
        let input = Input {
            quarterly_installments_paid_count: 2,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::ViolationQuarterlyInstallmentMissed
        );
        assert!(result.notes.contains("2 of 4"));
    }

    #[test]
    fn form_8804_not_filed_violation() {
        let input = Input {
            form_8804_filed_by_annual_deadline: false,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::ViolationForm8804NotFiledAtAnnualDeadline
        );
    }

    #[test]
    fn form_8805_not_issued_violation() {
        let input = Input {
            form_8805_issued_to_each_foreign_partner: false,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::ViolationForm8805NotIssuedToForeignPartner
        );
    }

    #[test]
    fn form_8804_c_partner_certificate_compliant() {
        let input = Input {
            form_8804_c_partner_level_certificate_filed: true,
            partnership_actual_withheld_dollars: 50_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::CompliantPartnerLevelCertificateFiledForm8804C
        );
        assert_eq!(result.required_withholding_dollars, 50_000);
    }

    #[test]
    fn section_1446f_transfer_10_pct_compliant() {
        let input = Input {
            scenario: WithholdingScenario::Section1446fPartnershipInterestTransfer,
            allocable_ecti_or_amount_realized_dollars: 2_000_000,
            partnership_actual_withheld_dollars: 200_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::CompliantSection1446fTenPercentTransferWithholdingApplied
        );
        assert_eq!(result.required_withholding_dollars, 200_000);
        assert_eq!(result.applicable_rate_basis_points, 1_000);
    }

    #[test]
    fn section_1446f_transferee_failed_to_withhold_violation() {
        let input = Input {
            scenario: WithholdingScenario::Section1446fPartnershipInterestTransfer,
            allocable_ecti_or_amount_realized_dollars: 2_000_000,
            partnership_actual_withheld_dollars: 50_000,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1446Mode::ViolationSection1446fTransferWithholdingFailed
        );
        assert!(result.notes.contains("shortfall $150000"));
    }

    #[test]
    fn citations_pin_section_1446_subsections_and_forms() {
        let result = compute(&baseline_noncorporate_foreign_partner_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1446(a)"));
        assert!(joined.contains("§ 1446(b)(1)"));
        assert!(joined.contains("§ 1446(c)"));
        assert!(joined.contains("§ 1446(d)"));
        assert!(joined.contains("§ 1446(e)"));
        assert!(joined.contains("§ 1446(f)"));
        assert!(joined.contains("TCJA 2017"));
        assert!(joined.contains("Form 8804"));
        assert!(joined.contains("Form 8805"));
        assert!(joined.contains("Form 8813"));
        assert!(joined.contains("Form 8804-C"));
        assert!(joined.contains("Treas. Reg. § 1.1446-3"));
        assert!(joined.contains("Treas. Reg. § 1.1446(f)-1"));
    }

    #[test]
    fn constant_pin_rates_and_installment_schedule() {
        assert_eq!(SECTION_1446_NONCORPORATE_RATE_BASIS_POINTS, 3_700);
        assert_eq!(SECTION_1446_CORPORATE_RATE_BASIS_POINTS, 2_100);
        assert_eq!(SECTION_1446_LTCG_RATE_BASIS_POINTS, 2_000);
        assert_eq!(SECTION_1446_UNRECAPTURED_1250_RATE_BASIS_POINTS, 2_500);
        assert_eq!(SECTION_1446_COLLECTIBLES_RATE_BASIS_POINTS, 2_800);
        assert_eq!(SECTION_1446_F_TRANSFER_WITHHOLDING_RATE_BASIS_POINTS, 1_000);
        assert_eq!(SECTION_1446_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SECTION_1446_QUARTERLY_INSTALLMENT_MONTHS, [4, 6, 9, 12]);
        assert_eq!(SECTION_1446_QUARTERLY_INSTALLMENT_DAY, 15);
        assert_eq!(SECTION_1446_FORM_8804_ANNUAL_DEADLINE_MONTH, 3);
        assert_eq!(SECTION_1446_FORM_8804_ANNUAL_DEADLINE_DAY, 15);
    }

    #[test]
    fn constant_pin_tcja_2017_and_final_regs_2020() {
        assert_eq!(SECTION_1446_F_TCJA_ENACTMENT_YEAR, 2017);
        assert_eq!(SECTION_1446_F_FINAL_REGS_YEAR, 2020);
    }

    #[test]
    fn saturating_overflow_defense_extreme_ecti() {
        let input = Input {
            allocable_ecti_or_amount_realized_dollars: u64::MAX,
            partnership_actual_withheld_dollars: u64::MAX,
            ..baseline_noncorporate_foreign_partner_compliant()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section1446Mode::CompliantQuarterlyInstallmentsPaidAndForm8804Filed
                | Section1446Mode::ViolationWithheldAtIncorrectRate
        ));
    }
}
