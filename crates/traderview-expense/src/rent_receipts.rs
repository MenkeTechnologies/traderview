//! State rent receipt requirement compliance table.
//!
//! When a tenant pays rent, several states statutorily require the
//! landlord to provide a written receipt — sometimes for every
//! payment, sometimes only for cash payments, sometimes only upon
//! tenant request. Failure to comply can preclude the landlord from
//! later denying receipt of the payment in an eviction action.
//!
//! Four regimes:
//!
//! - `MandatoryReceiptEveryPayment` — NY (RPL § 235-e of HSTPA of
//!   2019): landlord must provide written receipt for EVERY rent
//!   payment, regardless of method (cash, check, electronic, money
//!   order). Receipt must state the date, amount, period of payment,
//!   apartment number, and signature/title of the recipient.
//!
//! - `MandatoryReceiptCashPaymentsOnly` — CA (Cal. Civ. Code § 1499:
//!   demand receipt available on request), MD (Real Prop. § 8-208
//!   record-keeping + receipt for cash), NJ (Truth-in-Renting Act
//!   cash receipt rules), IL (Chicago RLTO + IL Mobile Home Park
//!   Act cash receipt). Receipt required for cash payments; check /
//!   electronic / money order payments generate their own paper
//!   trails.
//!
//! - `ReceiptUponTenantRequest` — WA (RCW 59.18.063): landlord must
//!   provide written receipt for any payment ONLY when the tenant
//!   requests one. No automatic obligation.
//!
//! - `NoStateReceiptRequirement` — most other states. Includes MA
//!   for monthly RENT payments (MA's separate receipt rules apply
//!   only to security deposits and last month's rent under Mass.
//!   Gen. Laws ch. 186 § 15B, tracked elsewhere). Common-law and
//!   local ordinances may impose limited requirements.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptRegime {
    MandatoryReceiptEveryPayment,
    MandatoryReceiptCashPaymentsOnly,
    ReceiptUponTenantRequest,
    NoStateReceiptRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Check,
    Electronic,
    MoneyOrder,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: ReceiptRegime,
    pub citation: &'static str,
}

