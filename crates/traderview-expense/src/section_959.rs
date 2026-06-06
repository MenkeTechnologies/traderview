//! IRC § 959 — Exclusion from Gross Income of Previously-Taxed Earnings and
//! Profits (PTEP) of a Controlled Foreign Corporation.
//!
//! § 959 prevents double US taxation of CFC earnings that have already been
//! included in a US shareholder's gross income under § 951(a) Subpart F,
//! § 951A GILTI / NCTI, or § 956 US-property investment provisions. When the
//! CFC later actually distributes those amounts, § 959 EXCLUDES the
//! distribution from the recipient US shareholder's gross income.
//!
//! § 959(a)(1) general exclusion: a distribution received by a US shareholder
//! from a CFC is excluded from gross income to the extent it represents
//! "previously-taxed earnings and profits" (PTEP).
//!
//! § 959(c) THREE PTEP CATEGORIES — distribution ordering rules:
//! - § 959(c)(1) PTEP: earnings attributable to amounts that have been or
//!   would have been included in gross income under § 951(a)(1)(B) (now
//!   redesignated post-TCJA — § 956 US property investment); first to be
//!   distributed.
//! - § 959(c)(2) PTEP: earnings attributable to amounts included under §
//!   951(a)(1)(A) Subpart F or § 951A GILTI / NCTI; second to be
//!   distributed.
//! - § 959(c)(3) non-PTEP: untaxed earnings; third to be distributed —
//!   triggers a fully taxable dividend (or qualifies for § 245A 100% DRD
//!   if foreign-source-portion + corporate-US-shareholder + holding-period
//!   conditions met).
//!
//! Notice 2019-01 (December 14, 2018): announced forthcoming regulations
//! requiring US shareholders and CFCs to maintain annual PTEP accounts
//! segregated into SIXTEEN PTEP groups within each § 904 FTC category —
//! nine § 959(c)(1) groups (including § 959(c)(2) PTEP reclassified to
//! § 959(c)(1) when invested in US property) plus seven § 959(c)(2) groups
//! (Subpart F + transition tax § 965 + GILTI / NCTI § 951A).
//!
//! Proposed Regulations REG-105479-18 (November 29, 2024 — published in the
//! Federal Register December 2, 2024) implement the Notice 2019-01 sixteen-
//! basket framework plus shareholder-level and foreign-corporation-level
//! accounting rules.
//!
//! Notice 2024-16 (announcing intent for additional proposed regs on §
//! 961(c) basis in § 332 liquidations and § 368(a)(1) asset reorgs).
//!
//! § 959(d) E&P attributable to PTEP distributions are NOT a dividend for
//! purposes of § 245A 100% DRD pathway — PTEP exclusion replaces the DRD.
//!
//! § 961 STOCK BASIS ADJUSTMENT: PTEP inclusion creates basis increase under
//! § 961(a); actual PTEP distribution creates basis decrease under § 961(b).
//! This ensures that the deferred-but-eventual taxation of CFC earnings does
//! not cause double tax via subsequent stock sale gain recognition.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionRecipient {
    /// US shareholder (corp or individual) eligible for § 959(a)(1)
    /// exclusion.
    UsShareholder,
    /// Less-than-10% shareholder — no Subpart F / GILTI / § 956 inclusions
    /// in prior years, so no PTEP for this shareholder; full distribution
    /// is taxable dividend.
    LessThan10PctNotUsShareholder,
    /// Foreign person — § 959 inapplicable; distribution analyzed under
    /// withholding regime.
    ForeignPerson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotUsShareholderNoExclusion,
    FullyAttributableToSection959C1PtepExcluded,
    FullyAttributableToSection959C2PtepExcluded,
    PartiallyAttributableNonPtepTaxableRemainder,
    FullyAttributableToNonPtepDividend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section959Input {
    pub distribution_recipient: DistributionRecipient,
    /// Total distribution from CFC in cents during the taxable year.
    pub total_distribution_cents: u64,
    /// § 959(c)(1) PTEP balance (US property investment-related, first to
    /// be distributed) in cents.
    pub section_959_c1_ptep_balance_cents: u64,
    /// § 959(c)(2) PTEP balance (Subpart F + GILTI / NCTI, second) in
    /// cents.
    pub section_959_c2_ptep_balance_cents: u64,
    /// § 959(c)(3) non-PTEP E&P (untaxed E&P, third) in cents.
    pub section_959_c3_non_ptep_balance_cents: u64,
    pub taxable_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section959Result {
    pub severity: Severity,
    pub section_959_c1_distributed_excluded_cents: u64,
    pub section_959_c2_distributed_excluded_cents: u64,
    pub section_959_c3_distributed_taxable_cents: u64,
    pub total_excluded_from_gross_income_cents: u64,
    pub total_taxable_dividend_cents: u64,
    pub remaining_c1_balance_cents: u64,
    pub remaining_c2_balance_cents: u64,
    pub remaining_c3_balance_cents: u64,
    pub section_961b_basis_decrease_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const PTEP_TOTAL_BASKETS: u32 = 16;
pub const PTEP_C1_BASKETS: u32 = 9;
pub const PTEP_C2_BASKETS: u32 = 7;
pub const NOTICE_2019_01_DATE: &str = "2018-12-14";
pub const PROPOSED_REGS_REG_105479_18_DATE: &str = "2024-11-29";

pub fn check(input: &Section959Input) -> Section959Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.distribution_recipient,
        DistributionRecipient::LessThan10PctNotUsShareholder | DistributionRecipient::ForeignPerson
    ) {
        notes.push(
            "§ 959(a)(1) exclusion is available only to US shareholders (10% or more direct \
             or indirect ownership) who had prior inclusions creating PTEP. Less-than-10% \
             shareholders had no Subpart F / GILTI / § 956 inclusions and thus have no PTEP \
             — full distribution is a taxable dividend (potentially qualifying for § 1(h)(11) \
             qualified dividend rate if treaty country). Foreign persons receive distribution \
             subject to chapter 3 withholding."
                .to_string(),
        );
        return empty_result(
            Severity::NotUsShareholderNoExclusion,
            input,
            actions,
            notes,
            "26 U.S.C. § 951(b); § 959(a)(1); § 1(h)(11)",
        );
    }

    let dist = input.total_distribution_cents;
    let c1_distributed = dist.min(input.section_959_c1_ptep_balance_cents);
    let after_c1 = dist.saturating_sub(c1_distributed);
    let c2_distributed = after_c1.min(input.section_959_c2_ptep_balance_cents);
    let after_c2 = after_c1.saturating_sub(c2_distributed);
    let c3_distributed = after_c2.min(input.section_959_c3_non_ptep_balance_cents);

    let total_excluded = c1_distributed.saturating_add(c2_distributed);
    let total_taxable = c3_distributed;
    let basis_decrease = total_excluded;

    let remaining_c1 = input
        .section_959_c1_ptep_balance_cents
        .saturating_sub(c1_distributed);
    let remaining_c2 = input
        .section_959_c2_ptep_balance_cents
        .saturating_sub(c2_distributed);
    let remaining_c3 = input
        .section_959_c3_non_ptep_balance_cents
        .saturating_sub(c3_distributed);

    let severity = if total_taxable == 0 && c1_distributed > 0 && c2_distributed == 0 {
        Severity::FullyAttributableToSection959C1PtepExcluded
    } else if total_taxable == 0 && c2_distributed > 0 {
        Severity::FullyAttributableToSection959C2PtepExcluded
    } else if total_taxable > 0 && total_excluded > 0 {
        Severity::PartiallyAttributableNonPtepTaxableRemainder
    } else if total_taxable > 0 && total_excluded == 0 {
        Severity::FullyAttributableToNonPtepDividend
    } else {
        Severity::NotApplicable
    };

    actions.push(format!(
        "Apply § 959(c) distribution-ordering rules: distribution of {} cents attributed \
         first to § 959(c)(1) PTEP (US property reclassification) = {} cents, then § \
         959(c)(2) PTEP (Subpart F + GILTI / NCTI) = {} cents, finally § 959(c)(3) non-\
         PTEP = {} cents. Total excluded from gross income under § 959(a)(1) = {} cents; \
         taxable dividend remainder = {} cents (potentially eligible for § 245A 100% DRD \
         pathway if domestic C corp recipient + foreign-source-portion + § 246(c) holding \
         period satisfied per [[section_245a]]). § 961(b) basis decrease in CFC stock = {} \
         cents (the PTEP-excluded portion).",
        dist,
        c1_distributed,
        c2_distributed,
        c3_distributed,
        total_excluded,
        total_taxable,
        basis_decrease
    ));

    if total_taxable > 0 {
        actions.push(
            "Non-PTEP portion of distribution taxable as dividend; if domestic C corp \
             shareholder, consider § 245A 100% DRD on foreign-source portion (see \
             [[section_245a]] iter 502). § 246(c) holding period must be > 365 days in \
             731-day window beginning 365 days before ex-dividend date. § 245A(d) FTC \
             disallowed on DRD-eligible amount; permanent book-tax difference."
                .to_string(),
        );
    }

    actions.push(format!(
        "Maintain PTEP account in sixteen-basket framework per Notice 2019-01 ({}) plus \
         Proposed Regs REG-105479-18 (published {}) — nine § 959(c)(1) groups (including \
         § 959(c)(2) PTEP reclassified to § 959(c)(1) upon US property investment) plus \
         seven § 959(c)(2) groups (Subpart F + § 965 transition tax + GILTI / NCTI § 951A). \
         Report on Form 5471 Schedule J + Schedule P (PTEP). Subscribe to ongoing PTEP \
         basket within each § 904(d) FTC category.",
        NOTICE_2019_01_DATE, PROPOSED_REGS_REG_105479_18_DATE
    ));

    notes.push(
        "Coordination with [[section_951]] (Subpart F inclusion creating § 959(c)(2) PTEP), \
         [[section_951a]] (GILTI / NCTI inclusion — iter 500 creating § 959(c)(2) PTEP), \
         [[section_956]] (US property investment creating § 959(c)(1) PTEP — iter 504; PTEP \
         reclassification from § 959(c)(2) to § 959(c)(1) when CFC invests existing PTEP \
         in US property), [[section_962]] (iter 510 — § 962 E&P treated as PTEP after \
         inclusion + § 962(d) actual distribution rule creates second layer of tax to extent \
         distribution exceeds cumulative § 962 tax paid), [[section_245a]] (iter 502 — DRD \
         pathway for non-PTEP foreign-source portion), [[section_961]] (basis adjustments \
         coordinating with PTEP — § 961(a) basis increase on inclusion + § 961(b) basis \
         decrease on PTEP distribution), [[section_965]] (transition tax — creates § \
         959(c)(2) PTEP that is treated as included for purposes of § 959), [[section_904]] \
         (FTC limitation baskets — sixteen PTEP groups maintained within each § 904(d) \
         basket)."
            .to_string(),
    );

    Section959Result {
        severity,
        section_959_c1_distributed_excluded_cents: c1_distributed,
        section_959_c2_distributed_excluded_cents: c2_distributed,
        section_959_c3_distributed_taxable_cents: c3_distributed,
        total_excluded_from_gross_income_cents: total_excluded,
        total_taxable_dividend_cents: total_taxable,
        remaining_c1_balance_cents: remaining_c1,
        remaining_c2_balance_cents: remaining_c2,
        remaining_c3_balance_cents: remaining_c3,
        section_961b_basis_decrease_cents: basis_decrease,
        recommended_actions: actions,
        citation: "26 U.S.C. § 959(a)-(d); § 961; Notice 2019-01; REG-105479-18; Notice 2024-16",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section959Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section959Result {
    notes.push(
        "Coordination with [[section_951]] (Subpart F), [[section_951a]] (GILTI / NCTI), \
         [[section_956]] (US property), [[section_962]] (individual election), \
         [[section_245a]] (DRD), [[section_961]] (basis), [[section_965]] (transition \
         tax), [[section_904]] (FTC baskets)."
            .to_string(),
    );
    let _ = input;
    Section959Result {
        severity,
        section_959_c1_distributed_excluded_cents: 0,
        section_959_c2_distributed_excluded_cents: 0,
        section_959_c3_distributed_taxable_cents: 0,
        total_excluded_from_gross_income_cents: 0,
        total_taxable_dividend_cents: 0,
        remaining_c1_balance_cents: 0,
        remaining_c2_balance_cents: 0,
        remaining_c3_balance_cents: 0,
        section_961b_basis_decrease_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section959Input {
        Section959Input {
            distribution_recipient: DistributionRecipient::UsShareholder,
            total_distribution_cents: 100_000_000_00,
            section_959_c1_ptep_balance_cents: 30_000_000_00,
            section_959_c2_ptep_balance_cents: 50_000_000_00,
            section_959_c3_non_ptep_balance_cents: 100_000_000_00,
            taxable_year: 2024,
        }
    }

    #[test]
    fn less_than_10_pct_shareholder_no_exclusion() {
        let mut i = baseline();
        i.distribution_recipient = DistributionRecipient::LessThan10PctNotUsShareholder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotUsShareholderNoExclusion));
        assert_eq!(r.total_excluded_from_gross_income_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 1(h)(11)")));
    }

    #[test]
    fn foreign_person_no_section_959_exclusion() {
        let mut i = baseline();
        i.distribution_recipient = DistributionRecipient::ForeignPerson;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotUsShareholderNoExclusion));
        assert!(r.notes.iter().any(|n| n.contains("chapter 3 withholding")));
    }

    #[test]
    fn distribution_first_attributed_to_section_959_c1() {
        let mut i = baseline();
        i.total_distribution_cents = 20_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_959_c1_distributed_excluded_cents, 20_000_000_00);
        assert_eq!(r.section_959_c2_distributed_excluded_cents, 0);
        assert_eq!(r.section_959_c3_distributed_taxable_cents, 0);
        assert!(matches!(
            r.severity,
            Severity::FullyAttributableToSection959C1PtepExcluded
        ));
    }

    #[test]
    fn distribution_exceeds_c1_then_attributed_to_c2() {
        let mut i = baseline();
        i.total_distribution_cents = 60_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_959_c1_distributed_excluded_cents, 30_000_000_00);
        assert_eq!(r.section_959_c2_distributed_excluded_cents, 30_000_000_00);
        assert_eq!(r.section_959_c3_distributed_taxable_cents, 0);
        assert!(matches!(
            r.severity,
            Severity::FullyAttributableToSection959C2PtepExcluded
        ));
    }

    #[test]
    fn distribution_exceeds_all_ptep_remainder_taxable() {
        let mut i = baseline();
        i.total_distribution_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_959_c1_distributed_excluded_cents, 30_000_000_00);
        assert_eq!(r.section_959_c2_distributed_excluded_cents, 50_000_000_00);
        assert_eq!(r.section_959_c3_distributed_taxable_cents, 20_000_000_00);
        assert!(matches!(
            r.severity,
            Severity::PartiallyAttributableNonPtepTaxableRemainder
        ));
    }

    #[test]
    fn distribution_with_no_ptep_fully_taxable_dividend() {
        let mut i = baseline();
        i.section_959_c1_ptep_balance_cents = 0;
        i.section_959_c2_ptep_balance_cents = 0;
        i.total_distribution_cents = 50_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_959_c3_distributed_taxable_cents, 50_000_000_00);
        assert_eq!(r.total_excluded_from_gross_income_cents, 0);
        assert!(matches!(
            r.severity,
            Severity::FullyAttributableToNonPtepDividend
        ));
    }

    #[test]
    fn section_961b_basis_decrease_equals_total_excluded() {
        let mut i = baseline();
        i.total_distribution_cents = 60_000_000_00;
        let r = check(&i);
        assert_eq!(
            r.section_961b_basis_decrease_cents,
            r.total_excluded_from_gross_income_cents
        );
    }

    #[test]
    fn remaining_balances_correctly_decremented() {
        let mut i = baseline();
        i.total_distribution_cents = 40_000_000_00;
        let r = check(&i);
        assert_eq!(r.remaining_c1_balance_cents, 0);
        assert_eq!(r.remaining_c2_balance_cents, 40_000_000_00);
        assert_eq!(r.remaining_c3_balance_cents, 100_000_000_00);
    }

    #[test]
    fn distribution_exceeds_total_ep_distribution_capped() {
        let mut i = baseline();
        i.total_distribution_cents = 500_000_000_00;
        let r = check(&i);
        let total_distributed = r.section_959_c1_distributed_excluded_cents
            + r.section_959_c2_distributed_excluded_cents
            + r.section_959_c3_distributed_taxable_cents;
        assert_eq!(total_distributed, 180_000_000_00);
    }

    #[test]
    fn ptep_total_baskets_pins_16() {
        assert_eq!(PTEP_TOTAL_BASKETS, 16);
    }

    #[test]
    fn ptep_c1_baskets_pins_9() {
        assert_eq!(PTEP_C1_BASKETS, 9);
    }

    #[test]
    fn ptep_c2_baskets_pins_7() {
        assert_eq!(PTEP_C2_BASKETS, 7);
    }

    #[test]
    fn notice_2019_01_date_pins_2018_12_14() {
        assert_eq!(NOTICE_2019_01_DATE, "2018-12-14");
    }

    #[test]
    fn proposed_regs_date_pins_2024_11_29() {
        assert_eq!(PROPOSED_REGS_REG_105479_18_DATE, "2024-11-29");
    }

    #[test]
    fn action_references_form_5471_schedule_j_and_p() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 5471")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule J")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule P")));
    }

    #[test]
    fn action_references_sixteen_basket_framework() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("sixteen-basket")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Notice 2019-01")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("REG-105479-18")));
    }

    #[test]
    fn action_references_section_245a_drd_pathway_when_taxable() {
        let mut i = baseline();
        i.total_distribution_cents = 200_000_000_00;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 245A 100% DRD")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 246(c)")));
    }

    #[test]
    fn coordination_note_references_951_956_962_245a_961_965() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_951")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_962")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_961")));
        assert!(r.notes.iter().any(|n| n.contains("section_965")));
        assert!(r.notes.iter().any(|n| n.contains("section_904")));
    }

    #[test]
    fn citation_pins_959_961_notice_2019_01_reg_105479_18_notice_2024_16() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 959(a)-(d)"));
        assert!(r.citation.contains("§ 961"));
        assert!(r.citation.contains("Notice 2019-01"));
        assert!(r.citation.contains("REG-105479-18"));
        assert!(r.citation.contains("Notice 2024-16"));
    }

    #[test]
    fn zero_distribution_zero_movement() {
        let mut i = baseline();
        i.total_distribution_cents = 0;
        let r = check(&i);
        assert_eq!(r.total_excluded_from_gross_income_cents, 0);
        assert_eq!(r.total_taxable_dividend_cents, 0);
    }

    #[test]
    fn extreme_distribution_does_not_overflow() {
        let mut i = baseline();
        i.total_distribution_cents = u64::MAX / 2;
        i.section_959_c3_non_ptep_balance_cents = u64::MAX / 2;
        let r = check(&i);
        let _ = r.section_961b_basis_decrease_cents;
    }

    #[test]
    fn no_distribution_no_basis_decrease() {
        let mut i = baseline();
        i.total_distribution_cents = 0;
        let r = check(&i);
        assert_eq!(r.section_961b_basis_decrease_cents, 0);
    }

    #[test]
    fn realistic_50m_distribution_30m_c1_20m_c2() {
        let mut i = baseline();
        i.total_distribution_cents = 50_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_959_c1_distributed_excluded_cents, 30_000_000_00);
        assert_eq!(r.section_959_c2_distributed_excluded_cents, 20_000_000_00);
        assert_eq!(r.section_959_c3_distributed_taxable_cents, 0);
    }

    #[test]
    fn c1_takes_priority_over_c2_per_ordering_rule() {
        let mut i = baseline();
        i.section_959_c1_ptep_balance_cents = 10_000_000_00;
        i.section_959_c2_ptep_balance_cents = 50_000_000_00;
        i.total_distribution_cents = 30_000_000_00;
        let r = check(&i);
        assert_eq!(r.section_959_c1_distributed_excluded_cents, 10_000_000_00);
        assert_eq!(r.section_959_c2_distributed_excluded_cents, 20_000_000_00);
    }
}
