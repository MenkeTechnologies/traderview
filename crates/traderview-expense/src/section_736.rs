//! IRC § 736 payments to a retiring partner or a deceased partner's
//! successor in interest.
//!
//! Bifurcates all liquidating payments to a retiring or deceased
//! partner between two distinct characterizations:
//!
//! **§ 736(a) payments**: payments NOT in exchange for partner's
//! interest in partnership property. Subdivides further:
//!
//! - **§ 736(a)(1)**: payment determined WITH REGARD TO partnership
//!   income → treated as DISTRIBUTIVE SHARE of partnership income.
//!   Ordinary character; reduces other partners' distributive shares.
//! - **§ 736(a)(2)**: payment determined WITHOUT REGARD TO
//!   partnership income → treated as § 707(c) GUARANTEED PAYMENT.
//!   Ordinary character; deductible by partnership.
//!
//! **§ 736(b) payments**: payments in exchange for partner's interest
//! in partnership PROPERTY. Capital character to recipient; treated
//! as § 731 / § 732 distribution; NOT deductible by partnership.
//!
//! **§ 736(b)(2) special rule for service partnerships with general
//! partner retiring/dying** (post-DRA 1993): unrealized receivables
//! under § 751(c) and goodwill (except as partnership agreement
//! provides for payment) are NOT treated as partnership property —
//! they fall back into § 736(a) ordinary-income treatment. Designed
//! to preserve the historical pre-1993 flexibility for service-firm
//! partner buyouts (law firms, accounting firms, medical practices).
//!
//! **DRA 1993 § 13262 amendment**: applicable to partners retiring
//! or dying on or after January 5, 1993. Before DRA 1993, the
//! § 736(b)(2) special rule applied to ALL partnerships. After DRA
//! 1993, the special rule applies ONLY to (1) service partnerships
//! (capital is not a material income-producing factor) where (2)
//! the retiring/deceased partner is a general partner.
//!
//! For capital-intensive partnerships (which is most trader
//! partnerships), the DRA 1993 amendment locks all payment-for-
//! unrealized-receivables and payment-for-goodwill into § 736(b)
//! capital-character treatment, not the § 736(a) ordinary-character
//! alternative.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const DRA_1993_EFFECTIVE_DATE_YEAR: u32 = 1993;
#[allow(dead_code)]
pub const DRA_1993_EFFECTIVE_DATE_MONTH: u32 = 1;
#[allow(dead_code)]
pub const DRA_1993_EFFECTIVE_DATE_DAY: u32 = 5;
#[allow(dead_code)]
pub const DRA_1993_AMENDMENT_SECTION: u32 = 13262;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    NotApplicable,
    RetiringPartner,
    DeceasedPartner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Character {
    NotApplicable,
    OrdinaryDistributiveShare736a1,
    OrdinaryGuaranteedPayment736a2,
    CapitalProperty736b,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section736aDistributiveShareDeterminedWithIncome,
    Section736aGuaranteedPaymentDeterminedWithoutIncome,
    Section736bCapitalPropertyExchange,
    Section736b2ServicePartnershipGeneralPartnerOrdinaryFallback,
    DraPre1993PartnershipPropertyRules,
    ViolationMischaracterizedAs736bCapitalShouldBeOrdinary,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub transaction_type: TransactionType,
    pub payment_amount_cents: u64,
    pub payment_determined_with_regard_to_partnership_income: bool,
    pub payment_for_interest_in_partnership_property: bool,
    pub partnership_is_service_partnership_capital_not_material: bool,
    pub partner_is_general_partner: bool,
    pub payment_for_unrealized_receivables_751c: bool,
    pub payment_for_goodwill: bool,
    pub partnership_agreement_provides_for_goodwill_payment: bool,
    pub retirement_or_death_year: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub character: Character,
    pub partnership_deduction_allowed: bool,
    pub recipient_ordinary_income_cents: u64,
    pub recipient_capital_proceeds_cents: u64,
    pub subject_to_self_employment_tax: bool,
    pub dra_1993_special_rule_applies: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section736Input = Input;
pub type Section736Output = Output;
pub type Section736Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 736(a) (payments not in exchange for partnership property)".to_string(),
        "IRC § 736(a)(1) (distributive share when determined with regard to income)".to_string(),
        "IRC § 736(a)(2) (guaranteed payment under § 707(c))".to_string(),
        "IRC § 736(b) (payments in exchange for partnership property — capital character)".to_string(),
        "IRC § 736(b)(2) (service-partnership general-partner unrealized-receivables/goodwill special rule)".to_string(),
        "IRC § 736(b)(3) (service partnership definition — capital not material income-producing factor)".to_string(),
        "IRC § 751(c) (unrealized receivables definition cross-reference)".to_string(),
        "IRC § 707(c) (guaranteed payment cross-reference)".to_string(),
        "IRC § 731 (partnership distribution rules — apply to § 736(b))".to_string(),
        "IRC § 732 (distributee basis in distributed property — apply to § 736(b))".to_string(),
        "DRA 1993 § 13262 (Pub. L. 103-66, eff. partners retiring/dying on or after Jan. 5, 1993)".to_string(),
        "Treas. Reg. § 1.736-1".to_string(),
    ];

    if matches!(input.transaction_type, TransactionType::NotApplicable) {
        notes.push("No partner retirement or death payment recorded.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            character: Character::NotApplicable,
            partnership_deduction_allowed: false,
            recipient_ordinary_income_cents: 0,
            recipient_capital_proceeds_cents: 0,
            subject_to_self_employment_tax: false,
            dra_1993_special_rule_applies: false,
            notes,
            citations,
        };
    }

    if input.retirement_or_death_year < DRA_1993_EFFECTIVE_DATE_YEAR {
        notes.push(format!(
            "Retirement/death year {} pre-dates DRA 1993 (eff. Jan. 5, {}); pre-amendment § 736(b)(2) flexibility applies to all partnerships.",
            input.retirement_or_death_year, DRA_1993_EFFECTIVE_DATE_YEAR
        ));
        return Output {
            severity: Severity::DraPre1993PartnershipPropertyRules,
            character: Character::CapitalProperty736b,
            partnership_deduction_allowed: false,
            recipient_ordinary_income_cents: 0,
            recipient_capital_proceeds_cents: input.payment_amount_cents,
            subject_to_self_employment_tax: false,
            dra_1993_special_rule_applies: true,
            notes,
            citations,
        };
    }

    let service_partnership_special_rule_applies = input
        .partnership_is_service_partnership_capital_not_material
        && input.partner_is_general_partner;

    if input.payment_for_interest_in_partnership_property {
        let goodwill_excluded_from_property = input.payment_for_goodwill
            && !input.partnership_agreement_provides_for_goodwill_payment
            && service_partnership_special_rule_applies;
        let receivables_excluded_from_property = input.payment_for_unrealized_receivables_751c
            && service_partnership_special_rule_applies;

        if goodwill_excluded_from_property || receivables_excluded_from_property {
            notes.push(format!(
                "§ 736(b)(2) special rule applies: service partnership + general partner + payment for {} → recharacterized as § 736(a) ordinary income (not § 736(b) capital).",
                if goodwill_excluded_from_property && receivables_excluded_from_property {
                    "unrealized receivables and goodwill"
                } else if goodwill_excluded_from_property {
                    "goodwill (no partnership-agreement provision)"
                } else {
                    "unrealized receivables under § 751(c)"
                }
            ));
            return Output {
                severity: Severity::Section736b2ServicePartnershipGeneralPartnerOrdinaryFallback,
                character: Character::OrdinaryGuaranteedPayment736a2,
                partnership_deduction_allowed: true,
                recipient_ordinary_income_cents: input.payment_amount_cents,
                recipient_capital_proceeds_cents: 0,
                subject_to_self_employment_tax: input.partner_is_general_partner,
                dra_1993_special_rule_applies: true,
                notes,
                citations,
            };
        }

        notes.push(format!(
            "§ 736(b): payment in exchange for partnership property — ${} treated as § 731/§ 732 capital distribution; no partnership deduction.",
            input.payment_amount_cents / 100
        ));
        return Output {
            severity: Severity::Section736bCapitalPropertyExchange,
            character: Character::CapitalProperty736b,
            partnership_deduction_allowed: false,
            recipient_ordinary_income_cents: 0,
            recipient_capital_proceeds_cents: input.payment_amount_cents,
            subject_to_self_employment_tax: false,
            dra_1993_special_rule_applies: service_partnership_special_rule_applies,
            notes,
            citations,
        };
    }

    if input.payment_determined_with_regard_to_partnership_income {
        notes.push(format!(
            "§ 736(a)(1): payment ${} determined with regard to partnership income → distributive share to recipient (ordinary), reduces other partners' shares.",
            input.payment_amount_cents / 100
        ));
        return Output {
            severity: Severity::Section736aDistributiveShareDeterminedWithIncome,
            character: Character::OrdinaryDistributiveShare736a1,
            partnership_deduction_allowed: false,
            recipient_ordinary_income_cents: input.payment_amount_cents,
            recipient_capital_proceeds_cents: 0,
            subject_to_self_employment_tax: input.partner_is_general_partner,
            dra_1993_special_rule_applies: service_partnership_special_rule_applies,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "§ 736(a)(2): payment ${} determined without regard to partnership income → § 707(c) guaranteed payment (ordinary), § 162 partnership deduction.",
        input.payment_amount_cents / 100
    ));
    Output {
        severity: Severity::Section736aGuaranteedPaymentDeterminedWithoutIncome,
        character: Character::OrdinaryGuaranteedPayment736a2,
        partnership_deduction_allowed: true,
        recipient_ordinary_income_cents: input.payment_amount_cents,
        recipient_capital_proceeds_cents: 0,
        subject_to_self_employment_tax: input.partner_is_general_partner,
        dra_1993_special_rule_applies: service_partnership_special_rule_applies,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_capital_intensive_partnership_buyout() -> Input {
        Input {
            transaction_type: TransactionType::RetiringPartner,
            payment_amount_cents: 10_000_000,
            payment_determined_with_regard_to_partnership_income: false,
            payment_for_interest_in_partnership_property: true,
            partnership_is_service_partnership_capital_not_material: false,
            partner_is_general_partner: true,
            payment_for_unrealized_receivables_751c: false,
            payment_for_goodwill: false,
            partnership_agreement_provides_for_goodwill_payment: false,
            retirement_or_death_year: 2026,
        }
    }

    #[test]
    fn capital_intensive_partnership_property_payment_is_736b_capital() {
        let out = check(&base_capital_intensive_partnership_buyout());
        assert_eq!(out.severity, Severity::Section736bCapitalPropertyExchange);
        assert_eq!(out.character, Character::CapitalProperty736b);
        assert!(!out.partnership_deduction_allowed);
        assert_eq!(out.recipient_capital_proceeds_cents, 10_000_000);
    }

    #[test]
    fn payment_determined_with_income_no_property_is_736a1_distributive_share() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.payment_for_interest_in_partnership_property = false;
        i.payment_determined_with_regard_to_partnership_income = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section736aDistributiveShareDeterminedWithIncome
        );
        assert_eq!(out.character, Character::OrdinaryDistributiveShare736a1);
        assert!(!out.partnership_deduction_allowed);
        assert_eq!(out.recipient_ordinary_income_cents, 10_000_000);
    }

    #[test]
    fn payment_determined_without_income_no_property_is_736a2_guaranteed_payment() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.payment_for_interest_in_partnership_property = false;
        i.payment_determined_with_regard_to_partnership_income = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section736aGuaranteedPaymentDeterminedWithoutIncome
        );
        assert_eq!(out.character, Character::OrdinaryGuaranteedPayment736a2);
        assert!(out.partnership_deduction_allowed);
    }

    #[test]
    fn service_partnership_general_partner_goodwill_falls_back_to_736a() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.partnership_is_service_partnership_capital_not_material = true;
        i.partner_is_general_partner = true;
        i.payment_for_goodwill = true;
        i.partnership_agreement_provides_for_goodwill_payment = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section736b2ServicePartnershipGeneralPartnerOrdinaryFallback
        );
        assert_eq!(out.character, Character::OrdinaryGuaranteedPayment736a2);
        assert!(out.partnership_deduction_allowed);
        assert!(out.dra_1993_special_rule_applies);
    }

    #[test]
    fn service_partnership_general_partner_unrealized_receivables_falls_back_to_736a() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.partnership_is_service_partnership_capital_not_material = true;
        i.partner_is_general_partner = true;
        i.payment_for_unrealized_receivables_751c = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section736b2ServicePartnershipGeneralPartnerOrdinaryFallback
        );
        assert!(out.dra_1993_special_rule_applies);
    }

    #[test]
    fn service_partnership_goodwill_with_agreement_provision_stays_736b_capital() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.partnership_is_service_partnership_capital_not_material = true;
        i.partner_is_general_partner = true;
        i.payment_for_goodwill = true;
        i.partnership_agreement_provides_for_goodwill_payment = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section736bCapitalPropertyExchange);
        assert_eq!(out.character, Character::CapitalProperty736b);
    }

    #[test]
    fn service_partnership_limited_partner_no_special_rule_stays_736b() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.partnership_is_service_partnership_capital_not_material = true;
        i.partner_is_general_partner = false;
        i.payment_for_goodwill = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section736bCapitalPropertyExchange);
    }

    #[test]
    fn capital_intensive_general_partner_no_special_rule() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.partnership_is_service_partnership_capital_not_material = false;
        i.partner_is_general_partner = true;
        i.payment_for_goodwill = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section736bCapitalPropertyExchange);
        assert!(!out.dra_1993_special_rule_applies);
    }

    #[test]
    fn pre_dra_1993_year_1992_pre_amendment_rules_apply() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.retirement_or_death_year = 1992;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DraPre1993PartnershipPropertyRules);
        assert!(out.dra_1993_special_rule_applies);
    }

    #[test]
    fn dra_1993_year_1993_amendment_applies() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.retirement_or_death_year = 1993;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section736bCapitalPropertyExchange);
    }

    #[test]
    fn deceased_partner_treated_same_as_retiring_for_736() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.transaction_type = TransactionType::DeceasedPartner;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section736bCapitalPropertyExchange);
    }

    #[test]
    fn not_applicable_returns_default() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.transaction_type = TransactionType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
        assert_eq!(out.character, Character::NotApplicable);
    }

    #[test]
    fn general_partner_736a_payment_subject_to_se_tax() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.payment_for_interest_in_partnership_property = false;
        i.partner_is_general_partner = true;
        let out = check(&i);
        assert!(out.subject_to_self_employment_tax);
    }

    #[test]
    fn limited_partner_736a_payment_not_subject_to_se_tax() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.payment_for_interest_in_partnership_property = false;
        i.partner_is_general_partner = false;
        let out = check(&i);
        assert!(!out.subject_to_self_employment_tax);
    }

    #[test]
    fn citations_pin_736a_736b_736b2_736b3_subsections() {
        let out = check(&base_capital_intensive_partnership_buyout());
        assert!(out.citations.iter().any(|c| c.contains("§ 736(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 736(a)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 736(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 736(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 736(b)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 736(b)(3)")));
    }

    #[test]
    fn citations_pin_dra_1993_section_13262() {
        let out = check(&base_capital_intensive_partnership_buyout());
        assert!(out.citations.iter().any(|c| c.contains("DRA 1993")));
        assert!(out.citations.iter().any(|c| c.contains("§ 13262")));
        assert!(out.citations.iter().any(|c| c.contains("Pub. L. 103-66")));
    }

    #[test]
    fn citations_pin_751c_707c_731_732_cross_refs() {
        let out = check(&base_capital_intensive_partnership_buyout());
        assert!(out.citations.iter().any(|c| c.contains("§ 751(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 707(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 731")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732")));
    }

    #[test]
    fn constant_pin_dra_1993_effective_date() {
        assert_eq!(DRA_1993_EFFECTIVE_DATE_YEAR, 1993);
        assert_eq!(DRA_1993_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(DRA_1993_EFFECTIVE_DATE_DAY, 5);
    }

    #[test]
    fn constant_pin_dra_1993_amendment_section_13262() {
        assert_eq!(DRA_1993_AMENDMENT_SECTION, 13262);
    }

    #[test]
    fn very_large_payment_saturating_no_overflow() {
        let mut i = base_capital_intensive_partnership_buyout();
        i.payment_amount_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.recipient_capital_proceeds_cents, u64::MAX);
    }
}
