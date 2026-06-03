//! NY Real Property Law § 235-e Duty to Provide Written Receipt and
//! Five-Day Late Rent Notice compliance for trader-landlords with
//! NY State rental inventory (residential and commercial).
//!
//! NY RPL § 235-e establishes three distinct landlord obligations:
//! (1) a written receipt requirement for cash and non-personal-check
//! rent payments; (2) a 3-year recordkeeping requirement for cash
//! payment records; and (3) a five-day late rent notice requirement
//! before commencing nonpayment proceedings (added by HSTPA 2019).
//!
//! **§ 235-e(a) written receipt requirement**: upon receipt of
//! rent payment in cash OR any instrument other than the personal
//! check of the lessee, lessor (or agent) must provide tenant with
//! a written receipt containing FIVE required elements:
//!
//! 1. **Date** of payment
//! 2. **Amount** of payment
//! 3. **Identity** of the premises
//! 4. **Period** for which paid
//! 5. **Signature and title** of person receiving the rent
//!
//! **Timing**:
//!
//! - Payment personally transmitted to lessor or agent: receipt
//!   issued **IMMEDIATELY**.
//! - Payment transmitted indirectly (mail, drop box, etc.):
//!   receipt within **15 DAYS** of lessor's or agent's receipt.
//!
//! **§ 235-e(b) recordkeeping**: lessor must maintain records of
//! cash payments for at least **3 years**.
//!
//! **§ 235-e(d) five-day late rent notice** (HSTPA 2019
//! amendment — Laws of 2019, c. 36): if landlord fails to receive
//! rent payment within **5 days** of the date specified in the
//! lease agreement, landlord must send tenant a written notice by
//! **CERTIFIED MAIL** stating the failure to receive rent payment.
//! Email and text messages are NOT sufficient — even if
//! acknowledged by the tenant. Applies to BOTH residential and
//! commercial tenancies.
//!
//! **§ 235-e(d) affirmative defense**: failure of lessor or agent
//! to send the five-day late notice may be used by the tenant as
//! an AFFIRMATIVE DEFENSE in any eviction proceeding based on
//! non-payment of rent. Tenant raising this defense need not prove
//! actual prejudice — the failure itself is the defense.
//!
//! **HSTPA 2019** — Housing Stability and Tenant Protection Act of
//! 2019 (Laws of 2019, c. 36) — overhauled NY landlord-tenant law
//! and added the § 235-e(d) certified-mail late-notice requirement
//! along with numerous other tenant protections.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const HSTPA_2019_ENACTMENT_YEAR: u32 = 2019;
#[allow(dead_code)]
pub const LATE_NOTICE_DAYS_AFTER_DUE_DATE: u32 = 5;
#[allow(dead_code)]
pub const INDIRECT_PAYMENT_RECEIPT_DAYS: u32 = 15;
#[allow(dead_code)]
pub const CASH_PAYMENT_RECORDS_RETENTION_YEARS: u32 = 3;
#[allow(dead_code)]
pub const REQUIRED_RECEIPT_FIELDS_COUNT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    NonPersonalCheckInstrument,
    PersonalCheck,
    ElectronicTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    Residential,
    Commercial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantCashReceiptProvidedImmediately,
    CompliantIndirectPaymentReceiptWithin15Days,
    CompliantFiveDayLateNoticeSentByCertifiedMail,
    CompliantAllReceiptAndNoticeObligationsMet,
    ViolationCashReceiptNotProvidedImmediately,
    ViolationIndirectPaymentReceiptNotProvidedWithin15Days,
    ViolationReceiptMissingRequiredElements,
    ViolationFiveDayLateNoticeNotSentByCertifiedMail,
    ViolationFiveDayLateNoticeSentByEmailOrTextOnlyNotSufficient,
    ViolationRecordsNotMaintained3Years,
    AggravatedViolationAffirmativeDefenseAvailableToTenantInNonpaymentProceeding,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub payment_method: PaymentMethod,
    pub payment_received_personally: bool,
    pub receipt_provided_immediately: bool,
    pub days_to_provide_receipt_after_indirect_receipt: u32,
    pub receipt_contains_all_required_fields: bool,
    pub days_late_rent_not_received: u32,
    pub five_day_late_notice_sent_by_certified_mail: bool,
    pub five_day_late_notice_sent_by_email_or_text_only: bool,
    pub cash_payment_records_retained_years: u32,
    pub nonpayment_proceeding_commenced: bool,
    pub tenancy_type: TenancyType,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub tenant_affirmative_defense_available: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type NyRentReceiptLateNoticeRequirementsInput = Input;
