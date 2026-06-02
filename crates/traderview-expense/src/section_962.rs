//! IRC § 962 — Election by Individuals to Be Subject to Tax at Corporate
//! Rates.
//!
//! § 962 permits an individual US shareholder of a CFC to elect to be taxed
//! at the corporate rate under § 11 (21% flat) on (1) Subpart F inclusions
//! under § 951(a), (2) GILTI / NCTI inclusions under § 951A, and (3) § 956
//! investment-in-US-property inclusions — instead of the individual marginal
//! rate (up to 37%). The election unlocks the § 250 GILTI / NCTI deduction
//! (50% pre-2026 / 40% post-OBBBA) plus § 960 deemed-paid foreign tax credit
//! that would otherwise be available only to a domestic C corporation.
//!
//! § 962(a)(1): tax computed at corporate § 11 rate on § 951(a) and § 951A
//! amounts as if the individual were a domestic corporation.
//!
//! § 962(a)(2): § 960 FTC allowed on the same basis as for a domestic
//! corporation — pre-OBBBA at 80% (20% haircut), post-OBBBA at 90% (10%
//! haircut per Pub. L. 119-21 OBBBA effective for taxable years beginning
//! after December 31, 2025).
//!
//! § 962(b): tax under § 962(a) cannot exceed the additional tax that would
//! be imposed if the inclusion were not made (ceiling rule).
//!
//! § 962(d) ACTUAL DISTRIBUTION RULE — critical multi-year consideration:
//! when CFC later makes actual distribution of § 962 E&P to electing
//! individual, distribution is INCLUDIBLE in gross income to the extent it
//! exceeds the amount of US tax PAID under § 962 in the inclusion year.
//! This effectively imposes a SECOND layer of US tax on the portion of CFC
//! earnings beyond what was paid as § 962 corporate tax — distinguishing
//! § 962 from § 245A 100% DRD pathway which would eliminate the second
//! layer entirely.
//!
//! Treas. Reg. § 1.962-2(b)(1): election made annually by attaching a
//! statement to Form 1040 — election applies to ALL CFCs of the electing
//! shareholder for that year, cannot be made piecemeal.
//!
//! Treas. Reg. § 1.245A-5(b)(2): § 245A 100% DRD is NOT available to § 962
//! electors on actual distributions of § 962 E&P — § 962 election forecloses
//! the participation-exemption pathway.
//!
//! Rev. Rul. 2019-10: partner of partnership and shareholder of S corp may
//! make § 962 election at their individual level for the partner / shareholder
//! share of CFC inclusions flowed through from the partnership / S corp.
//!
//! Coordination matrix: § 962 election is most beneficial when (1) foreign
//! tax rate ≥ ~13% (pre-OBBBA) or ~14% (post-OBBBA) — FTC plus § 250
//! deduction substantially eliminate current-year US tax, (2) future
//! distribution is expected to qualify for qualified dividend rate or treaty
//! preferential rate, (3) actual distribution is delayed (NPV of deferred
//! second-layer tax exceeds current-year savings).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareholderType {
    /// Individual US shareholder eligible for § 962 election.
    IndividualUsShareholder,
    /// Estate of US individual eligible for § 962 election.
    EstateOfIndividual,
    /// Trust treated as US person eligible for § 962 election.
    UsTrust,
    /// Partnership or S corp itself — § 962 election made at partner /
    /// shareholder level per Rev. Rul. 2019-10, not at entity level.
    PartnershipOrSCorpEntityLevel,
    /// Domestic C corporation — § 962 inapplicable (already taxed at
    /// corporate rate).
    DomesticCCorporation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InclusionType {
    SubpartFInclusion,
    GiltiOrNctiInclusion,
    Section956Inclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotEligibleShareholderType,
    ElectionNotMadeIndividualRateApplies,
    ElectionMadeCurrentYearBenefit,
    ActualDistributionExceedsTaxPaidSecondLayerTriggered,
    ActualDistributionWithinTaxPaidNoSecondLayer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section962Input {
    pub shareholder_type: ShareholderType,
    pub inclusion_type: InclusionType,
    pub election_made_for_year: bool,
    pub taxable_year: i32,
    /// CFC inclusion amount in cents (§ 951(a) Subpart F or § 951A GILTI/NCTI
    /// or § 956).
    pub inclusion_amount_cents: u64,
    /// Foreign taxes paid by CFC allocable to inclusion in cents (basis for
    /// § 960 deemed-paid FTC).
    pub allocable_foreign_tax_cents: u64,
    /// Individual marginal rate that would apply without § 962 election in
    /// basis points (e.g., 3700 = 37%).
    pub individual_marginal_rate_bps: u32,
    /// Corporate § 11 rate in basis points (post-TCJA 21% = 2_100).
    pub corporate_rate_bps: u32,
    /// Section 250 GILTI / NCTI deduction percentage in basis points
    /// (pre-OBBBA 50% = 5_000, post-OBBBA 40% = 4_000).
    pub section_250_deduction_bps: u32,
    /// Actual distribution received in current year of § 962 E&P in cents
    /// (separate event from inclusion year).
    pub current_year_actual_distribution_of_962_ep_cents: u64,
    /// Cumulative US tax PAID under § 962 elections in prior years (cap on
    /// excluded portion of subsequent distribution per § 962(d)) in cents.
    pub cumulative_section_962_tax_paid_prior_years_cents: u64,
    /// Qualified dividend rate in basis points (applicable to second-layer
    /// distribution at long-term capital gain rates, 0/15/20%).
    pub qualified_dividend_rate_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section962Result {
    pub severity: Severity,
    pub election_made: bool,
    pub section_250_deduction_amount_cents: u64,
    pub taxable_inclusion_after_section_250_cents: u64,
    pub gross_us_tax_at_corporate_rate_cents: u64,
    pub section_960_ftc_cents: u64,
    pub net_us_tax_with_election_cents: u64,
    pub hypothetical_us_tax_without_election_cents: u64,
    pub current_year_election_benefit_cents: u64,
    pub second_layer_distribution_taxable_cents: u64,
    pub second_layer_tax_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SECTION_962_INTRODUCED_YEAR: i32 = 1962;
pub const PRE_OBBBA_FTC_HAIRCUT_BPS: u32 = 2_000;
pub const POST_OBBBA_FTC_HAIRCUT_BPS: u32 = 1_000;
pub const OBBBA_EFFECTIVE_YEAR: i32 = 2026;

pub fn check(input: &Section962Input) -> Section962Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.shareholder_type,
        ShareholderType::PartnershipOrSCorpEntityLevel | ShareholderType::DomesticCCorporation
    ) {
        notes.push(
            "Partnership or S corp does NOT make § 962 election at entity level — election \
             is made by individual partner or shareholder under Rev. Rul. 2019-10 for their \
             pro rata share of CFC inclusions flowed through from the entity. Domestic C \
             corp is already taxed at § 11 corporate rate; § 962 inapplicable."
                .to_string(),
        );
        return empty_result(
            Severity::NotEligibleShareholderType,
            input,
            actions,
            notes,
            "26 U.S.C. § 962(a); Rev. Rul. 2019-10",
        );
    }

    let is_post_obbba = input.taxable_year >= OBBBA_EFFECTIVE_YEAR;
    let ftc_rate_bps = if is_post_obbba {
        10_000 - POST_OBBBA_FTC_HAIRCUT_BPS
    } else {
        10_000 - PRE_OBBBA_FTC_HAIRCUT_BPS
    };

    let section_250_deduction: u64 = if matches!(input.inclusion_type, InclusionType::GiltiOrNctiInclusion) {
        (u128::from(input.inclusion_amount_cents)
            * u128::from(input.section_250_deduction_bps)
            / 10_000) as u64
    } else {
        0
    };
    let taxable_after_250 = input.inclusion_amount_cents.saturating_sub(section_250_deduction);
    let gross_us_tax_corporate: u64 = (u128::from(taxable_after_250)
        * u128::from(input.corporate_rate_bps)
        / 10_000) as u64;
    let raw_ftc: u64 = (u128::from(input.allocable_foreign_tax_cents)
        * u128::from(ftc_rate_bps)
        / 10_000) as u64;
    let actual_ftc = raw_ftc.min(gross_us_tax_corporate);
    let net_us_tax_with_election = gross_us_tax_corporate.saturating_sub(actual_ftc);

    let hypothetical_us_tax_without: u64 = (u128::from(input.inclusion_amount_cents)
        * u128::from(input.individual_marginal_rate_bps)
        / 10_000) as u64;

    let current_year_benefit = hypothetical_us_tax_without
        .saturating_sub(net_us_tax_with_election);

    let second_layer_taxable = input
        .current_year_actual_distribution_of_962_ep_cents
        .saturating_sub(input.cumulative_section_962_tax_paid_prior_years_cents);
    let second_layer_tax: u64 = (u128::from(second_layer_taxable)
        * u128::from(input.qualified_dividend_rate_bps)
        / 10_000) as u64;

    let severity = if !input.election_made_for_year && input.current_year_actual_distribution_of_962_ep_cents == 0 {
        Severity::ElectionNotMadeIndividualRateApplies
    } else if input.current_year_actual_distribution_of_962_ep_cents > 0 {
        if second_layer_taxable > 0 {
            Severity::ActualDistributionExceedsTaxPaidSecondLayerTriggered
        } else {
            Severity::ActualDistributionWithinTaxPaidNoSecondLayer
        }
    } else {
        Severity::ElectionMadeCurrentYearBenefit
    };

    if input.election_made_for_year {
        actions.push(format!(
            "Attach § 962 election statement to Form 1040 per Treas. Reg. § 1.962-2(b)(1); \
             election applies to ALL CFCs for tax year {} (cannot be made piecemeal). \
             Compute corporate-rate tax: inclusion of {} cents minus § 250 deduction of {} \
             cents = {} cents taxable; at {}% corporate rate = {} cents gross tax; § 960 \
             deemed-paid FTC at {}% of allocable foreign tax = {} cents (capped at gross \
             tax); net US tax = {} cents. Without election, individual rate of {}% on full \
             inclusion = {} cents. Current-year benefit = {} cents.",
            input.taxable_year,
            input.inclusion_amount_cents,
            section_250_deduction,
            taxable_after_250,
            input.corporate_rate_bps,
            gross_us_tax_corporate,
            ftc_rate_bps,
            actual_ftc,
            net_us_tax_with_election,
            input.individual_marginal_rate_bps,
            hypothetical_us_tax_without,
            current_year_benefit
        ));
    }

    if input.current_year_actual_distribution_of_962_ep_cents > 0 {
        actions.push(format!(
            "Actual distribution of § 962 E&P of {} cents received; under § 962(d), \
             distribution is INCLUDIBLE in gross income to extent it EXCEEDS cumulative US \
             tax paid under § 962 elections in prior years ({} cents). Includible amount = \
             {} cents; taxed at qualified-dividend / treaty preferential rate of {} bps = \
             {} cents. NOTE: § 245A 100% DRD is NOT available per Treas. Reg. § 1.245A-\
             5(b)(2) — § 962 election forecloses participation-exemption pathway.",
            input.current_year_actual_distribution_of_962_ep_cents,
            input.cumulative_section_962_tax_paid_prior_years_cents,
            second_layer_taxable,
            input.qualified_dividend_rate_bps,
            second_layer_tax
        ));
    }

    if matches!(input.inclusion_type, InclusionType::GiltiOrNctiInclusion) {
        notes.push(format!(
            "GILTI / NCTI inclusion with § 962 election: § 250 deduction of {} bps applies \
             ({}% pre-OBBBA / {}% post-OBBBA). Effective US rate after § 250 and § 960 FTC \
             targets {} bps; election most beneficial when foreign effective tax rate \
             approximates {} bps.",
            input.section_250_deduction_bps,
            if is_post_obbba { 40 } else { 50 },
            if is_post_obbba { 40 } else { 50 },
            if is_post_obbba { 1_260 } else { 1_050 },
            if is_post_obbba { 1_400 } else { 1_315 }
        ));
    }

    notes.push(
        "Coordination with [[section_951]] (Subpart F inclusion), [[section_951a]] (GILTI / \
         NCTI inclusion — iter 500), [[section_956]] (US property investment inclusion — \
         iter 504; § 245A coordination rule applies for corporate shareholder OR § 962 \
         elector), [[section_245a]] (foreign-source-portion 100% DRD — iter 502; NOT \
         available to § 962 electors on actual distributions per Treas. Reg. § 1.245A-\
         5(b)(2)), [[section_250]] (FDII + GILTI / NCTI deduction — § 962 elector qualifies \
         per Treas. Reg. § 1.962-2(b)), [[section_960]] (deemed-paid FTC — § 962 elector \
         claims as if domestic corp), [[section_59a]] (BEAT separate regime), [[section_911]] \
         (foreign earned income exclusion — distinct individual regime)."
            .to_string(),
    );

    Section962Result {
        severity,
        election_made: input.election_made_for_year,
        section_250_deduction_amount_cents: section_250_deduction,
        taxable_inclusion_after_section_250_cents: taxable_after_250,
        gross_us_tax_at_corporate_rate_cents: gross_us_tax_corporate,
        section_960_ftc_cents: actual_ftc,
        net_us_tax_with_election_cents: net_us_tax_with_election,
        hypothetical_us_tax_without_election_cents: hypothetical_us_tax_without,
        current_year_election_benefit_cents: current_year_benefit,
        second_layer_distribution_taxable_cents: second_layer_taxable,
        second_layer_tax_cents: second_layer_tax,
        recommended_actions: actions,
        citation: "26 U.S.C. § 962(a)-(d); § 951(a); § 951A; § 956; § 250; § 960; Treas. Reg. § 1.962-2(b); Treas. Reg. § 1.245A-5(b)(2); Rev. Rul. 2019-10",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section962Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section962Result {
    notes.push(
        "Coordination with [[section_951]] (Subpart F), [[section_951a]] (GILTI / NCTI), \
         [[section_956]] (US property investment), [[section_245a]] (DRD pathway alternative), \
         [[section_250]] (deduction), [[section_960]] (FTC), [[section_59a]] (BEAT), \
         [[section_911]] (FEIE)."
            .to_string(),
    );
    let _ = input;
    Section962Result {
        severity,
        election_made: false,
        section_250_deduction_amount_cents: 0,
        taxable_inclusion_after_section_250_cents: 0,
        gross_us_tax_at_corporate_rate_cents: 0,
        section_960_ftc_cents: 0,
        net_us_tax_with_election_cents: 0,
        hypothetical_us_tax_without_election_cents: 0,
        current_year_election_benefit_cents: 0,
        second_layer_distribution_taxable_cents: 0,
        second_layer_tax_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section962Input {
        Section962Input {
            shareholder_type: ShareholderType::IndividualUsShareholder,
            inclusion_type: InclusionType::GiltiOrNctiInclusion,
            election_made_for_year: true,
            taxable_year: 2024,
            inclusion_amount_cents: 100_000_000_00,
            allocable_foreign_tax_cents: 10_000_000_00,
            individual_marginal_rate_bps: 3_700,
            corporate_rate_bps: 2_100,
            section_250_deduction_bps: 5_000,
            current_year_actual_distribution_of_962_ep_cents: 0,
            cumulative_section_962_tax_paid_prior_years_cents: 0,
            qualified_dividend_rate_bps: 2_000,
        }
    }

    #[test]
    fn partnership_entity_level_not_eligible() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::PartnershipOrSCorpEntityLevel;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleShareholderType));
        assert!(r.notes.iter().any(|n| n.contains("Rev. Rul. 2019-10")));
    }

    #[test]
    fn domestic_c_corp_not_eligible() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::DomesticCCorporation;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleShareholderType));
    }

    #[test]
    fn individual_with_election_current_year_benefit() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ElectionMadeCurrentYearBenefit));
        assert!(r.election_made);
        assert!(r.current_year_election_benefit_cents > 0);
    }

    #[test]
    fn election_not_made_individual_rate_applies() {
        let mut i = baseline();
        i.election_made_for_year = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ElectionNotMadeIndividualRateApplies));
        assert!(!r.election_made);
    }

    #[test]
    fn gilti_with_section_250_50_pct_pre_obbba() {
        let mut i = baseline();
        i.taxable_year = 2024;
        i.inclusion_amount_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_250_deduction_amount_cents, 50_000_000_00);
        assert_eq!(r.taxable_inclusion_after_section_250_cents, 50_000_000_00);
    }

    #[test]
    fn subpart_f_no_section_250_deduction() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::SubpartFInclusion;
        let r = check(&i);
        assert_eq!(r.section_250_deduction_amount_cents, 0);
        assert_eq!(r.taxable_inclusion_after_section_250_cents, i.inclusion_amount_cents);
    }

    #[test]
    fn section_956_no_section_250_deduction() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::Section956Inclusion;
        let r = check(&i);
        assert_eq!(r.section_250_deduction_amount_cents, 0);
    }

    #[test]
    fn corporate_rate_21_pct_applied() {
        let i = baseline();
        let r = check(&i);
        let expected_gross_tax = r.taxable_inclusion_after_section_250_cents * 21 / 100;
        assert_eq!(r.gross_us_tax_at_corporate_rate_cents, expected_gross_tax);
    }

    #[test]
    fn pre_obbba_ftc_rate_80_pct() {
        let mut i = baseline();
        i.taxable_year = 2024;
        let r = check(&i);
        let expected_ftc = i.allocable_foreign_tax_cents * 8_000 / 10_000;
        assert_eq!(
            r.section_960_ftc_cents,
            expected_ftc.min(r.gross_us_tax_at_corporate_rate_cents)
        );
    }

    #[test]
    fn post_obbba_ftc_rate_90_pct() {
        let mut i = baseline();
        i.taxable_year = 2026;
        i.section_250_deduction_bps = 4_000;
        let r = check(&i);
        let expected_ftc = i.allocable_foreign_tax_cents * 9_000 / 10_000;
        assert_eq!(
            r.section_960_ftc_cents,
            expected_ftc.min(r.gross_us_tax_at_corporate_rate_cents)
        );
    }

    #[test]
    fn ftc_capped_at_gross_us_tax() {
        let mut i = baseline();
        i.allocable_foreign_tax_cents = u64::MAX / 100;
        let r = check(&i);
        assert!(r.section_960_ftc_cents <= r.gross_us_tax_at_corporate_rate_cents);
    }

    #[test]
    fn current_year_benefit_correctly_computed() {
        let i = baseline();
        let r = check(&i);
        let expected_benefit = r
            .hypothetical_us_tax_without_election_cents
            .saturating_sub(r.net_us_tax_with_election_cents);
        assert_eq!(r.current_year_election_benefit_cents, expected_benefit);
    }

    #[test]
    fn hypothetical_individual_rate_37_pct() {
        let i = baseline();
        let r = check(&i);
        let expected = i.inclusion_amount_cents * 37 / 100;
        assert_eq!(r.hypothetical_us_tax_without_election_cents, expected);
    }

    #[test]
    fn second_layer_tax_when_distribution_exceeds_prior_tax_paid() {
        let mut i = baseline();
        i.election_made_for_year = false;
        i.current_year_actual_distribution_of_962_ep_cents = 50_000_000_00;
        i.cumulative_section_962_tax_paid_prior_years_cents = 10_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ActualDistributionExceedsTaxPaidSecondLayerTriggered
        ));
        assert_eq!(r.second_layer_distribution_taxable_cents, 40_000_000_00);
        assert_eq!(r.second_layer_tax_cents, 40_000_000_00 * 2_000 / 10_000);
    }

    #[test]
    fn second_layer_zero_when_distribution_within_tax_paid() {
        let mut i = baseline();
        i.election_made_for_year = false;
        i.current_year_actual_distribution_of_962_ep_cents = 10_000_000_00;
        i.cumulative_section_962_tax_paid_prior_years_cents = 50_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ActualDistributionWithinTaxPaidNoSecondLayer
        ));
        assert_eq!(r.second_layer_distribution_taxable_cents, 0);
        assert_eq!(r.second_layer_tax_cents, 0);
    }

    #[test]
    fn second_layer_action_pins_treas_reg_245a_5_b_2() {
        let mut i = baseline();
        i.election_made_for_year = false;
        i.current_year_actual_distribution_of_962_ep_cents = 50_000_000_00;
        i.cumulative_section_962_tax_paid_prior_years_cents = 10_000_000_00;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Treas. Reg. § 1.245A-5(b)(2)")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 962(d)")));
    }

    #[test]
    fn action_references_treas_reg_962_2_and_form_1040() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Treas. Reg. § 1.962-2(b)(1)")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1040")));
    }

    #[test]
    fn note_pins_obbba_post_2026_effective_rate() {
        let mut i = baseline();
        i.taxable_year = 2026;
        i.section_250_deduction_bps = 4_000;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("post-OBBBA")));
        assert!(r.notes.iter().any(|n| n.contains("1260")));
    }

    #[test]
    fn coordination_note_references_951_951a_956_245a_250_960() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_951")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_250")));
        assert!(r.notes.iter().any(|n| n.contains("section_960")));
    }

    #[test]
    fn citation_pins_962_treas_reg_245a_5_rev_rul_2019_10() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 962(a)-(d)"));
        assert!(r.citation.contains("Treas. Reg. § 1.962-2(b)"));
        assert!(r.citation.contains("Treas. Reg. § 1.245A-5(b)(2)"));
        assert!(r.citation.contains("Rev. Rul. 2019-10"));
    }

    #[test]
    fn section_962_introduced_year_pins_1962() {
        assert_eq!(SECTION_962_INTRODUCED_YEAR, 1962);
    }

    #[test]
    fn obbba_effective_year_pins_2026() {
        assert_eq!(OBBBA_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn pre_obbba_haircut_pins_20_pct() {
        assert_eq!(PRE_OBBBA_FTC_HAIRCUT_BPS, 2_000);
    }

    #[test]
    fn post_obbba_haircut_pins_10_pct() {
        assert_eq!(POST_OBBBA_FTC_HAIRCUT_BPS, 1_000);
    }

    #[test]
    fn estate_eligible_shareholder() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::EstateOfIndividual;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ElectionMadeCurrentYearBenefit));
    }

    #[test]
    fn us_trust_eligible_shareholder() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::UsTrust;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ElectionMadeCurrentYearBenefit));
    }

    #[test]
    fn extreme_inclusion_does_not_overflow() {
        let mut i = baseline();
        i.inclusion_amount_cents = u64::MAX / 100;
        i.allocable_foreign_tax_cents = u64::MAX / 1000;
        let r = check(&i);
        let _ = r.net_us_tax_with_election_cents;
    }

    #[test]
    fn zero_inclusion_zero_tax() {
        let mut i = baseline();
        i.inclusion_amount_cents = 0;
        i.allocable_foreign_tax_cents = 0;
        let r = check(&i);
        assert_eq!(r.net_us_tax_with_election_cents, 0);
        assert_eq!(r.current_year_election_benefit_cents, 0);
    }

    #[test]
    fn realistic_individual_25k_inclusion_savings_demonstrated() {
        let mut i = baseline();
        i.inclusion_amount_cents = 1_000_000_00;
        i.allocable_foreign_tax_cents = 100_000_00;
        let r = check(&i);
        assert!(r.current_year_election_benefit_cents > 0);
    }
}
