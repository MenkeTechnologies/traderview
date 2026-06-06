//! IRC § 1031 like-kind exchanges of real property.
//!
//! Trader-landlord-critical nonrecognition provision for deferred
//! gain on real-property exchanges. Substantially modified by the
//! **Tax Cuts and Jobs Act of 2017 (TCJA, Pub. L. 115-97)** which
//! limited § 1031 to **REAL PROPERTY ONLY** — eliminating like-kind
//! exchange treatment for personal property (machinery, vehicles,
//! art, livestock, intangibles) for exchanges completed after
//! December 31, 2017.
//!
//! **§ 1031(a)(1) general nonrecognition rule**: no gain or loss
//! shall be recognized on the exchange of real property held for
//! productive use in a trade or business or for investment, if
//! such real property is exchanged solely for real property of
//! like kind which is to be held either for productive use in a
//! trade or business or for investment.
//!
//! **§ 1031(a)(2) post-TCJA exclusions**:
//!
//! - Stock in trade or other property held primarily for sale
//!   (inventory)
//! - Stocks, bonds, or notes
//! - Other securities or evidences of indebtedness or interest
//! - Interests in a partnership
//! - Certificates of trust or beneficial interests
//! - Choses in action
//! - Real property in a foreign country (post-1989)
//! - PERSONAL PROPERTY (added by TCJA 2017)
//!
//! **§ 1031(a)(3) deferred exchange identification rules**:
//!
//! - **45-day identification period**: replacement property must be
//!   identified in writing to qualified intermediary within 45 days
//!   of transfer of relinquished property.
//! - **180-day exchange period**: replacement property must be
//!   received within 180 days of transfer of relinquished property
//!   OR by due date (including extensions) of tax return for year
//!   of transfer, whichever is EARLIER.
//!
//! **Identification rules** (Treas. Reg. § 1.1031(k)-1(c)(4)):
//!
//! - **3-property rule**: identify up to **3 properties** of any
//!   value without regard to FMV.
//! - **200% rule**: identify any number of properties as long as
//!   aggregate FMV does not exceed **200%** of aggregate FMV of
//!   relinquished properties.
//! - **95% rule**: identify any number if 95% of identified
//!   properties (by FMV) are actually acquired.
//!
//! **§ 1031(b) gain recognition on boot**: gain is recognized to
//! the extent of "boot" received — money or non-like-kind property
//! (including debt relief).
//!
//! **§ 1031(c)**: loss is NEVER recognized in a like-kind exchange.
//!
//! **§ 1031(d) basis in replacement property**: basis of replacement
//! property = adjusted basis of relinquished property PLUS gain
//! recognized PLUS boot paid MINUS boot received.
//!
//! **Qualified Intermediary (QI)** requirement under Treas. Reg.
//! § 1.1031(k)-1(g)(4): QI cannot be the taxpayer, employee,
//! attorney, accountant, or close relative; must hold proceeds of
//! relinquished property sale during exchange period to prevent
//! "constructive receipt" by taxpayer.
//!
//! **T.D. 9935** (November 2020) — final regulations defining
//! "real property" post-TCJA: land + improvements to land + unsevered
//! natural products of land + water and air space superjacent to
//! land + state-law real property + inherently permanent structures.
//!
//! **Incidental personal property test** (Treas. Reg. § 1.1031(k)
//! -1(c)(5)): personal property incidental to real property
//! exchange not disqualifying if aggregate FMV ≤ **15%** of
//! replacement property aggregate FMV (still subject to gain
//! treatment).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const IDENTIFICATION_PERIOD_DAYS: u32 = 45;
#[allow(dead_code)]
pub const EXCHANGE_PERIOD_DAYS: u32 = 180;
#[allow(dead_code)]
pub const TCJA_2017_REAL_PROPERTY_ONLY_EFFECTIVE_YEAR: u32 = 2018;
#[allow(dead_code)]
pub const THREE_PROPERTY_RULE_MAX: u32 = 3;
#[allow(dead_code)]
pub const TWO_HUNDRED_PERCENT_RULE_PERCENT: u32 = 200;
#[allow(dead_code)]
pub const NINETY_FIVE_PERCENT_ACQUIRED_RULE_PERCENT: u32 = 95;
#[allow(dead_code)]
pub const INCIDENTAL_PERSONAL_PROPERTY_MAX_PERCENT: u32 = 15;
#[allow(dead_code)]
pub const RELATED_PARTY_HOLDING_PERIOD_YEARS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    RealPropertyDomesticBusinessOrInvestment,
    PersonalProperty,
    InventoryOrStockInTrade,
    StocksBondsNotes,
    PartnershipInterest,
    CertificateOfTrustOrChoseInAction,
    ForeignRealProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantSimultaneousLikeKindExchange,
    CompliantDeferredExchangeWith45And180DayCompliance,
    CompliantBootReceivedPartialGainRecognized,
    ViolationPersonalPropertyExcludedPostTCJA,
    ViolationStocksBondsNotesPartnershipInterestExcluded,
    ViolationInventoryStockInTradeExcluded,
    ViolationForeignRealPropertyExcluded,
    Violation45DayIdentificationDeadlineMissed,
    Violation180DayExchangePeriodMissed,
    ViolationQualifiedIntermediaryNotIndependent,
    ViolationIdentificationExceeds3PropertyAnd200PctRulesAndUnder95Pct,
    ViolationIncidentalPersonalPropertyExceeds15Pct,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub exchange_year: u32,
    pub property_type: PropertyType,
    pub held_for_business_or_investment: bool,
    pub days_to_identify_replacement: u32,
    pub days_to_complete_exchange: u32,
    pub qualified_intermediary_used_and_independent: bool,
    pub count_of_identified_properties: u32,
    pub aggregate_fmv_identified_vs_relinquished_percent: u32,
    pub percent_of_identified_properties_actually_acquired: u32,
    pub boot_received_cents: u64,
    pub realized_gain_cents: u64,
    pub incidental_personal_property_percent_of_replacement: u32,
    pub adjusted_basis_relinquished_property_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub gain_recognized_cents: u64,
    pub gain_deferred_cents: u64,
    pub basis_in_replacement_property_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section1031Input = Input;
