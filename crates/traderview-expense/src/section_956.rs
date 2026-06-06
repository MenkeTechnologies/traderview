//! IRC § 956 — Investment of Earnings in United States Property.
//!
//! § 956 is an anti-deferral rule that treats a CFC's investment in US
//! property as a constructive distribution (deemed dividend) to its US
//! shareholders. The rule prevents US shareholders from accessing CFC
//! earnings without actually receiving a taxable dividend distribution —
//! e.g., by having the CFC make a loan to the US shareholder, pledge stock
//! as collateral for the shareholder's debt, or buy US tangible property.
//!
//! § 956(a) general rule: a US shareholder of a CFC must include in gross
//! income its pro rata share of the average amount of US property held by
//! the CFC at the close of each quarter of the taxable year, but only to
//! the extent of CFC earnings and profits.
//!
//! § 956(c) "United States property" includes (1) tangible property located
//! in the US; (2) stock of a domestic corporation; (3) obligations of US
//! persons; (4) right to use US patents / copyrights / inventions /
//! trademarks acquired or developed by CFC for use in US. § 956(c)(2) seven
//! statutory exceptions: US bank deposits, export property, certain shipping
//! property, certain insurance reserves, certain securities, certain
//! aircraft and vessels used in international commerce, certain working
//! capital with related US persons.
//!
//! **§ 245A coordination rule** (Treas. Reg. § 1.956-1(a)(2)-(4), effective
//! for CFC tax years beginning on or after July 22, 2019): a CORPORATE US
//! shareholder's § 956 inclusion is REDUCED to the extent that a § 245A
//! 100% DRD would be allowed if an amount equal to the potential § 956
//! inclusion had been hypothetically distributed as a dividend. Hypothetical
//! distribution attributable first to § 959(c)(2) PTEP (previously taxed
//! E&P) then to § 959(c)(3) non-PTEP E&P. This effectively eliminates §
//! 956 inclusion for corporate US shareholders in most situations,
//! restoring symmetry with the TCJA participation exemption.
//!
//! Non-corporate US shareholders (individuals, RICs, REITs, S corps,
//! partnerships) DO NOT benefit from the § 956 / § 245A coordination rule
//! — § 956 inclusion continues to apply because § 245A is unavailable.
//! These shareholders may consider § 962 election to be taxed as a domestic
//! C corporation on the CFC inclusion, claim § 245A DRD on the hypothetical
//! distribution, and thereby eliminate the § 956 inclusion.
//!
//! Common operational scenarios: (1) CFC pledges its stock to secure US
//! parent's loan from third-party bank — pledge treated as § 956 investment
//! per Rev. Rul. 90-112; (2) CFC guarantees US parent's debt — triggering §
//! 956 per Notice 88-108; (3) CFC makes intercompany loan to US affiliate;
//! (4) CFC acquires real estate or tangible property in the US; (5) CFC
//! purchases stock in domestic corporation that is not parent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsShareholderType {
    /// Domestic C corporation — eligible for § 245A coordination rule
    /// reducing § 956 inclusion.
    DomesticCCorporation,
    /// Individual / partnership / S corp — § 956 inclusion applies in full
    /// absent § 962 election.
    IndividualOrPassThroughNoSection962,
    /// Individual / partnership / S corp electing § 962 corporate-rate
    /// taxation — benefits from § 245A coordination rule.
    IndividualOrPassThroughWithSection962Election,
    /// RIC — § 956 inclusion applies in full; § 245A unavailable.
    RegulatedInvestmentCompany,
    /// REIT — § 956 inclusion applies in full; § 245A unavailable.
    RealEstateInvestmentTrust,
    /// Not a US shareholder (less than 10% ownership).
    LessThan10PctNotUsShareholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsPropertyType {
    /// No US property held by CFC.
    NoUsProperty,
    /// Tangible property in US (real estate, equipment).
    TangibleUsProperty,
    /// Stock of domestic corporation (other than § 956(c)(2)(B) excluded).
    DomesticCorporationStock,
    /// Obligation of US person (loan, guarantee, pledge of CFC stock to
    /// secure US person's debt per Rev. Rul. 90-112).
    UsPersonObligationLoanOrGuarantee,
    /// Right to use US-source intangible.
    UsSourceIntangibleRight,
    /// § 956(c)(2) statutory exception applies (US bank deposit, export
    /// property, working capital, etc.).
    StatutoryExceptionApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotUsShareholderNoInclusion,
    NoUsPropertyNoInclusion,
    StatutoryExceptionExempt,
    CorporateUsShareholderSection245aFullReduction,
    CorporateUsShareholderSection245aPartialReduction,
    IndividualSection956FullInclusion,
    RicOrReitSection956FullInclusion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section956Input {
    pub us_shareholder_type: UsShareholderType,
    pub taxable_year: i32,
    /// Whether foreign corp is a CFC under § 957(a) (greater-than-50% US
    /// shareholder ownership).
    pub foreign_corp_is_cfc: bool,
    pub us_property_type: UsPropertyType,
    /// Average amount of US property at close of each quarter in cents.
    pub avg_quarterly_us_property_cents: u64,
    /// CFC E&P in cents (§ 956(a) cap).
    pub cfc_earnings_and_profits_cents: u64,
    /// Previously-taxed earnings and profits under § 959(c)(2) PTEP in cents
    /// (hypothetical distribution attributable first).
    pub section_959_c2_ptep_cents: u64,
    /// Non-PTEP E&P under § 959(c)(3) in cents.
    pub section_959_c3_non_ptep_cents: u64,
    /// Foreign-source portion of non-PTEP E&P eligible for § 245A DRD if
    /// hypothetically distributed (in cents).
    pub foreign_source_portion_of_non_ptep_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section956Result {
    pub severity: Severity,
    pub gross_section_956_inclusion_cents: u64,
    pub hypothetical_distribution_cents: u64,
    pub ptep_attributable_cents: u64,
    pub non_ptep_attributable_cents: u64,
    pub section_245a_drd_offset_cents: u64,
    pub net_section_956_inclusion_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SECTION_245A_COORDINATION_EFFECTIVE_DATE: &str = "2019-07-22";
pub const SECTION_956_INTRODUCED_YEAR: i32 = 1962;

pub fn check(input: &Section956Input) -> Section956Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.us_shareholder_type,
        UsShareholderType::LessThan10PctNotUsShareholder
    ) {
        notes.push(
            "Less-than-10% shareholder is NOT a US shareholder under § 951(b); no § 956 \
             inclusion applies. CFC investment in US property has no direct US-tax \
             consequence for portfolio shareholders."
                .to_string(),
        );
        return empty_result(
            Severity::NotUsShareholderNoInclusion,
            input,
            actions,
            notes,
            "26 U.S.C. § 951(b); § 956(a)",
        );
    }

    if !input.foreign_corp_is_cfc {
        notes.push(
            "Foreign corporation NOT a CFC under § 957(a); § 956 inclusion does not apply. \
             Subpart F + § 956 regimes inapplicable absent CFC status. Consider PFIC \
             analysis per [[section_1297]] if foreign corp meets income or asset PFIC tests."
                .to_string(),
        );
        return empty_result(
            Severity::NotApplicable,
            input,
            actions,
            notes,
            "26 U.S.C. § 957(a); coord. § 1297",
        );
    }

    if matches!(input.us_property_type, UsPropertyType::NoUsProperty) {
        notes.push(
            "CFC holds no US property at close of quarters; no § 956 inclusion. Verify \
             quarterly snapshot per § 956(a)(2) at close of each quarter — not annual \
             average."
                .to_string(),
        );
        return empty_result(
            Severity::NoUsPropertyNoInclusion,
            input,
            actions,
            notes,
            "26 U.S.C. § 956(a)(2)",
        );
    }

    if matches!(
        input.us_property_type,
        UsPropertyType::StatutoryExceptionApplies
    ) {
        notes.push(
            "§ 956(c)(2) statutory exception applies — categories include (A) US bank \
             deposits, (B) export property, (C) certain shipping property, (D) insurance \
             reserves, (E) certain securities, (F) aircraft and vessels in international \
             commerce, (G) related-US-person working capital. No § 956 inclusion despite \
             nominal US property."
                .to_string(),
        );
        return empty_result(
            Severity::StatutoryExceptionExempt,
            input,
            actions,
            notes,
            "26 U.S.C. § 956(c)(2)",
        );
    }

    let gross_inclusion = input
        .avg_quarterly_us_property_cents
        .min(input.cfc_earnings_and_profits_cents);

    let hypothetical_distribution = gross_inclusion;
    let ptep_attributable = hypothetical_distribution.min(input.section_959_c2_ptep_cents);
    let non_ptep_attributable = hypothetical_distribution.saturating_sub(ptep_attributable);
    let non_ptep_foreign_source =
        non_ptep_attributable.min(input.foreign_source_portion_of_non_ptep_cents);

    let is_corporate_us_shareholder = matches!(
        input.us_shareholder_type,
        UsShareholderType::DomesticCCorporation
            | UsShareholderType::IndividualOrPassThroughWithSection962Election
    );

    let section_245a_offset = if is_corporate_us_shareholder {
        ptep_attributable.saturating_add(non_ptep_foreign_source)
    } else {
        0
    };

    let net_inclusion = gross_inclusion.saturating_sub(section_245a_offset);

    let severity = match (input.us_shareholder_type, net_inclusion, gross_inclusion) {
        (UsShareholderType::DomesticCCorporation, 0, _) => {
            Severity::CorporateUsShareholderSection245aFullReduction
        }
        (UsShareholderType::DomesticCCorporation, _, _) => {
            Severity::CorporateUsShareholderSection245aPartialReduction
        }
        (UsShareholderType::IndividualOrPassThroughWithSection962Election, 0, _) => {
            Severity::CorporateUsShareholderSection245aFullReduction
        }
        (UsShareholderType::IndividualOrPassThroughWithSection962Election, _, _) => {
            Severity::CorporateUsShareholderSection245aPartialReduction
        }
        (UsShareholderType::IndividualOrPassThroughNoSection962, _, _) => {
            Severity::IndividualSection956FullInclusion
        }
        (UsShareholderType::RegulatedInvestmentCompany, _, _)
        | (UsShareholderType::RealEstateInvestmentTrust, _, _) => {
            Severity::RicOrReitSection956FullInclusion
        }
        _ => Severity::IndividualSection956FullInclusion,
    };

    if is_corporate_us_shareholder {
        actions.push(format!(
            "Corporate US shareholder § 956 inclusion of {} cents reduced by § 245A \
             coordination rule (Treas. Reg. § 1.956-1(a)(2)) hypothetical distribution \
             offset of {} cents ({} PTEP + {} foreign-source non-PTEP); net § 956 \
             inclusion = {} cents. Symmetry with actual distribution per Final Regs \
             effective for CFC tax years beginning on or after {}. File Form 5471 \
             Schedule I-1.",
            gross_inclusion,
            section_245a_offset,
            ptep_attributable,
            non_ptep_foreign_source,
            net_inclusion,
            SECTION_245A_COORDINATION_EFFECTIVE_DATE
        ));
    } else if matches!(
        input.us_shareholder_type,
        UsShareholderType::IndividualOrPassThroughNoSection962
    ) {
        actions.push(format!(
            "Individual / pass-through US shareholder WITHOUT § 962 election: full gross § \
             956 inclusion of {} cents taxable as ordinary income at individual marginal \
             rate (up to 37%). Consider § 962 election to be taxed at 21% corporate rate \
             AND claim § 245A coordination rule benefit reducing § 956 inclusion to net \
             {} cents (effective {} CFC tax year).",
            gross_inclusion, net_inclusion, SECTION_245A_COORDINATION_EFFECTIVE_DATE
        ));
    } else {
        actions.push(format!(
            "RIC or REIT US shareholder: full gross § 956 inclusion of {} cents applies; \
             § 245A unavailable for RIC / REIT per § 245A(a) eligibility limited to \
             domestic C corp. No coordination-rule offset. Symmetry mismatch with actual \
             dividends — note that an actual distribution to RIC/REIT would similarly NOT \
             qualify for § 245A.",
            gross_inclusion
        ));
    }

    notes.push(
        "Coordination with [[section_951a]] (GILTI / NCTI — § 956 inclusion is residual \
         after Subpart F + GILTI / NCTI absorb most CFC income), [[section_245a]] (§ 245A \
         DRD basis for the coordination-rule hypothetical distribution offset), [[section_\
         1297]] (PFIC mutually exclusive with CFC), [[section_959]] (PTEP ordering rules — \
         § 959(c)(2) PTEP > § 959(c)(3) non-PTEP), [[section_962]] (election for individual \
         shareholder to be taxed at corporate rate AND claim coordination rule), [[section_\
         59a]] (BEAT — separate regime). § 956 inclusion treated as PTEP after inclusion \
         under § 959(c)(2) — avoids double tax on subsequent actual distribution."
            .to_string(),
    );

    Section956Result {
        severity,
        gross_section_956_inclusion_cents: gross_inclusion,
        hypothetical_distribution_cents: hypothetical_distribution,
        ptep_attributable_cents: ptep_attributable,
        non_ptep_attributable_cents: non_ptep_attributable,
        section_245a_drd_offset_cents: section_245a_offset,
        net_section_956_inclusion_cents: net_inclusion,
        recommended_actions: actions,
        citation: "26 U.S.C. § 956(a)-(d); Treas. Reg. § 1.956-1; § 245A; § 951(b); § 957(a); § 959(c); § 962",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section956Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section956Result {
    notes.push(
        "Coordination with [[section_951a]] (GILTI / NCTI), [[section_245a]] (DRD basis), \
         [[section_1297]] (PFIC), [[section_959]] (PTEP ordering), [[section_962]] \
         (election), [[section_59a]] (BEAT)."
            .to_string(),
    );
    let _ = input;
    Section956Result {
        severity,
        gross_section_956_inclusion_cents: 0,
        hypothetical_distribution_cents: 0,
        ptep_attributable_cents: 0,
        non_ptep_attributable_cents: 0,
        section_245a_drd_offset_cents: 0,
        net_section_956_inclusion_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section956Input {
        Section956Input {
            us_shareholder_type: UsShareholderType::DomesticCCorporation,
            taxable_year: 2024,
            foreign_corp_is_cfc: true,
            us_property_type: UsPropertyType::UsPersonObligationLoanOrGuarantee,
            avg_quarterly_us_property_cents: 100_000_000_00,
            cfc_earnings_and_profits_cents: 200_000_000_00,
            section_959_c2_ptep_cents: 0,
            section_959_c3_non_ptep_cents: 200_000_000_00,
            foreign_source_portion_of_non_ptep_cents: 200_000_000_00,
        }
    }

    #[test]
    fn less_than_10_pct_not_us_shareholder() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::LessThan10PctNotUsShareholder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotUsShareholderNoInclusion));
        assert_eq!(r.net_section_956_inclusion_cents, 0);
    }

    #[test]
    fn foreign_corp_not_cfc_inapplicable() {
        let mut i = baseline();
        i.foreign_corp_is_cfc = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("section_1297")));
    }

    #[test]
    fn no_us_property_no_inclusion() {
        let mut i = baseline();
        i.us_property_type = UsPropertyType::NoUsProperty;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoUsPropertyNoInclusion));
        assert!(r.notes.iter().any(|n| n.contains("§ 956(a)(2)")));
    }

    #[test]
    fn statutory_exception_exempts_inclusion() {
        let mut i = baseline();
        i.us_property_type = UsPropertyType::StatutoryExceptionApplies;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::StatutoryExceptionExempt));
        assert!(r.notes.iter().any(|n| n.contains("§ 956(c)(2)")));
        assert!(r.notes.iter().any(|n| n.contains("export property")));
    }

    #[test]
    fn corporate_us_shareholder_245a_full_reduction() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CorporateUsShareholderSection245aFullReduction
        ));
        assert_eq!(r.net_section_956_inclusion_cents, 0);
        assert_eq!(r.section_245a_drd_offset_cents, 100_000_000_00);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Treas. Reg. § 1.956-1(a)(2)")));
    }

    #[test]
    fn corporate_us_shareholder_partial_reduction_when_non_foreign_source_portion() {
        let mut i = baseline();
        i.foreign_source_portion_of_non_ptep_cents = 30_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CorporateUsShareholderSection245aPartialReduction
        ));
        assert_eq!(r.section_245a_drd_offset_cents, 30_000_000_00);
        assert_eq!(r.net_section_956_inclusion_cents, 70_000_000_00);
    }

    #[test]
    fn ptep_attributable_first_per_treas_reg() {
        let mut i = baseline();
        i.section_959_c2_ptep_cents = 60_000_000_00;
        i.section_959_c3_non_ptep_cents = 140_000_000_00;
        i.foreign_source_portion_of_non_ptep_cents = 140_000_000_00;
        let r = check(&i);
        assert_eq!(r.ptep_attributable_cents, 60_000_000_00);
        assert_eq!(r.non_ptep_attributable_cents, 40_000_000_00);
    }

    #[test]
    fn individual_no_962_full_inclusion() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughNoSection962;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::IndividualSection956FullInclusion
        ));
        assert_eq!(r.section_245a_drd_offset_cents, 0);
        assert_eq!(
            r.net_section_956_inclusion_cents,
            r.gross_section_956_inclusion_cents
        );
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 962 election")));
    }

    #[test]
    fn individual_with_962_election_benefits_from_coordination() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughWithSection962Election;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CorporateUsShareholderSection245aFullReduction
        ));
        assert_eq!(r.net_section_956_inclusion_cents, 0);
    }

    #[test]
    fn ric_full_inclusion_no_coordination() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::RegulatedInvestmentCompany;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::RicOrReitSection956FullInclusion
        ));
        assert_eq!(r.section_245a_drd_offset_cents, 0);
    }

    #[test]
    fn reit_full_inclusion_no_coordination() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::RealEstateInvestmentTrust;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::RicOrReitSection956FullInclusion
        ));
        assert_eq!(r.section_245a_drd_offset_cents, 0);
    }

    #[test]
    fn inclusion_capped_at_ep() {
        let mut i = baseline();
        i.avg_quarterly_us_property_cents = 500_000_000_00;
        i.cfc_earnings_and_profits_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.gross_section_956_inclusion_cents, 100_000_000_00);
    }

    #[test]
    fn inclusion_zero_when_ep_zero() {
        let mut i = baseline();
        i.cfc_earnings_and_profits_cents = 0;
        let r = check(&i);
        assert_eq!(r.gross_section_956_inclusion_cents, 0);
    }

    #[test]
    fn pledge_of_cfc_stock_is_us_property() {
        let mut i = baseline();
        i.us_property_type = UsPropertyType::UsPersonObligationLoanOrGuarantee;
        let r = check(&i);
        assert!(r.gross_section_956_inclusion_cents > 0 || r.section_245a_drd_offset_cents > 0);
    }

    #[test]
    fn tangible_us_property_triggers_inclusion() {
        let mut i = baseline();
        i.us_property_type = UsPropertyType::TangibleUsProperty;
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughNoSection962;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::IndividualSection956FullInclusion
        ));
        assert!(r.gross_section_956_inclusion_cents > 0);
    }

    #[test]
    fn domestic_corporation_stock_triggers_inclusion() {
        let mut i = baseline();
        i.us_property_type = UsPropertyType::DomesticCorporationStock;
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughNoSection962;
        let r = check(&i);
        assert!(r.gross_section_956_inclusion_cents > 0);
    }

    #[test]
    fn us_source_intangible_triggers_inclusion() {
        let mut i = baseline();
        i.us_property_type = UsPropertyType::UsSourceIntangibleRight;
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughNoSection962;
        let r = check(&i);
        assert!(r.gross_section_956_inclusion_cents > 0);
    }

    #[test]
    fn section_245a_coordination_effective_date_pins_2019_07_22() {
        assert_eq!(SECTION_245A_COORDINATION_EFFECTIVE_DATE, "2019-07-22");
    }

    #[test]
    fn section_956_introduced_year_pins_1962() {
        assert_eq!(SECTION_956_INTRODUCED_YEAR, 1962);
    }

    #[test]
    fn action_references_form_5471_schedule_i_1() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 5471")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule I-1")));
    }

    #[test]
    fn coordination_note_references_951a_245a_959_962() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("section_962")));
    }

    #[test]
    fn citation_pins_treas_reg_956_1_and_245a_coordination() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("Treas. Reg. § 1.956-1"));
        assert!(r.citation.contains("§ 245A"));
        assert!(r.citation.contains("§ 951(b)"));
        assert!(r.citation.contains("§ 957(a)"));
        assert!(r.citation.contains("§ 959(c)"));
        assert!(r.citation.contains("§ 962"));
    }

    #[test]
    fn realistic_corp_with_full_foreign_source_no_net_inclusion() {
        let mut i = baseline();
        i.avg_quarterly_us_property_cents = 50_000_000_000_00;
        i.cfc_earnings_and_profits_cents = 100_000_000_000_00;
        i.foreign_source_portion_of_non_ptep_cents = 100_000_000_000_00;
        i.section_959_c3_non_ptep_cents = 100_000_000_000_00;
        let r = check(&i);
        assert_eq!(r.net_section_956_inclusion_cents, 0);
    }

    #[test]
    fn realistic_individual_us_property_full_inclusion() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughNoSection962;
        i.avg_quarterly_us_property_cents = 10_000_000_00;
        i.cfc_earnings_and_profits_cents = 50_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::IndividualSection956FullInclusion
        ));
        assert_eq!(r.gross_section_956_inclusion_cents, 10_000_000_00);
        assert_eq!(r.net_section_956_inclusion_cents, 10_000_000_00);
    }

    #[test]
    fn corp_with_ptep_only_full_offset() {
        let mut i = baseline();
        i.avg_quarterly_us_property_cents = 50_000_000_00;
        i.cfc_earnings_and_profits_cents = 100_000_000_00;
        i.section_959_c2_ptep_cents = 100_000_000_00;
        i.section_959_c3_non_ptep_cents = 0;
        i.foreign_source_portion_of_non_ptep_cents = 0;
        let r = check(&i);
        assert_eq!(r.ptep_attributable_cents, 50_000_000_00);
        assert_eq!(r.net_section_956_inclusion_cents, 0);
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.avg_quarterly_us_property_cents = u64::MAX / 10;
        i.cfc_earnings_and_profits_cents = u64::MAX / 10;
        let r = check(&i);
        let _ = r.net_section_956_inclusion_cents;
    }

    #[test]
    fn zero_us_property_zero_inclusion() {
        let mut i = baseline();
        i.avg_quarterly_us_property_cents = 0;
        let r = check(&i);
        assert_eq!(r.gross_section_956_inclusion_cents, 0);
        assert_eq!(r.net_section_956_inclusion_cents, 0);
    }

    #[test]
    fn note_pins_pledge_and_guarantee_revenue_rulings() {
        let i = baseline();
        let _r = check(&i);
        let module_doc = include_str!("section_956.rs");
        assert!(module_doc.contains("Rev. Rul. 90-112"));
        assert!(module_doc.contains("Notice 88-108"));
    }
}
