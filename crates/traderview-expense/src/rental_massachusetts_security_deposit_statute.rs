//! Massachusetts Security Deposit Statute compliance for trader-
//! landlords with MA rental inventory.
//!
//! Massachusetts General Laws Chapter 186, Section 15B is widely
//! regarded as the most stringent security deposit statute in the
//! United States. Failure to comply with ANY procedural requirement
//! exposes the landlord to **TRIPLE DAMAGES** (three times the
//! deposit amount) plus court costs and reasonable attorney fees,
//! AND additional liability under MGL c. 93A (the Massachusetts
//! Consumer Protection Act).
//!
//! **§ 15B(1)(b) maximum deposit**: security deposit may not
//! exceed **ONE MONTH'S RENT** at commencement of tenancy.
//!
//! **§ 15B(2)(b) written receipt**: lessor must provide written
//! receipt within **30 days** of receipt listing amount received,
//! name and address of lessor, name of person receiving, date of
//! receipt, and description of leased premises.
//!
//! **§ 15B(2)(c) Statement of Condition** (also called Apartment
//! Condition Statement): within **10 days** after commencement of
//! tenancy OR on receipt of deposit (whichever is later), lessor
//! must furnish a written statement of the present condition of
//! the premises containing a comprehensive listing of any damage
//! then existing. Statement must (1) be signed by lessor or agent,
//! (2) contain a notice in **12-point bold-face type** at the top
//! of the first page, (3) tenant has **15 days** to return signed
//! statement to lessor. If tenant submits a separate list of
//! damages, lessor must reply within 15 days with agreement or
//! disagreement.
//!
//! **§ 15B(3)(a) separate Massachusetts bank account**: deposit
//! must be held in a separate **interest-bearing account in a bank
//! located within Massachusetts**, under terms placing the deposit
//! beyond claims of lessor's creditors.
//!
//! **§ 15B(3)(b) annual interest**: lessor holding deposit for one
//! year or longer must pay interest at the lesser of **5% per year**
//! or actual bank interest received. Interest paid annually on
//! tenancy anniversary; if tenancy terminates before anniversary,
//! accrued interest within **30 days** of termination.
//!
//! **§ 15B(4) return of deposit**: within **30 days** after
//! termination of occupancy, lessor must return deposit or balance
//! with itemized list of damages signed under penalty of perjury
//! and including written invoices, bills, receipts, or estimates
//! to repair the damage.
//!
//! **§ 15B(7) triple damages**: failure to (1) return deposit within
//! 30 days, (2) return interest within 30 days, (3) maintain
//! separate account, (4) disclose bank information, (5) comply with
//! statement of condition procedure — results in damages equal to
//! **THREE TIMES** the deposit amount plus court costs and
//! reasonable attorney fees. Additional MGL c. 93A liability may
//! double the recovery again for unfair or deceptive practices.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const MA_SECURITY_DEPOSIT_MAX_MONTHS_RENT: u32 = 1;
#[allow(dead_code)]
pub const MA_INTEREST_RATE_PERCENT_X_100: u32 = 500;
#[allow(dead_code)]
pub const MA_DEPOSIT_RECEIPT_DAYS: u32 = 30;
#[allow(dead_code)]
pub const MA_STATEMENT_OF_CONDITION_DAYS: u32 = 10;
#[allow(dead_code)]
pub const MA_TENANT_STATEMENT_RESPONSE_DAYS: u32 = 15;
#[allow(dead_code)]
pub const MA_LESSOR_REPLY_DAYS: u32 = 15;
#[allow(dead_code)]
pub const MA_DEPOSIT_RETURN_DAYS_AFTER_TERMINATION: u32 = 30;
#[allow(dead_code)]
pub const MA_INTEREST_PAYMENT_DAYS_AFTER_TERMINATION: u32 = 30;
#[allow(dead_code)]
pub const MA_TRIPLE_DAMAGES_MULTIPLIER: u64 = 3;
#[allow(dead_code)]
pub const STATEMENT_OF_CONDITION_BOLD_FACE_FONT_SIZE_POINT: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantFullStatutoryCompliance,
    ViolationDepositExceedsOneMonthRent,
    ViolationDepositReceiptNotProvidedWithin30Days,
    ViolationDepositNotInSeparateMassBankAccount,
    ViolationNoStatementOfConditionWithin10Days,
    ViolationStatementOfConditionMissing12PtBoldNotice,
    ViolationInterestNotPaidAnnually,
    ViolationDepositNotReturnedWithin30DaysOfTermination,
    ViolationItemizedDamagesListMissingOrUnsignedPerjury,
    AggravatedTripleDamagesUnderSection15B7,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub security_deposit_cents: u64,
    pub monthly_rent_cents: u64,
    pub deposit_in_separate_ma_bank_account: bool,
    pub statement_of_condition_provided_within_10_days: bool,
    pub statement_in_12_pt_bold_face_top_of_first_page: bool,
    pub written_receipt_provided_within_30_days: bool,
    pub annual_interest_paid_or_credited: bool,
    pub days_since_tenancy_termination: u32,
    pub deposit_returned_after_termination: bool,
    pub deposit_being_retained: bool,
    pub itemized_damages_list_provided: bool,
    pub itemized_damages_signed_under_penalty_of_perjury: bool,
    pub include_aggregated_triple_damages_exposure: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub max_allowed_deposit_cents: u64,
    pub annual_interest_owed_cents: u64,
    pub triple_damages_exposure_cents: u64,
    pub compliant: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type MassachusettsSecurityDepositInput = Input;
