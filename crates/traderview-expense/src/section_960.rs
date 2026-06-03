//! IRC § 960 — Deemed-Paid Credit for Subpart F Inclusions, GILTI / NCTI,
//! and PTEP Distributions.
//!
//! § 960 grants domestic corporate US shareholders a DEEMED-PAID foreign
//! tax credit for foreign taxes paid by a controlled foreign corporation
//! (CFC) attributable to amounts the US shareholder includes in income.
//! Treats the US shareholder as if it had paid the CFC's foreign tax
//! directly, enabling FTC against US tax on the inclusion.
//!
//! § 960(a) SUBPART F deemed-paid credit: domestic corp US shareholder
//! including § 951(a)(1) Subpart F amounts is deemed to have paid the CFC's
//! foreign income taxes properly attributable to that inclusion. Full
//! creditability (no haircut) for Subpart F basket.
//!
//! § 960(b) PREVIOUSLY TAXED E&P (PTEP) distribution credit: foreign taxes
//! paid in the year of an actual § 959 distribution of PTEP (typically
//! foreign withholding tax) are deemed paid by the receiving US shareholder.
//! Coordinates with `section_959` sixteen-basket PTEP framework.
//!
//! § 960(c) DOMESTIC CORPORATION INCLUSIONS RULE: deemed-paid mechanism
//! applies only to domestic C corporations; individuals + S corps +
//! partnerships ineligible UNLESS § 962 election in place (see
//! `section_962` iter 510).
//!
//! § 960(d) GILTI / NCTI DEEMED-PAID CREDIT: US shareholder including a
//! § 951A amount is deemed to have paid foreign income taxes equal to
//! (PRE-OBBBA 80% / POST-OBBBA 90%) × inclusion percentage × tested
//! foreign income taxes in tested-income group within each § 904
//! category. The reduction percentage is sometimes called the "GILTI
//! haircut."
//!
//! OBBBA § 960(d) RATE CHANGE: Pub. L. 119-21 (signed July 4, 2025)
//! increased § 960(d) deemed-paid rate from 80% to 90% effective for
//! taxable years of US shareholders beginning after December 31, 2025.
//!
//! § 960(d)(4) PTEP DISTRIBUTION HAIRCUT (OBBBA): disallows credit for 10%
//! of foreign income taxes paid or accrued (including § 960(b)(1) deemed-
//! paid) with respect to a § 959(a) PTEP distribution where the PTEP
//! results from a § 951A inclusion in a US shareholder taxable year ending
//! AFTER June 28, 2025. Notice 2025-77 (interim guidance pending proposed
//! regulations) implements; taxpayers may rely on Section 3 of Notice
//! 2025-77 for US shareholder taxable years beginning before proposed
//! regulations published.
//!
//! Treas. Reg. § 1.960-1 (basic framework + PTEP accounts), § 1.960-2
//! (foreign income taxes deemed paid under § 960(a) and (d)), § 1.960-3
//! (PTEP § 960(b) credit) implement. Proposed regs REG-105479-18
//! (November 29, 2024 published December 2, 2024) modify and replace
//! prior regs to address PTEP sixteen-basket framework under Notice
//! 2019-01.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareholderType {
    /// Domestic C corporation — full eligibility under § 960(c).
    DomesticCCorporation,
    /// Individual / S corp / partnership with § 962 election in place —
    /// eligible.
    IndividualWithSection962Election,
    /// Individual / S corp / partnership without § 962 election — NOT
    /// eligible for § 960(a)/(d) deemed-paid credit.
    IndividualWithoutSection962Election,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InclusionType {
    /// § 951(a)(1) Subpart F inclusion — § 960(a) full creditability.
    SubpartFSection951a,
    /// § 951A GILTI / NCTI inclusion — § 960(d) haircut 80% (pre-OBBBA)
    /// / 90% (post-OBBBA).
    GiltiOrNctiSection951A,
    /// § 959(a) PTEP distribution — § 960(b) credit for withholding tax
    /// plus § 960(d)(4) 10% haircut if PTEP results from § 951A inclusion.
    Section959aPtepDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotEligibleNoSection962Election,
    SubpartFFullDeemedPaidCredit,
    GiltiNcti80PctPreObbba,
    GiltiNcti90PctPostObbba,
    PtepDistributionFullCredit,
    PtepDistributionWithGiltiHaircutOpbba,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section960Input {
    pub shareholder_type: ShareholderType,
    pub inclusion_type: InclusionType,
    pub taxable_year: i32,
    /// CFC foreign income taxes attributable to inclusion in cents.
    pub cfc_foreign_taxes_attributable_cents: u64,
    /// US shareholder's inclusion percentage (proportional share) in basis
    /// points (10_000 = 100%).
    pub inclusion_percentage_bps: u32,
    /// Whether PTEP distribution results from § 951A GILTI / NCTI inclusion
    /// (triggers § 960(d)(4) 10% haircut post-June 28, 2025).
    pub ptep_results_from_section_951a_inclusion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section960Result {
    pub severity: Severity,
    pub deemed_paid_credit_cents: u64,
    pub applicable_rate_bps: u32,
    pub haircut_amount_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const PRE_OBBBA_GILTI_DEEMED_PAID_BPS: u32 = 8_000;
pub const POST_OBBBA_GILTI_DEEMED_PAID_BPS: u32 = 9_000;
pub const OBBBA_EFFECTIVE_YEAR: i32 = 2026;
pub const SECTION_960D4_PTEP_HAIRCUT_BPS: u32 = 1_000;
pub const SECTION_960D4_EFFECTIVE_DATE: &str = "2025-06-28";
pub const NOTICE_2025_77_INTERIM_GUIDANCE: &str = "Notice 2025-77";

pub fn check(input: &Section960Input) -> Section960Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.shareholder_type,
        ShareholderType::IndividualWithoutSection962Election
    ) {
        notes.push(
            "Individual / S corp / partnership US shareholder WITHOUT § 962 election: § 960 \
             deemed-paid FTC mechanism inapplicable per § 960(c). Foreign withholding tax \
             on actual distribution may still be creditable under § 901 (subject to § 904 \
             limit + § 901(k) holding period); but deemed-paid credit for CFC-level taxes \
             not available. Consider [[section_962]] election (iter 510) to be taxed at \
             corporate rate AND access § 960 deemed-paid mechanism."
                .to_string(),
        );
        return empty_result(
            Severity::NotEligibleNoSection962Election,
            input,
            actions,
            notes,
            "26 U.S.C. § 960(c); § 962",
        );
    }

    let inclusion_pct_capped = input.inclusion_percentage_bps.min(10_000);
    let proportional_taxes: u64 = (u128::from(input.cfc_foreign_taxes_attributable_cents)
        * u128::from(inclusion_pct_capped)
        / 10_000) as u64;

    let (rate_bps, severity, deemed_paid, haircut) = match input.inclusion_type {
        InclusionType::SubpartFSection951a => (
            10_000u32,
            Severity::SubpartFFullDeemedPaidCredit,
            proportional_taxes,
            0u64,
        ),
        InclusionType::GiltiOrNctiSection951A => {
            let (rate, sev) = if input.taxable_year >= OBBBA_EFFECTIVE_YEAR {
                (POST_OBBBA_GILTI_DEEMED_PAID_BPS, Severity::GiltiNcti90PctPostObbba)
            } else {
                (PRE_OBBBA_GILTI_DEEMED_PAID_BPS, Severity::GiltiNcti80PctPreObbba)
            };
            let credit: u64 = (u128::from(proportional_taxes) * u128::from(rate) / 10_000) as u64;
            let haircut_amt = proportional_taxes.saturating_sub(credit);
            (rate, sev, credit, haircut_amt)
        }
        InclusionType::Section959aPtepDistribution => {
            if input.ptep_results_from_section_951a_inclusion
                && input.taxable_year >= OBBBA_EFFECTIVE_YEAR
            {
                let rate = 10_000 - SECTION_960D4_PTEP_HAIRCUT_BPS;
                let credit: u64 =
                    (u128::from(proportional_taxes) * u128::from(rate) / 10_000) as u64;
                let haircut_amt = proportional_taxes.saturating_sub(credit);
                (
                    rate,
                    Severity::PtepDistributionWithGiltiHaircutOpbba,
                    credit,
                    haircut_amt,
                )
            } else {
                (
                    10_000u32,
                    Severity::PtepDistributionFullCredit,
                    proportional_taxes,
                    0u64,
                )
            }
        }
    };

    actions.push(format!(
        "§ 960 deemed-paid credit for {:?} inclusion: CFC foreign income taxes of {} cents \
         × inclusion percentage {} bps = {} proportional taxes; applicable rate {} bps = {} \
         cents deemed-paid credit; haircut amount {} cents. Report on Form 1118 Schedule C \
         (for domestic corp) or Form 1116 (for individual with § 962 election). § 904 \
         limitation applies separately per basket (see [[section_904]] iter 516).",
        input.inclusion_type,
        input.cfc_foreign_taxes_attributable_cents,
        inclusion_pct_capped,
        proportional_taxes,
        rate_bps,
        deemed_paid,
        haircut
    ));

    if matches!(
        input.inclusion_type,
        InclusionType::Section959aPtepDistribution
    ) && input.ptep_results_from_section_951a_inclusion
        && input.taxable_year >= OBBBA_EFFECTIVE_YEAR
    {
        actions.push(format!(
            "§ 960(d)(4) PTEP distribution haircut: PTEP results from § 951A GILTI / NCTI \
             inclusion in US shareholder taxable year ending after {}; 10% of foreign income \
             taxes paid or accrued (including § 960(b)(1) deemed-paid) with respect to the \
             § 959(a) distribution DISALLOWED. {} interim guidance applies pending proposed \
             regulations; taxpayers may rely on Section 3 of the Notice for US shareholder \
             taxable years beginning before proposed regulations published.",
            SECTION_960D4_EFFECTIVE_DATE, NOTICE_2025_77_INTERIM_GUIDANCE
        ));
    }

    if matches!(
        input.inclusion_type,
        InclusionType::GiltiOrNctiSection951A
    ) {
        notes.push(format!(
            "§ 960(d) GILTI / NCTI deemed-paid credit: pre-OBBBA rate 80% (per TCJA 2017 \
             original § 960(d)(1)); post-OBBBA rate 90% effective for US shareholder taxable \
             years beginning after December 31, {} per Pub. L. 119-21. Sometimes called the \
             'GILTI haircut'. Inclusion percentage = aggregate net tested income / aggregate \
             tested foreign income taxes — limits credit to share of CFC's tested foreign \
             income taxes attributable to NET tested income (not gross).",
            OBBBA_EFFECTIVE_YEAR - 1
        ));
    }

    notes.push(
        "Coordination with [[section_901]] (general FTC operative provision — iter 518), \
         [[section_904]] (FTC limitation by basket — iter 516; § 951A NCTI has separate \
         basket without carryover per § 904(c)(1) flush language), [[section_951]] (Subpart \
         F inclusion mechanism), [[section_951a]] (GILTI / NCTI inclusion — iter 500), \
         [[section_956]] (CFC US property investment — iter 504), [[section_959]] (PTEP \
         sixteen-basket framework — iter 512 maintained within each § 904(d) basket), \
         [[section_962]] (individual corporate-rate election — iter 510 — only mechanism \
         for individual to access § 960 deemed-paid mechanism), [[section_245a]] (foreign-\
         source DRD — iter 502 § 245A(d) FTC disallowance), [[section_965]] (transition \
         tax — iter 514 § 965(g) FTC denial 55.7%/77.1%), [[section_902]] (REPEALED by \
         TCJA for taxable years of foreign corp beginning after December 31, 2017 — \
         historical pooled FTC framework)."
            .to_string(),
    );

    Section960Result {
        severity,
        deemed_paid_credit_cents: deemed_paid,
        applicable_rate_bps: rate_bps,
        haircut_amount_cents: haircut,
        recommended_actions: actions,
        citation: "26 U.S.C. § 960(a)-(d)(4); § 951; § 951A; § 959; § 962; § 904; Pub. L. 115-97; Pub. L. 119-21; Notice 2025-77; Treas. Reg. § 1.960-2",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section960Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section960Result {
    notes.push(
        "Coordination with [[section_901]] (FTC), [[section_904]] (FTC limit), \
         [[section_951]] (Subpart F), [[section_951a]] (GILTI / NCTI), [[section_956]] \
         (CFC US property), [[section_959]] (PTEP), [[section_962]] (election), \
         [[section_245a]] (DRD), [[section_965]] (transition tax)."
            .to_string(),
    );
    let _ = input;
    Section960Result {
        severity,
        deemed_paid_credit_cents: 0,
        applicable_rate_bps: 0,
        haircut_amount_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section960Input {
        Section960Input {
            shareholder_type: ShareholderType::DomesticCCorporation,
            inclusion_type: InclusionType::SubpartFSection951a,
            taxable_year: 2024,
            cfc_foreign_taxes_attributable_cents: 1_000_000_00,
            inclusion_percentage_bps: 10_000,
            ptep_results_from_section_951a_inclusion: false,
        }
    }

    #[test]
    fn subpart_f_full_deemed_paid_credit() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SubpartFFullDeemedPaidCredit));
        assert_eq!(r.deemed_paid_credit_cents, 1_000_000_00);
        assert_eq!(r.applicable_rate_bps, 10_000);
        assert_eq!(r.haircut_amount_cents, 0);
    }

    #[test]
    fn individual_without_962_election_not_eligible() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::IndividualWithoutSection962Election;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleNoSection962Election));
        assert_eq!(r.deemed_paid_credit_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 962")));
    }

    #[test]
    fn individual_with_962_election_eligible() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::IndividualWithSection962Election;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SubpartFFullDeemedPaidCredit));
    }

    #[test]
    fn gilti_pre_obbba_80_pct_credit() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::GiltiOrNctiSection951A;
        i.taxable_year = 2024;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::GiltiNcti80PctPreObbba));
        assert_eq!(r.applicable_rate_bps, 8_000);
        assert_eq!(r.deemed_paid_credit_cents, 800_000_00);
        assert_eq!(r.haircut_amount_cents, 200_000_00);
    }

    #[test]
    fn gilti_post_obbba_90_pct_credit() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::GiltiOrNctiSection951A;
        i.taxable_year = 2026;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::GiltiNcti90PctPostObbba));
        assert_eq!(r.applicable_rate_bps, 9_000);
        assert_eq!(r.deemed_paid_credit_cents, 900_000_00);
        assert_eq!(r.haircut_amount_cents, 100_000_00);
    }

    #[test]
    fn gilti_2025_still_pre_obbba_80_pct() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::GiltiOrNctiSection951A;
        i.taxable_year = 2025;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::GiltiNcti80PctPreObbba));
        assert_eq!(r.applicable_rate_bps, 8_000);
    }

    #[test]
    fn ptep_distribution_full_credit_when_not_from_gilti() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::Section959aPtepDistribution;
        i.ptep_results_from_section_951a_inclusion = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PtepDistributionFullCredit));
        assert_eq!(r.applicable_rate_bps, 10_000);
        assert_eq!(r.haircut_amount_cents, 0);
    }

    #[test]
    fn ptep_distribution_from_gilti_post_obbba_haircut() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::Section959aPtepDistribution;
        i.ptep_results_from_section_951a_inclusion = true;
        i.taxable_year = 2026;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PtepDistributionWithGiltiHaircutOpbba
        ));
        assert_eq!(r.applicable_rate_bps, 9_000);
        assert_eq!(r.deemed_paid_credit_cents, 900_000_00);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Notice 2025-77")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("2025-06-28")));
    }

    #[test]
    fn ptep_distribution_from_gilti_pre_obbba_no_haircut() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::Section959aPtepDistribution;
        i.ptep_results_from_section_951a_inclusion = true;
        i.taxable_year = 2024;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PtepDistributionFullCredit));
        assert_eq!(r.applicable_rate_bps, 10_000);
    }

    #[test]
    fn inclusion_percentage_50_pct_halves_proportional_taxes() {
        let mut i = baseline();
        i.inclusion_percentage_bps = 5_000;
        let r = check(&i);
        assert_eq!(r.deemed_paid_credit_cents, 500_000_00);
    }

    #[test]
    fn inclusion_percentage_over_100_pct_capped() {
        let mut i = baseline();
        i.inclusion_percentage_bps = 20_000;
        let r = check(&i);
        assert_eq!(r.deemed_paid_credit_cents, 1_000_000_00);
    }

    #[test]
    fn pre_obbba_gilti_haircut_pins_80_pct() {
        assert_eq!(PRE_OBBBA_GILTI_DEEMED_PAID_BPS, 8_000);
    }

    #[test]
    fn post_obbba_gilti_haircut_pins_90_pct() {
        assert_eq!(POST_OBBBA_GILTI_DEEMED_PAID_BPS, 9_000);
    }

    #[test]
    fn obbba_effective_year_pins_2026() {
        assert_eq!(OBBBA_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn section_960d4_ptep_haircut_pins_10_pct() {
        assert_eq!(SECTION_960D4_PTEP_HAIRCUT_BPS, 1_000);
    }

    #[test]
    fn section_960d4_effective_date_pins_2025_06_28() {
        assert_eq!(SECTION_960D4_EFFECTIVE_DATE, "2025-06-28");
    }

    #[test]
    fn notice_2025_77_constant_pinned() {
        assert_eq!(NOTICE_2025_77_INTERIM_GUIDANCE, "Notice 2025-77");
    }

    #[test]
    fn action_references_form_1118_and_1116() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1118")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1116")));
    }

    #[test]
    fn note_pins_gilti_haircut_terminology() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::GiltiOrNctiSection951A;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("GILTI haircut")));
        assert!(r.notes.iter().any(|n| n.contains("Inclusion percentage")));
        assert!(r.notes.iter().any(|n| n.contains("Pub. L. 119-21")));
    }

    #[test]
    fn coordination_note_references_all_international_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_901")));
        assert!(r.notes.iter().any(|n| n.contains("section_904")));
        assert!(r.notes.iter().any(|n| n.contains("section_951")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("section_962")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_965")));
        assert!(r.notes.iter().any(|n| n.contains("section_902")));
    }

    #[test]
    fn citation_pins_960_treas_reg_pub_l_115_97_119_21_notice_2025_77() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 960(a)-(d)(4)"));
        assert!(r.citation.contains("Pub. L. 115-97"));
        assert!(r.citation.contains("Pub. L. 119-21"));
        assert!(r.citation.contains("Notice 2025-77"));
        assert!(r.citation.contains("Treas. Reg. § 1.960-2"));
    }

    #[test]
    fn realistic_corp_with_gilti_75_pct_inclusion_pre_obbba() {
        let mut i = baseline();
        i.inclusion_type = InclusionType::GiltiOrNctiSection951A;
        i.taxable_year = 2024;
        i.inclusion_percentage_bps = 7_500;
        i.cfc_foreign_taxes_attributable_cents = 10_000_000_00;
        let r = check(&i);
        let proportional = 10_000_000_00u64 * 7_500 / 10_000;
        let expected = proportional * 8_000 / 10_000;
        assert_eq!(r.deemed_paid_credit_cents, expected);
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.cfc_foreign_taxes_attributable_cents = u64::MAX / 100;
        let r = check(&i);
        let _ = r.deemed_paid_credit_cents;
    }

    #[test]
    fn zero_foreign_taxes_zero_credit() {
        let mut i = baseline();
        i.cfc_foreign_taxes_attributable_cents = 0;
        let r = check(&i);
        assert_eq!(r.deemed_paid_credit_cents, 0);
    }
}