const fn rule(regime: ReceiptRegime, citation: &'static str) -> StateRule {
    StateRule { regime, citation }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use ReceiptRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            MandatoryReceiptEveryPayment,
            "N.Y. RPL § 235-e (HSTPA of 2019) — written receipt required for EVERY rent payment regardless of method; receipt must state date, amount, period, apartment, signature/title",
        ),
    );

    // Cash-only required.
    m.insert(
        "CA",
        rule(
            MandatoryReceiptCashPaymentsOnly,
            "Cal. Civ. Code § 1499 — landlord must furnish receipt for cash payment on tenant demand",
        ),
    );
    m.insert(
        "MD",
        rule(
            MandatoryReceiptCashPaymentsOnly,
            "Md. Real Prop. § 8-208 — landlord record-keeping requirement; receipt for cash payments required",
        ),
    );
    m.insert(
        "NJ",
        rule(
            MandatoryReceiptCashPaymentsOnly,
            "N.J. Truth-in-Renting Act — receipt required for cash payments with parties, dates, amounts, description, signature",
        ),
    );
    m.insert(
        "IL",
        rule(
            MandatoryReceiptCashPaymentsOnly,
            "Chicago RLTO + Ill. Mobile Home Park Act 765 ILCS 745/8 — receipt required for cash payments in covered jurisdictions",
        ),
    );

    // Upon tenant request.
    m.insert(
        "WA",
        rule(
            ReceiptUponTenantRequest,
            "Wash. RCW 59.18.063 — written receipt required upon tenant request for any payment",
        ),
    );

    // NoStateReceiptRequirement for all remaining states + DC. MA
    // intentionally falls here because its receipt requirements under
    // Mass. Gen. Laws ch. 186 § 15B(2)(c) apply ONLY to security
    // deposit and last month's rent (covered in other modules), not
    // monthly rent payments.
    let no_rule = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IN", "IA", "KS",
        "KY", "LA", "ME", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM", "NC", "ND",
        "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WV", "WI", "WY",
    ];
    for code in no_rule {
        let citation: &'static str = if code == "MA" {
            "No state mandate for monthly RENT receipts; Mass. Gen. Laws ch. 186 § 15B(2)(c) covers only deposit + last month rent receipts (tracked elsewhere)"
        } else {
            "No state-level rent receipt requirement; common-law and local ordinances may apply"
        };
        m.insert(code, rule(NoStateReceiptRequirement, citation));
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptInput {
    pub state_code: String,
    pub payment_method: PaymentMethod,
    pub tenant_requested_receipt: bool,
    pub landlord_provided_receipt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptResult {
    pub regime: ReceiptRegime,
    pub receipt_required: bool,
    pub landlord_compliant: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &ReceiptInput) -> ReceiptResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: ReceiptRegime::NoStateReceiptRequirement,
        citation: "Unknown state code; assuming no state-level receipt requirement",
    });

    let required = match rule.regime {
        ReceiptRegime::MandatoryReceiptEveryPayment => true,
        ReceiptRegime::MandatoryReceiptCashPaymentsOnly => {
            input.payment_method == PaymentMethod::Cash
        }
        ReceiptRegime::ReceiptUponTenantRequest => input.tenant_requested_receipt,
        ReceiptRegime::NoStateReceiptRequirement => false,
    };

    let compliant = !required || input.landlord_provided_receipt;

    let note = match (rule.regime, required, compliant) {
        (ReceiptRegime::MandatoryReceiptEveryPayment, _, true) =>
            "MandatoryReceiptEveryPayment (NY RPL § 235-e): receipt provided for the payment. Compliant.".to_string(),
        (ReceiptRegime::MandatoryReceiptEveryPayment, _, false) =>
            "MandatoryReceiptEveryPayment VIOLATION: NY RPL § 235-e requires written receipt for EVERY rent payment regardless of method; no receipt provided.".to_string(),
        (ReceiptRegime::MandatoryReceiptCashPaymentsOnly, true, true) => format!(
            "MandatoryReceiptCashPaymentsOnly: cash payment received with receipt. Compliant. ({:?} payment.)",
            input.payment_method,
        ),
        (ReceiptRegime::MandatoryReceiptCashPaymentsOnly, true, false) =>
            "MandatoryReceiptCashPaymentsOnly VIOLATION: cash payment required receipt; none provided. Landlord exposure for failure-of-receipt defense in eviction.".to_string(),
        (ReceiptRegime::MandatoryReceiptCashPaymentsOnly, false, _) => format!(
            "MandatoryReceiptCashPaymentsOnly: {:?} payment generates its own paper trail; no receipt mandate.",
            input.payment_method,
        ),
        (ReceiptRegime::ReceiptUponTenantRequest, true, true) =>
            "ReceiptUponTenantRequest: tenant requested receipt and landlord provided one. Compliant.".to_string(),
        (ReceiptRegime::ReceiptUponTenantRequest, true, false) =>
            "ReceiptUponTenantRequest VIOLATION: tenant requested receipt but landlord did not provide one.".to_string(),
        (ReceiptRegime::ReceiptUponTenantRequest, false, _) =>
            "ReceiptUponTenantRequest: tenant did not request receipt; no landlord duty triggered.".to_string(),
        (ReceiptRegime::NoStateReceiptRequirement, _, _) =>
            "NoStateReceiptRequirement: no state-level mandate; common-law and local ordinances may apply.".to_string(),
    };

    ReceiptResult {
        regime: rule.regime,
        receipt_required: required,
        landlord_compliant: compliant,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, method: PaymentMethod) -> ReceiptInput {
        ReceiptInput {
            state_code: state.to_string(),
            payment_method: method,
            tenant_requested_receipt: false,
            landlord_provided_receipt: false,
        }
    }

    // NY — every payment.

    #[test]
    fn ny_cash_payment_requires_receipt() {
        let r = check(&input("NY", PaymentMethod::Cash));
        assert_eq!(r.regime, ReceiptRegime::MandatoryReceiptEveryPayment);
        assert!(r.receipt_required);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ny_check_payment_still_requires_receipt() {
        let r = check(&input("NY", PaymentMethod::Check));
        assert!(r.receipt_required);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ny_electronic_payment_still_requires_receipt() {
        let r = check(&input("NY", PaymentMethod::Electronic));
        assert!(r.receipt_required);
    }

    #[test]
    fn ny_money_order_payment_still_requires_receipt() {
        let r = check(&input("NY", PaymentMethod::MoneyOrder));
        assert!(r.receipt_required);
    }

    #[test]
    fn ny_with_receipt_complies() {
        let mut i = input("NY", PaymentMethod::Check);
        i.landlord_provided_receipt = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // Cash-only states.

    #[test]
    fn ca_cash_payment_requires_receipt() {
        let r = check(&input("CA", PaymentMethod::Cash));
        assert_eq!(r.regime, ReceiptRegime::MandatoryReceiptCashPaymentsOnly);
        assert!(r.receipt_required);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_check_payment_no_receipt_required() {
        let r = check(&input("CA", PaymentMethod::Check));
        assert!(!r.receipt_required);
        assert!(r.landlord_compliant);
        assert!(r.note.contains("Check"));
        assert!(r.note.contains("paper trail"));
    }

    #[test]
    fn ca_electronic_payment_no_receipt_required() {
        let r = check(&input("CA", PaymentMethod::Electronic));
        assert!(!r.receipt_required);
    }

    #[test]
    fn ca_money_order_no_receipt_required() {
        let r = check(&input("CA", PaymentMethod::MoneyOrder));
        assert!(!r.receipt_required);
    }

    #[test]
    fn md_cash_payment_requires_receipt() {
        let r = check(&input("MD", PaymentMethod::Cash));
        assert!(r.receipt_required);
    }

    #[test]
    fn nj_cash_payment_requires_receipt() {
        let r = check(&input("NJ", PaymentMethod::Cash));
        assert!(r.receipt_required);
    }

    #[test]
    fn il_cash_payment_requires_receipt() {
        let r = check(&input("IL", PaymentMethod::Cash));
        assert!(r.receipt_required);
    }

    // WA — upon request.

    #[test]
    fn wa_no_request_no_receipt_required() {
        let r = check(&input("WA", PaymentMethod::Cash));
        assert_eq!(r.regime, ReceiptRegime::ReceiptUponTenantRequest);
        assert!(!r.receipt_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn wa_with_request_requires_receipt() {
        let mut i = input("WA", PaymentMethod::Cash);
        i.tenant_requested_receipt = true;
        let r = check(&i);
        assert!(r.receipt_required);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn wa_request_check_payment_also_requires_receipt() {
        // WA rule covers any payment method when requested.
        let mut i = input("WA", PaymentMethod::Check);
        i.tenant_requested_receipt = true;
        let r = check(&i);
        assert!(r.receipt_required);
    }

    // MA — explicitly NoStateReceiptRequirement for monthly rent.

    #[test]
    fn ma_monthly_rent_no_state_requirement_for_receipt() {
        // MA's receipt rules under Ch. 186 § 15B(2)(c) apply only to
        // security deposit and last month rent, not monthly rent.
        let r = check(&input("MA", PaymentMethod::Cash));
        assert_eq!(r.regime, ReceiptRegime::NoStateReceiptRequirement);
        assert!(!r.receipt_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ma_citation_explains_deposit_carveout() {
        let r = check(&input("MA", PaymentMethod::Cash));
        assert!(r.citation.contains("deposit"));
        assert!(r.citation.contains("ch. 186"));
    }

    // No-rule states.

    #[test]
    fn no_rule_states_compliant_by_default() {
        for st in &["TX", "FL", "OR", "OH", "GA", "VA"] {
            let r = check(&input(st, PaymentMethod::Cash));
            assert_eq!(r.regime, ReceiptRegime::NoStateReceiptRequirement, "{st}");
            assert!(!r.receipt_required, "{st}");
            assert!(r.landlord_compliant, "{st}");
        }
    }

    // Coverage / invariants.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(
            codes.len(),
            51,
            "expected 50 states + DC, got {}",
            codes.len()
        );
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn only_ny_uses_every_payment_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ReceiptRegime::MandatoryReceiptEveryPayment {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected NY only with MandatoryReceiptEveryPayment"
        );
    }

    #[test]
    fn cash_only_regime_states_4() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ReceiptRegime::MandatoryReceiptCashPaymentsOnly {
                count += 1;
            }
        }
        assert_eq!(
            count, 4,
            "expected CA + MD + NJ + IL only with cash-only regime"
        );
    }

    #[test]
    fn only_wa_uses_on_request_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ReceiptRegime::ReceiptUponTenantRequest {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected WA only with ReceiptUponTenantRequest");
    }

    #[test]
    fn unknown_state_falls_back_to_no_rule() {
        let r = check(&input("XX", PaymentMethod::Cash));
        assert_eq!(r.regime, ReceiptRegime::NoStateReceiptRequirement);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ny", PaymentMethod::Cash));
        assert!(r.receipt_required);
    }

    // Notes.

    #[test]
    fn ny_violation_note_mentions_rpl_235e() {
        let r = check(&input("NY", PaymentMethod::Cash));
        assert!(r.note.contains("RPL § 235-e"));
        assert!(r.note.contains("VIOLATION"));
    }

    #[test]
    fn ca_check_compliant_note_mentions_paper_trail() {
        let r = check(&input("CA", PaymentMethod::Check));
        assert!(r.note.contains("paper trail"));
    }

    #[test]
    fn ca_cash_violation_mentions_landlord_exposure() {
        let r = check(&input("CA", PaymentMethod::Cash));
        assert!(r.note.contains("Landlord exposure"));
    }
}