pub type MassachusettsSecurityDepositOutput = Output;
pub type MassachusettsSecurityDepositResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "MGL c. 186, § 15B (Massachusetts Security Deposit Statute)".to_string(),
        "MGL c. 186, § 15B(1)(b) (one-month-rent maximum)".to_string(),
        "MGL c. 186, § 15B(2)(b) (written receipt 30-day requirement)".to_string(),
        "MGL c. 186, § 15B(2)(c) (Statement of Condition — 10 days + 12-pt bold notice)"
            .to_string(),
        "MGL c. 186, § 15B(3)(a) (separate MA bank account requirement)".to_string(),
        "MGL c. 186, § 15B(3)(b) (5% annual interest or actual lesser)".to_string(),
        "MGL c. 186, § 15B(4) (30-day return + itemized damages under penalty of perjury)"
            .to_string(),
        "MGL c. 186, § 15B(7) (triple damages + court costs + attorney fees)".to_string(),
        "MGL c. 93A (Massachusetts Consumer Protection Act — secondary basis for recovery)"
            .to_string(),
        "Mass.gov — Massachusetts law about tenants' security deposits".to_string(),
        "MassLegalHelp — Chapter 3 Security Deposits & Last Month's Rent".to_string(),
    ];

    let max_allowed = input.monthly_rent_cents;
    let annual_interest = input
        .security_deposit_cents
        .saturating_mul(MA_INTEREST_RATE_PERCENT_X_100 as u64)
        / 10_000;
    let triple_exposure = input
        .security_deposit_cents
        .saturating_mul(MA_TRIPLE_DAMAGES_MULTIPLIER);

    if input.security_deposit_cents > max_allowed {
        notes.push(format!(
            "Deposit ${} exceeds one month's rent ${} — § 15B(1)(b) violation; triple damages exposure ${}.",
            input.security_deposit_cents / 100,
            max_allowed / 100,
            triple_exposure / 100
        ));
        return Output {
            severity: Severity::ViolationDepositExceedsOneMonthRent,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if !input.written_receipt_provided_within_30_days {
        notes.push(format!(
            "Written receipt not provided within {} days — § 15B(2)(b) violation.",
            MA_DEPOSIT_RECEIPT_DAYS
        ));
        return Output {
            severity: Severity::ViolationDepositReceiptNotProvidedWithin30Days,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if !input.deposit_in_separate_ma_bank_account {
        notes.push("Deposit not held in separate interest-bearing Massachusetts bank account — § 15B(3)(a) violation; triple damages exposure.".to_string());
        return Output {
            severity: Severity::ViolationDepositNotInSeparateMassBankAccount,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if !input.statement_of_condition_provided_within_10_days {
        notes.push(format!(
            "Statement of Condition not provided within {} days of tenancy commencement — § 15B(2)(c) violation.",
            MA_STATEMENT_OF_CONDITION_DAYS
        ));
        return Output {
            severity: Severity::ViolationNoStatementOfConditionWithin10Days,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if !input.statement_in_12_pt_bold_face_top_of_first_page {
        notes.push(format!(
            "Statement of Condition missing {}-point bold-face notice at top of first page — § 15B(2)(c) violation.",
            STATEMENT_OF_CONDITION_BOLD_FACE_FONT_SIZE_POINT
        ));
        return Output {
            severity: Severity::ViolationStatementOfConditionMissing12PtBoldNotice,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if !input.annual_interest_paid_or_credited {
        notes.push("Annual interest at 5% (or lesser actual) not paid or credited — § 15B(3)(b) violation.".to_string());
        return Output {
            severity: Severity::ViolationInterestNotPaidAnnually,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if input.days_since_tenancy_termination > MA_DEPOSIT_RETURN_DAYS_AFTER_TERMINATION
        && !input.deposit_returned_after_termination
    {
        notes.push(format!(
            "Deposit not returned within {} days of tenancy termination ({} days elapsed) — § 15B(4) violation; triple damages exposure ${}.",
            MA_DEPOSIT_RETURN_DAYS_AFTER_TERMINATION,
            input.days_since_tenancy_termination,
            triple_exposure / 100
        ));
        let severity = if input.include_aggregated_triple_damages_exposure {
            Severity::AggravatedTripleDamagesUnderSection15B7
        } else {
            Severity::ViolationDepositNotReturnedWithin30DaysOfTermination
        };
        return Output {
            severity,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    if input.deposit_being_retained
        && (!input.itemized_damages_list_provided
            || !input.itemized_damages_signed_under_penalty_of_perjury)
    {
        notes.push("Lessor retaining deposit but missing itemized damages list or unsigned under penalty of perjury — § 15B(4) violation.".to_string());
        return Output {
            severity: Severity::ViolationItemizedDamagesListMissingOrUnsignedPerjury,
            max_allowed_deposit_cents: max_allowed,
            annual_interest_owed_cents: annual_interest,
            triple_damages_exposure_cents: triple_exposure,
            compliant: false,
            notes,
            citations,
        };
    }

    notes.push("Full MGL c. 186, § 15B compliance: 1-month-rent max + receipt + separate MA bank account + Statement of Condition + annual interest + 30-day return + itemized damages (if retained).".to_string());
    Output {
        severity: Severity::CompliantFullStatutoryCompliance,
        max_allowed_deposit_cents: max_allowed,
        annual_interest_owed_cents: annual_interest,
        triple_damages_exposure_cents: 0,
        compliant: true,
        notes,
        citations,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn base_compliant() -> Input {
        Input {
            security_deposit_cents: 250_000,
            monthly_rent_cents: 250_000,
            deposit_in_separate_ma_bank_account: true,
            statement_of_condition_provided_within_10_days: true,
            statement_in_12_pt_bold_face_top_of_first_page: true,
            written_receipt_provided_within_30_days: true,
            annual_interest_paid_or_credited: true,
            days_since_tenancy_termination: 15,
            deposit_returned_after_termination: true,
            deposit_being_retained: false,
            itemized_damages_list_provided: false,
            itemized_damages_signed_under_penalty_of_perjury: false,
            include_aggregated_triple_damages_exposure: false,
        }
    }

    #[test]
    fn fully_compliant_baseline() {
        let out = check(&base_compliant());
        assert_eq!(out.severity, Severity::CompliantFullStatutoryCompliance);
        assert!(out.compliant);
        assert_eq!(out.triple_damages_exposure_cents, 0);
    }

    #[test]
    fn annual_interest_at_5_pct_baseline_2500() {
        let out = check(&base_compliant());
        assert_eq!(out.annual_interest_owed_cents, 12_500);
    }

    #[test]
    fn deposit_exceeds_one_month_rent_violation() {
        let mut i = base_compliant();
        i.security_deposit_cents = 300_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationDepositExceedsOneMonthRent);
        assert_eq!(out.triple_damages_exposure_cents, 900_000);
    }

    #[test]
    fn deposit_at_exactly_one_month_rent_compliant() {
        let out = check(&base_compliant());
        assert!(out.compliant);
    }

    #[test]
    fn written_receipt_not_provided_violation() {
        let mut i = base_compliant();
        i.written_receipt_provided_within_30_days = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationDepositReceiptNotProvidedWithin30Days
        );
    }

    #[test]
    fn deposit_not_in_ma_bank_account_violation() {
        let mut i = base_compliant();
        i.deposit_in_separate_ma_bank_account = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationDepositNotInSeparateMassBankAccount
        );
    }

    #[test]
    fn no_statement_of_condition_within_10_days_violation() {
        let mut i = base_compliant();
        i.statement_of_condition_provided_within_10_days = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationNoStatementOfConditionWithin10Days
        );
    }

    #[test]
    fn statement_missing_12_pt_bold_face_violation() {
        let mut i = base_compliant();
        i.statement_in_12_pt_bold_face_top_of_first_page = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationStatementOfConditionMissing12PtBoldNotice
        );
    }

    #[test]
    fn annual_interest_not_paid_violation() {
        let mut i = base_compliant();
        i.annual_interest_paid_or_credited = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationInterestNotPaidAnnually);
    }

    #[test]
    fn deposit_not_returned_within_30_days_violation() {
        let mut i = base_compliant();
        i.days_since_tenancy_termination = 45;
        i.deposit_returned_after_termination = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationDepositNotReturnedWithin30DaysOfTermination
        );
        assert_eq!(out.triple_damages_exposure_cents, 750_000);
    }

    #[test]
    fn deposit_at_exactly_30_days_still_compliant() {
        let mut i = base_compliant();
        i.days_since_tenancy_termination = 30;
        i.deposit_returned_after_termination = false;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn deposit_at_31_days_not_returned_violation() {
        let mut i = base_compliant();
        i.days_since_tenancy_termination = 31;
        i.deposit_returned_after_termination = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationDepositNotReturnedWithin30DaysOfTermination
        );
    }

    #[test]
    fn aggravated_triple_damages_under_15B7() {
        let mut i = base_compliant();
        i.days_since_tenancy_termination = 60;
        i.deposit_returned_after_termination = false;
        i.include_aggregated_triple_damages_exposure = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AggravatedTripleDamagesUnderSection15B7
        );
        assert_eq!(out.triple_damages_exposure_cents, 750_000);
    }

    #[test]
    fn deposit_retained_without_itemized_damages_violation() {
        let mut i = base_compliant();
        i.deposit_being_retained = true;
        i.itemized_damages_list_provided = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationItemizedDamagesListMissingOrUnsignedPerjury
        );
    }

    #[test]
    fn deposit_retained_without_perjury_signature_violation() {
        let mut i = base_compliant();
        i.deposit_being_retained = true;
        i.itemized_damages_list_provided = true;
        i.itemized_damages_signed_under_penalty_of_perjury = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationItemizedDamagesListMissingOrUnsignedPerjury
        );
    }

    #[test]
    fn deposit_retained_with_proper_itemized_damages_compliant() {
        let mut i = base_compliant();
        i.deposit_being_retained = true;
        i.itemized_damages_list_provided = true;
        i.itemized_damages_signed_under_penalty_of_perjury = true;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn citations_pin_mgl_c186_15b_subsections() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(1)(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(2)(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(2)(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(3)(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(3)(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(4)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 15B(7)")));
    }

    #[test]
    fn citations_pin_mgl_c93a_consumer_protection() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("c. 93A")));
    }

    #[test]
    fn constant_pin_1_month_rent_maximum() {
        assert_eq!(MA_SECURITY_DEPOSIT_MAX_MONTHS_RENT, 1);
    }

    #[test]
    fn constant_pin_5_percent_interest_rate_x_100() {
        assert_eq!(MA_INTEREST_RATE_PERCENT_X_100, 500);
    }

    #[test]
    fn constant_pin_30_day_receipt() {
        assert_eq!(MA_DEPOSIT_RECEIPT_DAYS, 30);
    }

    #[test]
    fn constant_pin_10_day_statement_of_condition() {
        assert_eq!(MA_STATEMENT_OF_CONDITION_DAYS, 10);
    }

    #[test]
    fn constant_pin_30_day_return_after_termination() {
        assert_eq!(MA_DEPOSIT_RETURN_DAYS_AFTER_TERMINATION, 30);
    }

    #[test]
    fn constant_pin_triple_damages_multiplier_3x() {
        assert_eq!(MA_TRIPLE_DAMAGES_MULTIPLIER, 3);
    }

    #[test]
    fn constant_pin_12_point_bold_face_font_size() {
        assert_eq!(STATEMENT_OF_CONDITION_BOLD_FACE_FONT_SIZE_POINT, 12);
    }

    #[test]
    fn very_large_deposit_saturating_no_overflow() {
        let mut i = base_compliant();
        i.security_deposit_cents = u64::MAX;
        i.monthly_rent_cents = u64::MAX;
        let out = check(&i);
        assert!(out.annual_interest_owed_cents > 0);
    }
}
