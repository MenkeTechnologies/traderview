//! Multi-jurisdictional tenant rent receipt requirement
//! framework. Trader-landlord critical because (1) cash
//! rent receipts are mandatory in many states regardless
//! of tenant request; (2) receipt-issuance failures
//! create per-violation civil exposure plus an evidentiary
//! presumption against the landlord in rent-payment
//! disputes; (3) record retention obligations (3-year NY,
//! similar elsewhere) extend long beyond tenancy
//! termination; (4) modern payment methods (Zelle, Venmo,
//! ACH) require careful receipt practices to satisfy
//! state mandates. Companion to landlord_annual_rent_
//! statement, tenant_late_fee_cap, tenant_positive_rent_
//! reporting, rental_junk_fee_transparency.
//!
//! **New York Real Property Law § 235-e** — landlord must
//! provide written receipt when rent paid by:
//! 1. CASH;
//! 2. MONEY ORDER;
//! 3. CASHIER'S CHECK; OR
//! 4. ANY FORM OTHER THAN tenant's personal check.
//!
//! Receipt content requirements:
//! 1. PAYMENT DATE;
//! 2. AMOUNT received;
//! 3. PERIOD for which rent was paid;
//! 4. APARTMENT NUMBER;
//! 5. SIGNATURE of person receiving payment;
//! 6. TITLE of person receiving payment.
//!
//! Timing:
//! 1. IN-PERSON payment by cash or money order — IMMEDIATE
//!    receipt at time of payment.
//! 2. NON-IN-PERSON payment — receipt within **15 DAYS**.
//!
//! Personal check receipt — tenant may REQUEST in writing
//! a rent receipt for personal-check payment; after first
//! request, landlord must provide receipt EVERY MONTH
//! thereafter.
//!
//! **3-YEAR RECORD RETENTION** — landlord must keep proof
//! of cash rent receipts for **3 YEARS**.
//!
//! **California Civil Code § 1499** — receipt MANDATORY
//! upon tenant request:
//! 1. Receipt must be SIGNED AND DATED;
//! 2. Receipt must be provided at TIME OF PAYMENT (not
//!    year-end / not in lump sum at tax time);
//! 3. Applies to ALL payment methods (cash, check, money
//!    order, electronic) when requested.
//!
//! **Massachusetts G.L. c. 186 § 15B** — limited mandate:
//! 1. LAST MONTH'S RENT at commencement of tenancy:
//!    landlord MUST provide signed receipt;
//! 2. REGULAR MONTHLY RENT: NOT required (no statutory
//!    mandate even for cash payments);
//! 3. Receipt content (when required): amount paid, date,
//!    description of what payment was for, landlord's
//!    name, tenant's name, name of person to whom
//!    payment was given.
//!
//! **Washington RCW 59.18.063** — cash payment mandatory
//! receipt requirement:
//! 1. CASH payments — landlord MUST provide receipt;
//! 2. NON-CASH payments — receipt on tenant request.
//!
//! **Default** — no statewide mandate; common-law
//! payment-of-rent dispute defense; tenant bears burden
//! of proving payment (unless receipt requested and
//! refused). Some local ordinances (Chicago RLTO, San
//! Francisco) impose receipt requirements not present
//! at state level.
//!
//! Citations: N.Y. Real Prop. Law § 235-e; Cal. Civ. Code
//! § 1499; Mass. G.L. c. 186 § 15B; Wash. Rev. Code
//! § 59.18.063; Chicago RLTO § 5-12-080(g).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYork,
    California,
    Massachusetts,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    MoneyOrder,
    CashiersCheck,
    PersonalCheck,
    ElectronicAchOrZelle,
    LastMonthsRentAtCommencement,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantRentReceiptRequirementInput {
    pub jurisdiction: Jurisdiction,
    pub payment_method: PaymentMethod,
    /// Whether payment made in person.
    pub in_person_payment: bool,
    /// Whether tenant requested receipt (CA + WA non-cash
    /// trigger).
    pub tenant_requested_receipt: bool,
    /// Whether landlord provided receipt.
    pub receipt_provided: bool,
    /// Days since payment if receipt not yet provided
    /// (NY 15-day non-in-person window).
    pub days_since_payment: u32,
    /// Whether receipt contains all required content
    /// elements (date + amount + period + apartment +
    /// signature + title).
    pub receipt_content_complete: bool,
    /// Whether landlord retains cash rent receipt records
    /// for 3 years (NY § 235-e requirement).
    pub three_year_record_retention_maintained: bool,
    /// Whether receipt provided at time of payment (CA
    /// § 1499) vs lump-sum year-end.
    pub receipt_at_time_of_payment: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantRentReceiptRequirementResult {
    pub jurisdiction: Jurisdiction,
    pub receipt_obligation_triggered: bool,
    pub receipt_compliant: bool,
    pub timing_compliant: bool,
    pub content_compliant: bool,
    pub retention_compliant: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantRentReceiptRequirementInput) -> TenantRentReceiptRequirementResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (receipt_obligation_triggered, timing_compliant, content_compliant, retention_compliant) =
        match input.jurisdiction {
            Jurisdiction::NewYork => {
                let non_personal_check =
                    !matches!(input.payment_method, PaymentMethod::PersonalCheck);
                let obligation_triggered = non_personal_check
                    || (matches!(input.payment_method, PaymentMethod::PersonalCheck)
                        && input.tenant_requested_receipt);
                let timing = !obligation_triggered
                    || input.receipt_provided
                    || (!input.in_person_payment && input.days_since_payment <= 15);
                let content = !obligation_triggered
                    || !input.receipt_provided
                    || input.receipt_content_complete;
                let retention = !matches!(input.payment_method, PaymentMethod::Cash)
                    || input.three_year_record_retention_maintained;

                if obligation_triggered && input.in_person_payment && !input.receipt_provided {
                    failure_reasons.push(
                        "N.Y. Real Prop. Law § 235-e — when rent paid IN PERSON by CASH or MONEY ORDER, tenant is entitled to IMMEDIATE WRITTEN RECEIPT at time of payment".to_string(),
                    );
                }
                if obligation_triggered
                    && !input.in_person_payment
                    && !input.receipt_provided
                    && input.days_since_payment > 15
                {
                    failure_reasons.push(format!(
                        "N.Y. Real Prop. Law § 235-e — NON-IN-PERSON payment requires written receipt within 15 DAYS; {} days elapsed without receipt",
                        input.days_since_payment
                    ));
                }
                if obligation_triggered && input.receipt_provided && !input.receipt_content_complete
                {
                    failure_reasons.push(
                        "N.Y. Real Prop. Law § 235-e — required receipt content: (1) PAYMENT DATE; (2) AMOUNT; (3) PERIOD for which rent was paid; (4) APARTMENT NUMBER; (5) SIGNATURE of person receiving payment; (6) TITLE of person receiving payment".to_string(),
                    );
                }
                if matches!(input.payment_method, PaymentMethod::Cash)
                    && !input.three_year_record_retention_maintained
                {
                    failure_reasons.push(
                        "N.Y. Real Prop. Law § 235-e — landlord must keep proof of CASH rent receipts for 3 YEARS".to_string(),
                    );
                }
                (obligation_triggered, timing, content, retention)
            }
            Jurisdiction::California => {
                let obligation_triggered = input.tenant_requested_receipt;
                let timing = !obligation_triggered
                    || (input.receipt_provided && input.receipt_at_time_of_payment);
                let content = !obligation_triggered
                    || !input.receipt_provided
                    || input.receipt_content_complete;

                if obligation_triggered && !input.receipt_provided {
                    failure_reasons.push(
                        "Cal. Civ. Code § 1499 — upon TENANT REQUEST, landlord MUST provide a SIGNED AND DATED receipt; applies to all payment methods".to_string(),
                    );
                }
                if obligation_triggered
                    && input.receipt_provided
                    && !input.receipt_at_time_of_payment
                {
                    failure_reasons.push(
                        "Cal. Civ. Code § 1499 — receipt must be provided AT TIME OF PAYMENT (not year-end / not in lump sum at tax time)".to_string(),
                    );
                }
                (obligation_triggered, timing, content, true)
            }
            Jurisdiction::Massachusetts => {
                let last_month = matches!(
                    input.payment_method,
                    PaymentMethod::LastMonthsRentAtCommencement
                );
                let obligation_triggered = last_month;
                let timing = !obligation_triggered || input.receipt_provided;
                let content = !obligation_triggered
                    || !input.receipt_provided
                    || input.receipt_content_complete;

                if obligation_triggered && !input.receipt_provided {
                    failure_reasons.push(
                        "Mass. G.L. c. 186 § 15B — when tenant provides LAST MONTH'S RENT at commencement of tenancy, landlord MUST give a SIGNED RECEIPT with amount paid + date + description + landlord's name + tenant's name + name of person receiving payment".to_string(),
                    );
                }
                (obligation_triggered, timing, content, true)
            }
            Jurisdiction::Washington => {
                let cash_payment = matches!(input.payment_method, PaymentMethod::Cash);
                let obligation_triggered = cash_payment || input.tenant_requested_receipt;
                let timing = !obligation_triggered || input.receipt_provided;
                let content = !obligation_triggered
                    || !input.receipt_provided
                    || input.receipt_content_complete;

                if cash_payment && !input.receipt_provided {
                    failure_reasons.push(
                        "Wash. Rev. Code § 59.18.063 — when rent paid in CASH, landlord MUST provide written receipt".to_string(),
                    );
                }
                if input.tenant_requested_receipt && !cash_payment && !input.receipt_provided {
                    failure_reasons.push(
                        "Wash. Rev. Code § 59.18.063 — non-cash payments: receipt required upon TENANT REQUEST".to_string(),
                    );
                }
                (obligation_triggered, timing, content, true)
            }
            Jurisdiction::Default => (false, true, true, true),
        };

    let receipt_compliant = !receipt_obligation_triggered
        || (timing_compliant && content_compliant && retention_compliant);

    let notes: Vec<String> = vec![
        "N.Y. Real Prop. Law § 235-e — landlord must provide written receipt when rent paid by CASH OR MONEY ORDER OR CASHIER'S CHECK OR ANY FORM OTHER THAN tenant's personal check; in-person cash/money order = IMMEDIATE receipt; non-in-person = within 15 DAYS".to_string(),
        "N.Y. Real Prop. Law § 235-e content requirements: (1) PAYMENT DATE; (2) AMOUNT; (3) PERIOD for which rent was paid; (4) APARTMENT NUMBER; (5) SIGNATURE of person receiving payment; (6) TITLE of person receiving payment".to_string(),
        "N.Y. Real Prop. Law § 235-e PERSONAL CHECK — tenant may REQUEST in writing a rent receipt; after first request, landlord must provide receipt EVERY MONTH thereafter".to_string(),
        "N.Y. Real Prop. Law § 235-e RECORD RETENTION — landlord must keep proof of CASH rent receipts for 3 YEARS".to_string(),
        "Cal. Civ. Code § 1499 — upon TENANT REQUEST, landlord MUST provide SIGNED AND DATED receipt at TIME OF PAYMENT (not year-end / not in lump sum at tax time); applies to ALL payment methods (cash, check, money order, electronic)".to_string(),
        "Mass. G.L. c. 186 § 15B — LIMITED MANDATE: LAST MONTH'S RENT at commencement of tenancy requires signed receipt with amount + date + description + landlord's name + tenant's name + recipient's name; REGULAR MONTHLY RENT does NOT require receipt (even cash)".to_string(),
        "Wash. Rev. Code § 59.18.063 — CASH payments MANDATORY receipt; NON-CASH payments on tenant request".to_string(),
        "Default — no statewide mandate; common-law payment-of-rent dispute defense; tenant bears burden of proving payment (unless receipt requested and refused); some local ordinances (Chicago RLTO § 5-12-080(g), San Francisco) impose receipt requirements absent at state level".to_string(),
        "Trader-landlord critical because (1) cash rent receipts are mandatory in many states regardless of tenant request; (2) receipt-issuance failures create per-violation civil exposure plus evidentiary presumption against landlord in rent-payment disputes; (3) record retention obligations (3-year NY, similar elsewhere) extend long beyond tenancy termination; (4) modern payment methods (Zelle, Venmo, ACH) require careful receipt practices to satisfy state mandates".to_string(),
        "Cross-jurisdictional architecture: NY uses METHOD-TRIGGERED MANDATE + 3-YEAR RETENTION; California uses REQUEST-TRIGGERED MANDATE + TIME-OF-PAYMENT TIMING; Massachusetts uses LIMITED LAST-MONTH-ONLY MANDATE; Washington uses CASH-TRIGGERED + REQUEST-TRIGGERED MANDATE; Default uses common-law evidentiary defense".to_string(),
    ];

    TenantRentReceiptRequirementResult {
        jurisdiction: input.jurisdiction,
        receipt_obligation_triggered,
        receipt_compliant,
        timing_compliant,
        content_compliant,
        retention_compliant,
        failure_reasons,
        citation: "N.Y. Real Prop. Law § 235-e; Cal. Civ. Code § 1499; Mass. G.L. c. 186 § 15B; Wash. Rev. Code § 59.18.063; Chicago RLTO § 5-12-080(g)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_cash_compliant() -> TenantRentReceiptRequirementInput {
        TenantRentReceiptRequirementInput {
            jurisdiction: Jurisdiction::NewYork,
            payment_method: PaymentMethod::Cash,
            in_person_payment: true,
            tenant_requested_receipt: false,
            receipt_provided: true,
            days_since_payment: 0,
            receipt_content_complete: true,
            three_year_record_retention_maintained: true,
            receipt_at_time_of_payment: true,
        }
    }

    #[test]
    fn ny_cash_in_person_with_receipt_compliant() {
        let r = check(&ny_cash_compliant());
        assert!(r.receipt_obligation_triggered);
        assert!(r.receipt_compliant);
    }

    #[test]
    fn ny_cash_in_person_no_receipt_violation() {
        let mut i = ny_cash_compliant();
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 235-e") && f.contains("IMMEDIATE WRITTEN RECEIPT")));
    }

    #[test]
    fn ny_money_order_in_person_no_receipt_violation() {
        let mut i = ny_cash_compliant();
        i.payment_method = PaymentMethod::MoneyOrder;
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_compliant);
    }

    #[test]
    fn ny_cashiers_check_no_receipt_violation() {
        let mut i = ny_cash_compliant();
        i.payment_method = PaymentMethod::CashiersCheck;
        i.in_person_payment = false;
        i.receipt_provided = false;
        i.days_since_payment = 20;
        let r = check(&i);
        assert!(!r.timing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 235-e") && f.contains("15 DAYS")));
    }

    #[test]
    fn ny_personal_check_no_request_no_obligation() {
        let mut i = ny_cash_compliant();
        i.payment_method = PaymentMethod::PersonalCheck;
        i.in_person_payment = true;
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_obligation_triggered);
    }

    #[test]
    fn ny_personal_check_with_request_obligation_triggered() {
        let mut i = ny_cash_compliant();
        i.payment_method = PaymentMethod::PersonalCheck;
        i.tenant_requested_receipt = true;
        let r = check(&i);
        assert!(r.receipt_obligation_triggered);
    }

    #[test]
    fn ny_15_day_boundary_non_in_person_compliant() {
        let mut i = ny_cash_compliant();
        i.payment_method = PaymentMethod::ElectronicAchOrZelle;
        i.in_person_payment = false;
        i.receipt_provided = false;
        i.days_since_payment = 15;
        let r = check(&i);
        assert!(r.timing_compliant);
    }

    #[test]
    fn ny_16_day_non_in_person_violation() {
        let mut i = ny_cash_compliant();
        i.payment_method = PaymentMethod::ElectronicAchOrZelle;
        i.in_person_payment = false;
        i.receipt_provided = false;
        i.days_since_payment = 16;
        let r = check(&i);
        assert!(!r.timing_compliant);
    }

    #[test]
    fn ny_missing_content_violation() {
        let mut i = ny_cash_compliant();
        i.receipt_content_complete = false;
        let r = check(&i);
        assert!(!r.content_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 235-e")
            && f.contains("PAYMENT DATE")
            && f.contains("APARTMENT NUMBER")
            && f.contains("TITLE")));
    }

    #[test]
    fn ny_no_3_year_retention_violation() {
        let mut i = ny_cash_compliant();
        i.three_year_record_retention_maintained = false;
        let r = check(&i);
        assert!(!r.retention_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 235-e") && f.contains("3 YEARS")));
    }

    #[test]
    fn ca_request_triggers_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.tenant_requested_receipt = true;
        let r = check(&i);
        assert!(r.receipt_obligation_triggered);
    }

    #[test]
    fn ca_no_request_no_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.tenant_requested_receipt = false;
        let r = check(&i);
        assert!(!r.receipt_obligation_triggered);
    }

    #[test]
    fn ca_request_no_receipt_violation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.tenant_requested_receipt = true;
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1499") && f.contains("SIGNED AND DATED")));
    }

    #[test]
    fn ca_year_end_lump_sum_violation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.tenant_requested_receipt = true;
        i.receipt_provided = true;
        i.receipt_at_time_of_payment = false;
        let r = check(&i);
        assert!(!r.timing_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1499")
            && f.contains("AT TIME OF PAYMENT")
            && f.contains("year-end")));
    }

    #[test]
    fn ma_last_month_rent_receipt_required() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.payment_method = PaymentMethod::LastMonthsRentAtCommencement;
        let r = check(&i);
        assert!(r.receipt_obligation_triggered);
    }

    #[test]
    fn ma_regular_cash_no_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.payment_method = PaymentMethod::Cash;
        let r = check(&i);
        assert!(!r.receipt_obligation_triggered);
    }

    #[test]
    fn ma_last_month_no_receipt_violation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.payment_method = PaymentMethod::LastMonthsRentAtCommencement;
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_compliant);
        assert!(
            r.failure_reasons
                .iter()
                .any(|f| f.contains("c. 186 § 15B")
                    && f.contains("LAST MONTH'S RENT at commencement"))
        );
    }

    #[test]
    fn wa_cash_mandatory_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Washington;
        i.payment_method = PaymentMethod::Cash;
        let r = check(&i);
        assert!(r.receipt_obligation_triggered);
    }

    #[test]
    fn wa_cash_no_receipt_violation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Washington;
        i.payment_method = PaymentMethod::Cash;
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 59.18.063") && f.contains("CASH")));
    }

    #[test]
    fn wa_non_cash_request_triggers_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Washington;
        i.payment_method = PaymentMethod::PersonalCheck;
        i.tenant_requested_receipt = true;
        let r = check(&i);
        assert!(r.receipt_obligation_triggered);
    }

    #[test]
    fn wa_non_cash_no_request_no_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Washington;
        i.payment_method = PaymentMethod::PersonalCheck;
        i.tenant_requested_receipt = false;
        let r = check(&i);
        assert!(!r.receipt_obligation_triggered);
    }

    #[test]
    fn default_jurisdiction_no_obligation() {
        let mut i = ny_cash_compliant();
        i.jurisdiction = Jurisdiction::Default;
        i.payment_method = PaymentMethod::Cash;
        i.receipt_provided = false;
        let r = check(&i);
        assert!(!r.receipt_obligation_triggered);
        assert!(r.receipt_compliant);
    }

    #[test]
    fn jurisdiction_truth_table_five_cells() {
        for jur in [
            Jurisdiction::NewYork,
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::Washington,
            Jurisdiction::Default,
        ] {
            let mut i = ny_cash_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn ny_uniquely_engages_3_year_retention_invariant() {
        let mut ny = ny_cash_compliant();
        ny.three_year_record_retention_maintained = false;
        let r_ny = check(&ny);
        assert!(!r_ny.retention_compliant);

        for jur in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::Washington,
            Jurisdiction::Default,
        ] {
            let mut i = ny_cash_compliant();
            i.jurisdiction = jur;
            i.three_year_record_retention_maintained = false;
            let r = check(&i);
            assert!(r.retention_compliant, "jur={:?}", jur);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ny_cash_compliant());
        assert!(r.citation.contains("N.Y. Real Prop. Law § 235-e"));
        assert!(r.citation.contains("Cal. Civ. Code § 1499"));
        assert!(r.citation.contains("Mass. G.L. c. 186 § 15B"));
        assert!(r.citation.contains("Wash. Rev. Code § 59.18.063"));
        assert!(r.citation.contains("Chicago RLTO § 5-12-080(g)"));
    }

    #[test]
    fn note_pins_ny_immediate_vs_15_day_split() {
        let r = check(&ny_cash_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 235-e")
            && n.contains("CASH OR MONEY ORDER OR CASHIER'S CHECK")
            && n.contains("IMMEDIATE")
            && n.contains("15 DAYS")));
    }

    #[test]
    fn note_pins_ny_six_content_elements() {
        let r = check(&ny_cash_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 235-e content requirements")
                && n.contains("PAYMENT DATE")
                && n.contains("AMOUNT")
                && n.contains("PERIOD")
                && n.contains("APARTMENT NUMBER")
                && n.contains("SIGNATURE")
                && n.contains("TITLE")));
    }

    #[test]
    fn note_pins_ny_personal_check_recurring() {
        let r = check(&ny_cash_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 235-e PERSONAL CHECK") && n.contains("EVERY MONTH thereafter")));
    }

    #[test]
    fn note_pins_ny_3_year_retention() {
        let r = check(&ny_cash_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 235-e RECORD RETENTION") && n.contains("3 YEARS")));
    }

    #[test]
    fn note_pins_ca_request_signed_dated_time_of_payment() {
        let r = check(&ny_cash_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1499")
            && n.contains("SIGNED AND DATED")
            && n.contains("TIME OF PAYMENT")
            && n.contains("ALL payment methods")));
    }

    #[test]
    fn note_pins_ma_limited_last_month_only() {
        let r = check(&ny_cash_compliant());
        assert!(r.notes.iter().any(|n| n.contains("c. 186 § 15B")
            && n.contains("LIMITED MANDATE")
            && n.contains("LAST MONTH'S RENT")
            && n.contains("REGULAR MONTHLY RENT does NOT require")));
    }

    #[test]
    fn note_pins_wa_cash_mandatory_request_non_cash() {
        let r = check(&ny_cash_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 59.18.063")
            && n.contains("CASH payments MANDATORY")
            && n.contains("NON-CASH payments on tenant request")));
    }

    #[test]
    fn note_pins_default_local_ordinances() {
        let r = check(&ny_cash_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("common-law")
            && n.contains("Chicago RLTO § 5-12-080(g)")));
    }

    #[test]
    fn note_pins_trader_landlord_modern_payment_methods() {
        let r = check(&ny_cash_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical")
                && n.contains("3-year NY")
                && n.contains("Zelle, Venmo, ACH")));
    }

    #[test]
    fn note_pins_cross_jurisdictional_architecture() {
        let r = check(&ny_cash_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cross-jurisdictional architecture")
                && n.contains("METHOD-TRIGGERED MANDATE")
                && n.contains("REQUEST-TRIGGERED MANDATE")
                && n.contains("LIMITED LAST-MONTH-ONLY MANDATE")
                && n.contains("CASH-TRIGGERED")));
    }

    #[test]
    fn multiple_ny_failures_stack() {
        let mut i = ny_cash_compliant();
        i.in_person_payment = false;
        i.days_since_payment = 30;
        i.receipt_provided = false;
        i.three_year_record_retention_maintained = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 2);
    }
}