pub type Section1031Output = Output;
pub type Section1031Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 1031(a)(1) (general nonrecognition rule for like-kind exchanges)".to_string(),
        "IRC § 1031(a)(2) (excluded property post-TCJA)".to_string(),
        "IRC § 1031(a)(3) (deferred exchange — 45/180 day rules)".to_string(),
        "IRC § 1031(b) (gain recognition on boot)".to_string(),
        "IRC § 1031(c) (loss never recognized)".to_string(),
        "IRC § 1031(d) (basis in replacement property)".to_string(),
        "IRC § 1031(f) (related party exchanges — 2-year holding)".to_string(),
        "Treas. Reg. § 1.1031(a)-1 (general rules)".to_string(),
        "Treas. Reg. § 1.1031(k)-1 (deferred exchanges)".to_string(),
        "Treas. Reg. § 1.1031(k)-1(c)(4) (3-property + 200% + 95% identification rules)"
            .to_string(),
        "Treas. Reg. § 1.1031(k)-1(c)(5) (incidental personal property 15% test)".to_string(),
        "Treas. Reg. § 1.1031(k)-1(g)(4) (qualified intermediary safe harbor)".to_string(),
        "TCJA 2017 (Pub. L. 115-97 § 13303 — real property only restriction)".to_string(),
        "T.D. 9935 (Nov 2020 — final regulations defining real property post-TCJA)".to_string(),
    ];

    if input.exchange_year >= TCJA_2017_REAL_PROPERTY_ONLY_EFFECTIVE_YEAR {
        match input.property_type {
            PropertyType::PersonalProperty => {
                notes.push("Personal property exchange post-TCJA 2017 — § 1031 no longer applies; full gain recognized.".to_string());
                return Output {
                    severity: Severity::ViolationPersonalPropertyExcludedPostTCJA,
                    compliant: false,
                    gain_recognized_cents: input.realized_gain_cents,
                    gain_deferred_cents: 0,
                    basis_in_replacement_property_cents: 0,
                    notes,
                    citations,
                };
            }
            PropertyType::StocksBondsNotes | PropertyType::PartnershipInterest => {
                notes.push("Stocks, bonds, notes, or partnership interest — excluded from § 1031 under § 1031(a)(2).".to_string());
                return Output {
                    severity: Severity::ViolationStocksBondsNotesPartnershipInterestExcluded,
                    compliant: false,
                    gain_recognized_cents: input.realized_gain_cents,
                    gain_deferred_cents: 0,
                    basis_in_replacement_property_cents: 0,
                    notes,
                    citations,
                };
            }
            PropertyType::InventoryOrStockInTrade => {
                notes.push(
                    "Inventory or stock in trade — excluded from § 1031 as primarily for sale."
                        .to_string(),
                );
                return Output {
                    severity: Severity::ViolationInventoryStockInTradeExcluded,
                    compliant: false,
                    gain_recognized_cents: input.realized_gain_cents,
                    gain_deferred_cents: 0,
                    basis_in_replacement_property_cents: 0,
                    notes,
                    citations,
                };
            }
            PropertyType::ForeignRealProperty => {
                notes.push(
                    "Foreign real property — excluded from § 1031 since 1989 amendment."
                        .to_string(),
                );
                return Output {
                    severity: Severity::ViolationForeignRealPropertyExcluded,
                    compliant: false,
                    gain_recognized_cents: input.realized_gain_cents,
                    gain_deferred_cents: 0,
                    basis_in_replacement_property_cents: 0,
                    notes,
                    citations,
                };
            }
            PropertyType::CertificateOfTrustOrChoseInAction => {
                notes.push(
                    "Certificate of trust or chose in action — excluded under § 1031(a)(2)."
                        .to_string(),
                );
                return Output {
                    severity: Severity::ViolationStocksBondsNotesPartnershipInterestExcluded,
                    compliant: false,
                    gain_recognized_cents: input.realized_gain_cents,
                    gain_deferred_cents: 0,
                    basis_in_replacement_property_cents: 0,
                    notes,
                    citations,
                };
            }
            PropertyType::RealPropertyDomesticBusinessOrInvestment => {}
        }
    }

    if input.incidental_personal_property_percent_of_replacement
        > INCIDENTAL_PERSONAL_PROPERTY_MAX_PERCENT
    {
        notes.push(format!(
            "Incidental personal property {}% > {}% threshold under Treas. Reg. § 1.1031(k)-1(c)(5) — exceeds permitted incidental scope.",
            input.incidental_personal_property_percent_of_replacement,
            INCIDENTAL_PERSONAL_PROPERTY_MAX_PERCENT
        ));
        return Output {
            severity: Severity::ViolationIncidentalPersonalPropertyExceeds15Pct,
            compliant: false,
            gain_recognized_cents: input.realized_gain_cents,
            gain_deferred_cents: 0,
            basis_in_replacement_property_cents: 0,
            notes,
            citations,
        };
    }

    if !input.qualified_intermediary_used_and_independent {
        notes.push("Qualified Intermediary not used or not independent (Treas. Reg. § 1.1031(k)-1(g)(4)) — taxpayer has constructive receipt of proceeds; deferral disqualified.".to_string());
        return Output {
            severity: Severity::ViolationQualifiedIntermediaryNotIndependent,
            compliant: false,
            gain_recognized_cents: input.realized_gain_cents,
            gain_deferred_cents: 0,
            basis_in_replacement_property_cents: 0,
            notes,
            citations,
        };
    }

    if input.days_to_identify_replacement > IDENTIFICATION_PERIOD_DAYS {
        notes.push(format!(
            "Replacement property identification {} days > {}-day deadline — § 1031(a)(3) violation; deferral disqualified.",
            input.days_to_identify_replacement, IDENTIFICATION_PERIOD_DAYS
        ));
        return Output {
            severity: Severity::Violation45DayIdentificationDeadlineMissed,
            compliant: false,
            gain_recognized_cents: input.realized_gain_cents,
            gain_deferred_cents: 0,
            basis_in_replacement_property_cents: 0,
            notes,
            citations,
        };
    }

    if input.days_to_complete_exchange > EXCHANGE_PERIOD_DAYS {
        notes.push(format!(
            "Exchange completion {} days > {}-day deadline — § 1031(a)(3) violation; deferral disqualified.",
            input.days_to_complete_exchange, EXCHANGE_PERIOD_DAYS
        ));
        return Output {
            severity: Severity::Violation180DayExchangePeriodMissed,
            compliant: false,
            gain_recognized_cents: input.realized_gain_cents,
            gain_deferred_cents: 0,
            basis_in_replacement_property_cents: 0,
            notes,
            citations,
        };
    }

    let three_property_rule_satisfied =
        input.count_of_identified_properties <= THREE_PROPERTY_RULE_MAX;
    let two_hundred_pct_rule_satisfied =
        input.aggregate_fmv_identified_vs_relinquished_percent <= TWO_HUNDRED_PERCENT_RULE_PERCENT;
    let ninety_five_pct_rule_satisfied = input.percent_of_identified_properties_actually_acquired
        >= NINETY_FIVE_PERCENT_ACQUIRED_RULE_PERCENT;
    if !three_property_rule_satisfied
        && !two_hundred_pct_rule_satisfied
        && !ninety_five_pct_rule_satisfied
    {
        notes.push(format!(
            "Identification rules violated: {} properties (> {}) AND {}% aggregate FMV (> {}%) AND {}% acquired (< {}%) — Treas. Reg. § 1.1031(k)-1(c)(4) violation.",
            input.count_of_identified_properties,
            THREE_PROPERTY_RULE_MAX,
            input.aggregate_fmv_identified_vs_relinquished_percent,
            TWO_HUNDRED_PERCENT_RULE_PERCENT,
            input.percent_of_identified_properties_actually_acquired,
            NINETY_FIVE_PERCENT_ACQUIRED_RULE_PERCENT
        ));
        return Output {
            severity: Severity::ViolationIdentificationExceeds3PropertyAnd200PctRulesAndUnder95Pct,
            compliant: false,
            gain_recognized_cents: input.realized_gain_cents,
            gain_deferred_cents: 0,
            basis_in_replacement_property_cents: 0,
            notes,
            citations,
        };
    }

    let gain_recognized = input.boot_received_cents.min(input.realized_gain_cents);
    let gain_deferred = input.realized_gain_cents.saturating_sub(gain_recognized);
    let basis_in_replacement = input
        .adjusted_basis_relinquished_property_cents
        .saturating_add(gain_recognized)
        .saturating_sub(input.boot_received_cents);

    if gain_recognized > 0 {
        notes.push(format!(
            "Boot received ${} → gain recognized ${} (§ 1031(b)); ${} deferred; basis in replacement property ${}.",
            input.boot_received_cents / 100,
            gain_recognized / 100,
            gain_deferred / 100,
            basis_in_replacement / 100
        ));
        return Output {
            severity: Severity::CompliantBootReceivedPartialGainRecognized,
            compliant: true,
            gain_recognized_cents: gain_recognized,
            gain_deferred_cents: gain_deferred,
            basis_in_replacement_property_cents: basis_in_replacement,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "§ 1031(a) compliant deferred exchange: 45-day identification + 180-day exchange + QI used + no boot received; full gain ${} deferred; basis ${} carries to replacement property.",
        input.realized_gain_cents / 100,
        input.adjusted_basis_relinquished_property_cents / 100
    ));
    Output {
        severity: Severity::CompliantDeferredExchangeWith45And180DayCompliance,
        compliant: true,
        gain_recognized_cents: 0,
        gain_deferred_cents: input.realized_gain_cents,
        basis_in_replacement_property_cents: input.adjusted_basis_relinquished_property_cents,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant_deferred() -> Input {
        Input {
            exchange_year: 2026,
            property_type: PropertyType::RealPropertyDomesticBusinessOrInvestment,
            held_for_business_or_investment: true,
            days_to_identify_replacement: 30,
            days_to_complete_exchange: 150,
            qualified_intermediary_used_and_independent: true,
            count_of_identified_properties: 3,
            aggregate_fmv_identified_vs_relinquished_percent: 150,
            percent_of_identified_properties_actually_acquired: 100,
            boot_received_cents: 0,
            realized_gain_cents: 50_000_000,
            incidental_personal_property_percent_of_replacement: 5,
            adjusted_basis_relinquished_property_cents: 20_000_000,
        }
    }

    #[test]
    fn deferred_exchange_45_180_day_compliant() {
        let out = check(&base_compliant_deferred());
        assert_eq!(
            out.severity,
            Severity::CompliantDeferredExchangeWith45And180DayCompliance
        );
        assert_eq!(out.gain_deferred_cents, 50_000_000);
        assert_eq!(out.basis_in_replacement_property_cents, 20_000_000);
    }

    #[test]
    fn personal_property_post_tcja_excluded() {
        let mut i = base_compliant_deferred();
        i.property_type = PropertyType::PersonalProperty;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationPersonalPropertyExcludedPostTCJA
        );
        assert_eq!(out.gain_recognized_cents, 50_000_000);
    }

    #[test]
    fn stocks_bonds_notes_excluded() {
        let mut i = base_compliant_deferred();
        i.property_type = PropertyType::StocksBondsNotes;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationStocksBondsNotesPartnershipInterestExcluded
        );
    }

    #[test]
    fn partnership_interest_excluded() {
        let mut i = base_compliant_deferred();
        i.property_type = PropertyType::PartnershipInterest;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationStocksBondsNotesPartnershipInterestExcluded
        );
    }

    #[test]
    fn inventory_excluded() {
        let mut i = base_compliant_deferred();
        i.property_type = PropertyType::InventoryOrStockInTrade;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationInventoryStockInTradeExcluded
        );
    }

    #[test]
    fn foreign_real_property_excluded() {
        let mut i = base_compliant_deferred();
        i.property_type = PropertyType::ForeignRealProperty;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationForeignRealPropertyExcluded);
    }

    #[test]
    fn forty_six_day_identification_violation() {
        let mut i = base_compliant_deferred();
        i.days_to_identify_replacement = 46;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Violation45DayIdentificationDeadlineMissed
        );
    }

    #[test]
    fn identification_at_exactly_45_days_compliant() {
        let mut i = base_compliant_deferred();
        i.days_to_identify_replacement = 45;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn one_hundred_eighty_one_day_exchange_violation() {
        let mut i = base_compliant_deferred();
        i.days_to_complete_exchange = 181;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Violation180DayExchangePeriodMissed);
    }

    #[test]
    fn exchange_at_exactly_180_days_compliant() {
        let mut i = base_compliant_deferred();
        i.days_to_complete_exchange = 180;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn qualified_intermediary_not_independent_violation() {
        let mut i = base_compliant_deferred();
        i.qualified_intermediary_used_and_independent = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationQualifiedIntermediaryNotIndependent
        );
    }

    #[test]
    fn three_property_rule_within_3_compliant() {
        let mut i = base_compliant_deferred();
        i.count_of_identified_properties = 3;
        i.aggregate_fmv_identified_vs_relinquished_percent = 500;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn two_hundred_percent_rule_within_200_compliant() {
        let mut i = base_compliant_deferred();
        i.count_of_identified_properties = 10;
        i.aggregate_fmv_identified_vs_relinquished_percent = 200;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn ninety_five_percent_rule_acquired_compliant() {
        let mut i = base_compliant_deferred();
        i.count_of_identified_properties = 10;
        i.aggregate_fmv_identified_vs_relinquished_percent = 500;
        i.percent_of_identified_properties_actually_acquired = 95;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn all_three_identification_rules_fail_violation() {
        let mut i = base_compliant_deferred();
        i.count_of_identified_properties = 10;
        i.aggregate_fmv_identified_vs_relinquished_percent = 500;
        i.percent_of_identified_properties_actually_acquired = 50;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationIdentificationExceeds3PropertyAnd200PctRulesAndUnder95Pct
        );
    }

    #[test]
    fn incidental_personal_property_exceeds_15_percent_violation() {
        let mut i = base_compliant_deferred();
        i.incidental_personal_property_percent_of_replacement = 16;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationIncidentalPersonalPropertyExceeds15Pct
        );
    }

    #[test]
    fn incidental_at_exactly_15_percent_compliant() {
        let mut i = base_compliant_deferred();
        i.incidental_personal_property_percent_of_replacement = 15;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn boot_received_partial_gain_recognized() {
        let mut i = base_compliant_deferred();
        i.boot_received_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantBootReceivedPartialGainRecognized
        );
        assert_eq!(out.gain_recognized_cents, 5_000_000);
        assert_eq!(out.gain_deferred_cents, 45_000_000);
    }

    #[test]
    fn boot_capped_at_realized_gain() {
        let mut i = base_compliant_deferred();
        i.boot_received_cents = 100_000_000;
        let out = check(&i);
        assert_eq!(out.gain_recognized_cents, 50_000_000);
    }

    #[test]
    fn citations_pin_1031_subsections() {
        let out = check(&base_compliant_deferred());
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(a)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(a)(3)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1031(f)")));
    }

    #[test]
    fn citations_pin_treas_reg_and_td_9935() {
        let out = check(&base_compliant_deferred());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.1031(k)-1")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("§ 1.1031(k)-1(c)(4)")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("§ 1.1031(k)-1(g)(4)")));
        assert!(out.citations.iter().any(|c| c.contains("T.D. 9935")));
        assert!(out.citations.iter().any(|c| c.contains("TCJA 2017")));
    }

    #[test]
    fn constant_pin_45_day_identification() {
        assert_eq!(IDENTIFICATION_PERIOD_DAYS, 45);
    }

    #[test]
    fn constant_pin_180_day_exchange() {
        assert_eq!(EXCHANGE_PERIOD_DAYS, 180);
    }

    #[test]
    fn constant_pin_3_property_rule_max() {
        assert_eq!(THREE_PROPERTY_RULE_MAX, 3);
    }

    #[test]
    fn constant_pin_200_percent_rule() {
        assert_eq!(TWO_HUNDRED_PERCENT_RULE_PERCENT, 200);
    }

    #[test]
    fn constant_pin_95_percent_acquired_rule() {
        assert_eq!(NINETY_FIVE_PERCENT_ACQUIRED_RULE_PERCENT, 95);
    }

    #[test]
    fn constant_pin_15_percent_incidental() {
        assert_eq!(INCIDENTAL_PERSONAL_PROPERTY_MAX_PERCENT, 15);
    }
}