pub type NyRentReceiptLateNoticeRequirementsOutput = Output;
pub type NyRentReceiptLateNoticeRequirementsResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NY Real Property Law § 235-e (Duty to provide a written receipt)".to_string(),
        "NY RPL § 235-e(a) (written receipt — five required elements)".to_string(),
        "NY RPL § 235-e(b) (cash payment recordkeeping — 3 years)".to_string(),
        "NY RPL § 235-e(d) (HSTPA 2019 — 5-day late rent notice by certified mail)".to_string(),
        "NY Housing Stability and Tenant Protection Act of 2019 (HSTPA, Laws of 2019, c. 36)".to_string(),
        "NY RPAPL § 711 (nonpayment proceeding — affirmative defense)".to_string(),
        "NY State Senate — RPL § 235-E Duty to provide a written receipt".to_string(),
        "NY State Bar Association — HSTPA Part III guidance".to_string(),
    ];

    let requires_receipt = matches!(
        input.payment_method,
        PaymentMethod::Cash | PaymentMethod::NonPersonalCheckInstrument
    );

    if input.cash_payment_records_retained_years < CASH_PAYMENT_RECORDS_RETENTION_YEARS
        && matches!(input.payment_method, PaymentMethod::Cash)
    {
        notes.push(format!(
            "Cash payment records retained only {} years (< {}-year statutory minimum) — § 235-e(b) violation.",
            input.cash_payment_records_retained_years,
            CASH_PAYMENT_RECORDS_RETENTION_YEARS
        ));
        return Output {
            severity: Severity::ViolationRecordsNotMaintained3Years,
            compliant: false,
            tenant_affirmative_defense_available: false,
            notes,
            citations,
        };
    }

    if requires_receipt && !input.receipt_contains_all_required_fields {
        notes.push(format!(
            "Receipt missing one or more of {} required elements (date + amount + premises + period + signature) — § 235-e(a) violation.",
            REQUIRED_RECEIPT_FIELDS_COUNT
        ));
        return Output {
            severity: Severity::ViolationReceiptMissingRequiredElements,
            compliant: false,
            tenant_affirmative_defense_available: false,
            notes,
            citations,
        };
    }

    if requires_receipt
        && input.payment_received_personally
        && !input.receipt_provided_immediately
    {
        notes.push("Personally-transmitted rent payment — receipt not provided immediately as required by § 235-e(a).".to_string());
        return Output {
            severity: Severity::ViolationCashReceiptNotProvidedImmediately,
            compliant: false,
            tenant_affirmative_defense_available: false,
            notes,
            citations,
        };
    }

    if requires_receipt
        && !input.payment_received_personally
        && input.days_to_provide_receipt_after_indirect_receipt > INDIRECT_PAYMENT_RECEIPT_DAYS
    {
        notes.push(format!(
            "Indirectly-transmitted rent payment — receipt provided {} days after receipt (> {}-day deadline) — § 235-e(a) violation.",
            input.days_to_provide_receipt_after_indirect_receipt,
            INDIRECT_PAYMENT_RECEIPT_DAYS
        ));
        return Output {
            severity: Severity::ViolationIndirectPaymentReceiptNotProvidedWithin15Days,
            compliant: false,
            tenant_affirmative_defense_available: false,
            notes,
            citations,
        };
    }

    if input.days_late_rent_not_received > LATE_NOTICE_DAYS_AFTER_DUE_DATE {
        if input.five_day_late_notice_sent_by_email_or_text_only {
            notes.push("Five-day late notice sent by email or text only — § 235-e(d) requires CERTIFIED MAIL; email/text not sufficient even if acknowledged.".to_string());
            return Output {
                severity: Severity::ViolationFiveDayLateNoticeSentByEmailOrTextOnlyNotSufficient,
                compliant: false,
                tenant_affirmative_defense_available: input.nonpayment_proceeding_commenced,
                notes,
                citations,
            };
        }
        if !input.five_day_late_notice_sent_by_certified_mail {
            notes.push(format!(
                "Rent not received within {} days of due date and certified-mail late notice not sent — § 235-e(d) violation; tenant entitled to affirmative defense.",
                LATE_NOTICE_DAYS_AFTER_DUE_DATE
            ));
            let severity = if input.nonpayment_proceeding_commenced {
                Severity::AggravatedViolationAffirmativeDefenseAvailableToTenantInNonpaymentProceeding
            } else {
                Severity::ViolationFiveDayLateNoticeNotSentByCertifiedMail
            };
            return Output {
                severity,
                compliant: false,
                tenant_affirmative_defense_available: input.nonpayment_proceeding_commenced,
                notes,
                citations,
            };
        }
    }

    notes.push(format!(
        "Full § 235-e compliance: receipt provided per timing rules ({}/{}-day), 5-day late notice sent by certified mail (if triggered), records retained {} years.",
        if input.payment_received_personally { "immediate" } else { "15" },
        if input.payment_received_personally { "immediate" } else { "day" },
        CASH_PAYMENT_RECORDS_RETENTION_YEARS
    ));
    Output {
        severity: Severity::CompliantAllReceiptAndNoticeObligationsMet,
        compliant: true,
        tenant_affirmative_defense_available: false,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant_cash_personal() -> Input {
        Input {
            payment_method: PaymentMethod::Cash,
            payment_received_personally: true,
            receipt_provided_immediately: true,
            days_to_provide_receipt_after_indirect_receipt: 0,
            receipt_contains_all_required_fields: true,
            days_late_rent_not_received: 0,
            five_day_late_notice_sent_by_certified_mail: false,
            five_day_late_notice_sent_by_email_or_text_only: false,
            cash_payment_records_retained_years: 3,
            nonpayment_proceeding_commenced: false,
            tenancy_type: TenancyType::Residential,
        }
    }

    #[test]
    fn cash_personal_immediate_receipt_compliant() {
        let out = check(&base_compliant_cash_personal());
        assert_eq!(
            out.severity,
            Severity::CompliantAllReceiptAndNoticeObligationsMet
        );
        assert!(out.compliant);
    }

    #[test]
    fn cash_personal_receipt_not_immediate_violation() {
        let mut i = base_compliant_cash_personal();
        i.receipt_provided_immediately = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationCashReceiptNotProvidedImmediately
        );
    }

    #[test]
    fn indirect_payment_receipt_within_15_days_compliant() {
        let mut i = base_compliant_cash_personal();
        i.payment_received_personally = false;
        i.days_to_provide_receipt_after_indirect_receipt = 10;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn indirect_payment_receipt_at_exactly_15_days_compliant() {
        let mut i = base_compliant_cash_personal();
        i.payment_received_personally = false;
        i.days_to_provide_receipt_after_indirect_receipt = 15;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn indirect_payment_receipt_16_days_violation() {
        let mut i = base_compliant_cash_personal();
        i.payment_received_personally = false;
        i.days_to_provide_receipt_after_indirect_receipt = 16;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationIndirectPaymentReceiptNotProvidedWithin15Days
        );
    }

    #[test]
    fn receipt_missing_required_elements_violation() {
        let mut i = base_compliant_cash_personal();
        i.receipt_contains_all_required_fields = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationReceiptMissingRequiredElements
        );
    }

    #[test]
    fn cash_records_not_3_years_violation() {
        let mut i = base_compliant_cash_personal();
        i.cash_payment_records_retained_years = 2;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationRecordsNotMaintained3Years
        );
    }

    #[test]
    fn five_day_late_notice_sent_by_certified_mail_compliant() {
        let mut i = base_compliant_cash_personal();
        i.days_late_rent_not_received = 10;
        i.five_day_late_notice_sent_by_certified_mail = true;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn five_day_late_notice_not_sent_violation() {
        let mut i = base_compliant_cash_personal();
        i.days_late_rent_not_received = 10;
        i.five_day_late_notice_sent_by_certified_mail = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFiveDayLateNoticeNotSentByCertifiedMail
        );
    }

    #[test]
    fn five_day_late_notice_email_only_not_sufficient_violation() {
        let mut i = base_compliant_cash_personal();
        i.days_late_rent_not_received = 10;
        i.five_day_late_notice_sent_by_email_or_text_only = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFiveDayLateNoticeSentByEmailOrTextOnlyNotSufficient
        );
    }

    #[test]
    fn aggravated_violation_when_nonpayment_proceeding_commenced() {
        let mut i = base_compliant_cash_personal();
        i.days_late_rent_not_received = 10;
        i.five_day_late_notice_sent_by_certified_mail = false;
        i.nonpayment_proceeding_commenced = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AggravatedViolationAffirmativeDefenseAvailableToTenantInNonpaymentProceeding
        );
        assert!(out.tenant_affirmative_defense_available);
    }

    #[test]
    fn personal_check_no_receipt_required() {
        let mut i = base_compliant_cash_personal();
        i.payment_method = PaymentMethod::PersonalCheck;
        i.receipt_provided_immediately = false;
        i.receipt_contains_all_required_fields = false;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn rent_within_5_days_no_late_notice_required() {
        let mut i = base_compliant_cash_personal();
        i.days_late_rent_not_received = 5;
        i.five_day_late_notice_sent_by_certified_mail = false;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn commercial_tenancy_5_day_notice_also_required() {
        let mut i = base_compliant_cash_personal();
        i.tenancy_type = TenancyType::Commercial;
        i.days_late_rent_not_received = 10;
        i.five_day_late_notice_sent_by_certified_mail = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFiveDayLateNoticeNotSentByCertifiedMail
        );
    }

    #[test]
    fn citations_pin_rpl_235e_subsections() {
        let out = check(&base_compliant_cash_personal());
        assert!(out.citations.iter().any(|c| c.contains("§ 235-e")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-e(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-e(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-e(d)")));
    }

    #[test]
    fn citations_pin_hstpa_2019_and_rpapl_711() {
        let out = check(&base_compliant_cash_personal());
        assert!(out.citations.iter().any(|c| c.contains("HSTPA")));
        assert!(out.citations.iter().any(|c| c.contains("Laws of 2019")));
        assert!(out.citations.iter().any(|c| c.contains("RPAPL § 711")));
    }

    #[test]
    fn constant_pin_5_day_late_notice() {
        assert_eq!(LATE_NOTICE_DAYS_AFTER_DUE_DATE, 5);
    }

    #[test]
    fn constant_pin_15_day_indirect_payment_receipt() {
        assert_eq!(INDIRECT_PAYMENT_RECEIPT_DAYS, 15);
    }

    #[test]
    fn constant_pin_3_year_records_retention() {
        assert_eq!(CASH_PAYMENT_RECORDS_RETENTION_YEARS, 3);
    }

    #[test]
    fn constant_pin_5_required_receipt_fields() {
        assert_eq!(REQUIRED_RECEIPT_FIELDS_COUNT, 5);
    }

    #[test]
    fn constant_pin_hstpa_2019_year() {
        assert_eq!(HSTPA_2019_ENACTMENT_YEAR, 2019);
    }
}
