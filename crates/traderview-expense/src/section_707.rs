//! IRC § 707 transactions between a partner and the partnership.
//!
//! Four distinct operative paragraphs, all of which override Subchapter
//! K default rules in different ways:
//!
//! - **§ 707(a)** — partner-partnership transactions are treated as if
//!   between non-partners (capital-asset purchase, ordinary sale, etc.)
//!   when partner is NOT acting in capacity as a partner.
//!
//! - **§ 707(a)(2)(A)** — payments to a partner FOR SERVICES that are
//!   contingent on partnership income may be recharacterized as either
//!   a § 707(c) guaranteed payment or a distributive share depending on
//!   the facts and circumstances.
//!
//! - **§ 707(a)(2)(B)** — DISGUISED SALES: a contribution of property
//!   to the partnership followed by a related distribution is recast
//!   as a sale to the partnership. **Treas. Reg. § 1.707-3(c)(1)**
//!   creates a **two-year presumption**: distributions within 2 years
//!   of contribution are PRESUMED to be sales unless facts and
//!   circumstances clearly establish otherwise. **Treas. Reg.
//!   § 1.707-3(d)** creates the OPPOSITE presumption for transfers
//!   more than 2 years apart: presumed NOT sales unless facts clearly
//!   establish they are.
//!
//! - **§ 707(b)** — losses disallowed on sales between a partner who
//!   owns directly or indirectly more than 50% of the partnership
//!   capital or profits interest and the partnership, or between two
//!   partnerships in which the same person owns more than 50%.
//!
//! - **§ 707(c)** — GUARANTEED PAYMENTS: payments to a partner FOR
//!   SERVICES rendered or for the USE OF CAPITAL, determined WITHOUT
//!   REGARD TO partnership income. Treated as ordinary income to the
//!   recipient and as a § 162 ordinary deduction to the partnership.
//!   Under § 1402(a)(13), guaranteed payments to a limited partner
//!   for services rendered are NOT excluded from self-employment tax
//!   (the limited-partner SECA exclusion applies only to distributive
//!   share, not to GPs).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const DISGUISED_SALE_TWO_YEAR_PRESUMPTION_MONTHS: u32 = 24;
#[allow(dead_code)]
pub const RELATED_PARTY_OWNERSHIP_THRESHOLD_PERCENT: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    NotApplicable,
    GuaranteedPaymentForServicesOrCapital,
    DisguisedSaleAnalysis,
    RelatedPartyLossSale,
    ArmsLengthOther,
    PartnerServicesContingentOnIncome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Characterization {
    NotApplicable,
    OrdinaryIncomeGuaranteedPayment,
    DistributiveShareNotGuaranteedPayment,
    SaleProceedsRecastUnder707a2B,
    DisallowedLossUnder707b,
    NonPartnerCapitalTransactionUnder707a,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    GuaranteedPaymentSection707cOrdinaryWithSeTax,
    GuaranteedPaymentSection707cOrdinaryNoSeTaxLimitedPartnerCapital,
    DisguisedSaleWithin2YearsPresumedSale,
    DisguisedSaleMoreThan2YearsPresumedNotSale,
    LossDisallowedRelatedPartnersUnder707b,
    LossAllowedNonRelatedPartiesUnder707b,
    ArmsLengthTransactionUnder707aNoRecharacterization,
    ViolationGuaranteedPaymentMisreportedAsDistributiveShare,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub transaction_kind: TransactionKind,
    pub months_between_contribution_and_distribution: u32,
    pub contribution_property_value_cents: u64,
    pub distribution_amount_cents: u64,
    pub payment_amount_cents: u64,
    pub payment_determined_without_regard_to_partnership_income: bool,
    pub payment_for_services_not_capital: bool,
    pub entrepreneurial_risk_independent: bool,
    pub but_for_test_met: bool,
    pub related_party_ownership_percent: u32,
    pub loss_amount_cents: u64,
    pub reported_as_guaranteed_payment: bool,
    pub partner_is_limited_partner: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub characterization: Characterization,
    pub ordinary_income_to_partner_cents: u64,
    pub subject_to_self_employment_tax: bool,
    pub disguised_sale_triggered: bool,
    pub disallowed_loss_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section707Input = Input;
pub type Section707Output = Output;
pub type Section707Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 707(a) (partner-partnership transactions treated as between non-partners)"
            .to_string(),
        "IRC § 707(a)(2)(A) (payments to partner for services)".to_string(),
        "IRC § 707(a)(2)(B) (disguised sales of property to partnership)".to_string(),
        "IRC § 707(b) (losses disallowed between related partners)".to_string(),
        "IRC § 707(c) (guaranteed payments — ordinary income)".to_string(),
        "IRC § 1402(a)(13) (limited-partner SECA exclusion; GP for services NOT excluded)"
            .to_string(),
        "Treas. Reg. § 1.707-3 (disguised sales — general rules)".to_string(),
        "Treas. Reg. § 1.707-3(c)(1) (two-year presumption SALE)".to_string(),
        "Treas. Reg. § 1.707-3(d) (more-than-two-year presumption NOT sale)".to_string(),
        "Treas. Reg. § 1.707-4 (guaranteed payments + preferred returns + reimbursements)"
            .to_string(),
        "IRC § 162 (partnership ordinary deduction for guaranteed payment)".to_string(),
    ];

    match input.transaction_kind {
        TransactionKind::NotApplicable => {
            notes.push("No partner-partnership transaction under § 707 recorded.".to_string());
            Output {
                severity: Severity::NotApplicable,
                characterization: Characterization::NotApplicable,
                ordinary_income_to_partner_cents: 0,
                subject_to_self_employment_tax: false,
                disguised_sale_triggered: false,
                disallowed_loss_cents: 0,
                notes,
                citations,
            }
        }
        TransactionKind::GuaranteedPaymentForServicesOrCapital => {
            if !input.payment_determined_without_regard_to_partnership_income {
                notes.push("Payment determined with regard to partnership income — fails § 707(c) requirement; treated as distributive share, not guaranteed payment.".to_string());
                if input.reported_as_guaranteed_payment {
                    return Output {
                        severity:
                            Severity::ViolationGuaranteedPaymentMisreportedAsDistributiveShare,
                        characterization: Characterization::DistributiveShareNotGuaranteedPayment,
                        ordinary_income_to_partner_cents: 0,
                        subject_to_self_employment_tax: false,
                        disguised_sale_triggered: false,
                        disallowed_loss_cents: 0,
                        notes,
                        citations,
                    };
                }
                return Output {
                    severity: Severity::ArmsLengthTransactionUnder707aNoRecharacterization,
                    characterization: Characterization::DistributiveShareNotGuaranteedPayment,
                    ordinary_income_to_partner_cents: 0,
                    subject_to_self_employment_tax: false,
                    disguised_sale_triggered: false,
                    disallowed_loss_cents: 0,
                    notes,
                    citations,
                };
            }

            if input.partner_is_limited_partner && !input.payment_for_services_not_capital {
                notes.push(format!(
                    "Limited partner guaranteed payment for USE OF CAPITAL — § 1402(a)(13) excludes from SE tax; ordinary income ${}.",
                    input.payment_amount_cents / 100
                ));
                return Output {
                    severity:
                        Severity::GuaranteedPaymentSection707cOrdinaryNoSeTaxLimitedPartnerCapital,
                    characterization: Characterization::OrdinaryIncomeGuaranteedPayment,
                    ordinary_income_to_partner_cents: input.payment_amount_cents,
                    subject_to_self_employment_tax: false,
                    disguised_sale_triggered: false,
                    disallowed_loss_cents: 0,
                    notes,
                    citations,
                };
            }

            notes.push(format!(
                "§ 707(c) guaranteed payment ${} — ordinary income to partner + § 162 ordinary deduction to partnership; subject to SE tax under § 1402(a)(13) (GP for services NOT excluded).",
                input.payment_amount_cents / 100
            ));
            Output {
                severity: Severity::GuaranteedPaymentSection707cOrdinaryWithSeTax,
                characterization: Characterization::OrdinaryIncomeGuaranteedPayment,
                ordinary_income_to_partner_cents: input.payment_amount_cents,
                subject_to_self_employment_tax: true,
                disguised_sale_triggered: false,
                disallowed_loss_cents: 0,
                notes,
                citations,
            }
        }
        TransactionKind::DisguisedSaleAnalysis => {
            let within_two_year_window = input.months_between_contribution_and_distribution
                <= DISGUISED_SALE_TWO_YEAR_PRESUMPTION_MONTHS;
            let both_prongs_met = input.but_for_test_met && !input.entrepreneurial_risk_independent;

            if within_two_year_window && both_prongs_met {
                notes.push(format!(
                    "Distribution within {} months + both prongs met (but-for + no independent entrepreneurial risk) — Treas. Reg. § 1.707-3(c)(1) two-year presumption: PRESUMED sale.",
                    DISGUISED_SALE_TWO_YEAR_PRESUMPTION_MONTHS
                ));
                return Output {
                    severity: Severity::DisguisedSaleWithin2YearsPresumedSale,
                    characterization: Characterization::SaleProceedsRecastUnder707a2B,
                    ordinary_income_to_partner_cents: 0,
                    subject_to_self_employment_tax: false,
                    disguised_sale_triggered: true,
                    disallowed_loss_cents: 0,
                    notes,
                    citations,
                };
            }
            if within_two_year_window {
                notes.push(format!(
                    "Distribution within {} months but two-prong test not satisfied — two-year presumption rebutted; treated as separate contribution and distribution.",
                    DISGUISED_SALE_TWO_YEAR_PRESUMPTION_MONTHS
                ));
                return Output {
                    severity: Severity::DisguisedSaleMoreThan2YearsPresumedNotSale,
                    characterization: Characterization::NonPartnerCapitalTransactionUnder707a,
                    ordinary_income_to_partner_cents: 0,
                    subject_to_self_employment_tax: false,
                    disguised_sale_triggered: false,
                    disallowed_loss_cents: 0,
                    notes,
                    citations,
                };
            }
            notes.push(format!(
                "Distribution more than {} months after contribution — Treas. Reg. § 1.707-3(d) opposite presumption: PRESUMED NOT a sale.",
                DISGUISED_SALE_TWO_YEAR_PRESUMPTION_MONTHS
            ));
            Output {
                severity: Severity::DisguisedSaleMoreThan2YearsPresumedNotSale,
                characterization: Characterization::NonPartnerCapitalTransactionUnder707a,
                ordinary_income_to_partner_cents: 0,
                subject_to_self_employment_tax: false,
                disguised_sale_triggered: false,
                disallowed_loss_cents: 0,
                notes,
                citations,
            }
        }
        TransactionKind::RelatedPartyLossSale => {
            if input.related_party_ownership_percent > RELATED_PARTY_OWNERSHIP_THRESHOLD_PERCENT {
                notes.push(format!(
                    "Related-party ownership {}% > § 707(b) {}% threshold — loss ${} disallowed.",
                    input.related_party_ownership_percent,
                    RELATED_PARTY_OWNERSHIP_THRESHOLD_PERCENT,
                    input.loss_amount_cents / 100
                ));
                return Output {
                    severity: Severity::LossDisallowedRelatedPartnersUnder707b,
                    characterization: Characterization::DisallowedLossUnder707b,
                    ordinary_income_to_partner_cents: 0,
                    subject_to_self_employment_tax: false,
                    disguised_sale_triggered: false,
                    disallowed_loss_cents: input.loss_amount_cents,
                    notes,
                    citations,
                };
            }
            notes.push(format!(
                "Ownership {}% ≤ § 707(b) {}% threshold — loss allowed.",
                input.related_party_ownership_percent, RELATED_PARTY_OWNERSHIP_THRESHOLD_PERCENT
            ));
            Output {
                severity: Severity::LossAllowedNonRelatedPartiesUnder707b,
                characterization: Characterization::NonPartnerCapitalTransactionUnder707a,
                ordinary_income_to_partner_cents: 0,
                subject_to_self_employment_tax: false,
                disguised_sale_triggered: false,
                disallowed_loss_cents: 0,
                notes,
                citations,
            }
        }
        TransactionKind::PartnerServicesContingentOnIncome => {
            notes.push("Payment for services contingent on partnership income — § 707(a)(2)(A) recharacterization required: treated as distributive share rather than guaranteed payment.".to_string());
            Output {
                severity: Severity::ArmsLengthTransactionUnder707aNoRecharacterization,
                characterization: Characterization::DistributiveShareNotGuaranteedPayment,
                ordinary_income_to_partner_cents: 0,
                subject_to_self_employment_tax: false,
                disguised_sale_triggered: false,
                disallowed_loss_cents: 0,
                notes,
                citations,
            }
        }
        TransactionKind::ArmsLengthOther => {
            notes.push("Arm's-length partner-partnership transaction treated under § 707(a) as if between non-partners (capital-asset purchase/sale, lease, lending, etc.).".to_string());
            Output {
                severity: Severity::ArmsLengthTransactionUnder707aNoRecharacterization,
                characterization: Characterization::NonPartnerCapitalTransactionUnder707a,
                ordinary_income_to_partner_cents: 0,
                subject_to_self_employment_tax: false,
                disguised_sale_triggered: false,
                disallowed_loss_cents: 0,
                notes,
                citations,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_gp() -> Input {
        Input {
            transaction_kind: TransactionKind::GuaranteedPaymentForServicesOrCapital,
            months_between_contribution_and_distribution: 0,
            contribution_property_value_cents: 0,
            distribution_amount_cents: 0,
            payment_amount_cents: 10_000_000,
            payment_determined_without_regard_to_partnership_income: true,
            payment_for_services_not_capital: true,
            entrepreneurial_risk_independent: false,
            but_for_test_met: false,
            related_party_ownership_percent: 0,
            loss_amount_cents: 0,
            reported_as_guaranteed_payment: true,
            partner_is_limited_partner: false,
        }
    }

    #[test]
    fn guaranteed_payment_for_services_is_ordinary_with_se_tax() {
        let out = check(&base_gp());
        assert_eq!(
            out.severity,
            Severity::GuaranteedPaymentSection707cOrdinaryWithSeTax
        );
        assert_eq!(
            out.characterization,
            Characterization::OrdinaryIncomeGuaranteedPayment
        );
        assert_eq!(out.ordinary_income_to_partner_cents, 10_000_000);
        assert!(out.subject_to_self_employment_tax);
    }

    #[test]
    fn limited_partner_gp_for_capital_no_se_tax() {
        let mut i = base_gp();
        i.partner_is_limited_partner = true;
        i.payment_for_services_not_capital = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::GuaranteedPaymentSection707cOrdinaryNoSeTaxLimitedPartnerCapital
        );
        assert!(!out.subject_to_self_employment_tax);
    }

    #[test]
    fn limited_partner_gp_for_services_still_se_tax_per_1402a13() {
        let mut i = base_gp();
        i.partner_is_limited_partner = true;
        i.payment_for_services_not_capital = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::GuaranteedPaymentSection707cOrdinaryWithSeTax
        );
        assert!(out.subject_to_self_employment_tax);
    }

    #[test]
    fn payment_determined_with_regard_to_income_misreported_as_gp_is_violation() {
        let mut i = base_gp();
        i.payment_determined_without_regard_to_partnership_income = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationGuaranteedPaymentMisreportedAsDistributiveShare
        );
        assert_eq!(
            out.characterization,
            Characterization::DistributiveShareNotGuaranteedPayment
        );
    }

    #[test]
    fn disguised_sale_within_2_years_both_prongs_met_presumed_sale() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::DisguisedSaleAnalysis;
        i.months_between_contribution_and_distribution = 12;
        i.but_for_test_met = true;
        i.entrepreneurial_risk_independent = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisguisedSaleWithin2YearsPresumedSale
        );
        assert!(out.disguised_sale_triggered);
    }

    #[test]
    fn disguised_sale_within_2_years_prongs_not_met_presumption_rebutted() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::DisguisedSaleAnalysis;
        i.months_between_contribution_and_distribution = 12;
        i.but_for_test_met = false;
        i.entrepreneurial_risk_independent = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisguisedSaleMoreThan2YearsPresumedNotSale
        );
        assert!(!out.disguised_sale_triggered);
    }

    #[test]
    fn disguised_sale_more_than_2_years_opposite_presumption_not_sale() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::DisguisedSaleAnalysis;
        i.months_between_contribution_and_distribution = 30;
        i.but_for_test_met = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisguisedSaleMoreThan2YearsPresumedNotSale
        );
    }

    #[test]
    fn two_year_boundary_24_months_still_within_window() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::DisguisedSaleAnalysis;
        i.months_between_contribution_and_distribution = 24;
        i.but_for_test_met = true;
        i.entrepreneurial_risk_independent = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisguisedSaleWithin2YearsPresumedSale
        );
    }

    #[test]
    fn two_year_boundary_25_months_outside_window() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::DisguisedSaleAnalysis;
        i.months_between_contribution_and_distribution = 25;
        i.but_for_test_met = true;
        i.entrepreneurial_risk_independent = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisguisedSaleMoreThan2YearsPresumedNotSale
        );
    }

    #[test]
    fn related_party_loss_above_50_percent_disallowed_under_707b() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::RelatedPartyLossSale;
        i.related_party_ownership_percent = 60;
        i.loss_amount_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::LossDisallowedRelatedPartnersUnder707b
        );
        assert_eq!(out.disallowed_loss_cents, 5_000_000);
    }

    #[test]
    fn related_party_50_percent_boundary_loss_allowed() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::RelatedPartyLossSale;
        i.related_party_ownership_percent = 50;
        i.loss_amount_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::LossAllowedNonRelatedPartiesUnder707b
        );
        assert_eq!(out.disallowed_loss_cents, 0);
    }

    #[test]
    fn related_party_51_percent_boundary_loss_disallowed() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::RelatedPartyLossSale;
        i.related_party_ownership_percent = 51;
        i.loss_amount_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::LossDisallowedRelatedPartnersUnder707b
        );
    }

    #[test]
    fn arms_length_other_transaction_under_707a() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::ArmsLengthOther;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ArmsLengthTransactionUnder707aNoRecharacterization
        );
        assert_eq!(
            out.characterization,
            Characterization::NonPartnerCapitalTransactionUnder707a
        );
    }

    #[test]
    fn partner_services_contingent_on_income_recharacterized_as_distributive() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::PartnerServicesContingentOnIncome;
        let out = check(&i);
        assert_eq!(
            out.characterization,
            Characterization::DistributiveShareNotGuaranteedPayment
        );
    }

    #[test]
    fn not_applicable_transaction_returns_default() {
        let mut i = base_gp();
        i.transaction_kind = TransactionKind::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_all_707_subsections() {
        let out = check(&base_gp());
        assert!(out.citations.iter().any(|c| c.contains("§ 707(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 707(a)(2)(A)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 707(a)(2)(B)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 707(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 707(c)")));
    }

    #[test]
    fn citations_pin_treas_reg_1_707_3_and_4() {
        let out = check(&base_gp());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.707-3")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.707-3(c)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.707-3(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.707-4")));
    }

    #[test]
    fn citations_pin_1402a13_limited_partner_seca() {
        let out = check(&base_gp());
        assert!(out.citations.iter().any(|c| c.contains("§ 1402(a)(13)")));
    }

    #[test]
    fn constant_pin_24_month_disguised_sale_window() {
        assert_eq!(DISGUISED_SALE_TWO_YEAR_PRESUMPTION_MONTHS, 24);
    }

    #[test]
    fn constant_pin_50_pct_related_party_threshold() {
        assert_eq!(RELATED_PARTY_OWNERSHIP_THRESHOLD_PERCENT, 50);
    }

    #[test]
    fn very_large_gp_amount_saturating_no_overflow() {
        let mut i = base_gp();
        i.payment_amount_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.ordinary_income_to_partner_cents, u64::MAX);
    }
}
