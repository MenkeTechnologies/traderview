//! IRC §448 — Limitation on use of cash method of accounting.
//!
//! Foundational rule for any C-corp or partnership with a C-corp
//! partner. §448(a) generally requires accrual method, but §448(b)(3)
//! creates a SMALL BUSINESS EXCEPTION that lets a taxpayer who meets
//! the §448(c) gross receipts test use the cash method — and as a
//! cascade benefit, escape §263A UNICAP, §471 inventory maintenance,
//! §163(j) business interest limitation, and §460 long-term contract
//! recognition.
//!
//! **§448(c) gross receipts test**: average annual gross receipts
//! over the THREE preceding tax years must not exceed the threshold.
//! Threshold is indexed for inflation from a $25M TCJA-2017 base:
//!
//! | Tax year | Threshold ($M) |
//! |----------|----------------|
//! | 2018     | 25             |
//! | 2024     | 30             |
//! | 2025     | 31             |
//! | 2026     | 32             |
//!
//! Rounded to nearest $1M annually.
//!
//! **§448(a)(3) tax shelter exclusion**: even if the gross receipts
//! test is satisfied, the §448(b)(3) small business exception does
//! NOT apply to any "tax shelter". A tax shelter under §448(d)(3)
//! cross-referencing §461(i)(3) means:
//!
//! - Any enterprise other than a C corporation in which interests
//!   have ever been offered for sale through registered securities
//! - Any syndicate within the meaning of §1256(e)(3)(B) (allocates
//!   > 35% of losses to limited partners)
//! - Any entity described in §6662(d)(2)(C)(ii) (significant tax-
//!   avoidance purpose)
//!
//! **Cascade exemptions when §448(c) is satisfied AND not a tax
//! shelter** — all four flow from the §448 small-business status:
//!
//! - **§263A UNICAP** exempt — no capitalization of indirect costs
//!   into inventory
//! - **§471 inventory** exempt — no requirement to maintain
//!   inventories; treats inventory as supplies / NIMS
//! - **§163(j) business interest** exempt — no 30%-of-ATI limit
//! - **§460 long-term contracts** exempt — completed-contract
//!   method available
//!
//! **§448(c)(2) aggregation rules**: related entities under common
//! control (§52(a) / §52(b)) are aggregated for the gross receipts
//! test. A small business owned 50%+ by a larger affiliated group is
//! tested at the group level.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    CCorp,
    PartnershipWithCCorpPartner,
    SCorp,
    OtherPassThrough,
    SoleProprietor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section448Input {
    pub tax_year: i32,
    pub entity_type: EntityType,
    pub gross_receipts_year_minus_1_dollars: i64,
    pub gross_receipts_year_minus_2_dollars: i64,
    pub gross_receipts_year_minus_3_dollars: i64,
    /// True if entity is a "tax shelter" under §448(d)(3) /
    /// §461(i)(3) (registered-securities enterprise, §1256(e)(3)(B)
    /// syndicate, or §6662(d)(2)(C)(ii) tax-avoidance entity).
    pub is_tax_shelter: bool,
    /// Aggregate gross receipts of all entities under common control
    /// per §448(c)(2) / §52(a)(b). Set equal to the entity's own
    /// gross receipts if no aggregation applies.
    pub aggregated_gross_receipts_year_minus_1_dollars: i64,
    pub aggregated_gross_receipts_year_minus_2_dollars: i64,
    pub aggregated_gross_receipts_year_minus_3_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section448Result {
    pub gross_receipts_threshold_dollars: i64,
    pub average_gross_receipts_dollars: i64,
    pub aggregated_average_gross_receipts_dollars: i64,
    pub meets_gross_receipts_test: bool,
    pub tax_shelter_disqualification: bool,
    pub qualifies_as_small_business: bool,
    pub mandatory_accrual_under_448a: bool,
    pub can_use_cash_method: bool,
    pub exempt_from_263a_unicap: bool,
    pub exempt_from_471_inventory: bool,
    pub exempt_from_163j_business_interest: bool,
    pub exempt_from_460_long_term_contracts: bool,
    pub citation: String,
    pub note: String,
}

/// §448(c) inflation-indexed threshold for a given tax year. Returns
/// the published threshold for known years; falls back to 2025
/// threshold for any year not pinned.
fn gross_receipts_threshold(tax_year: i32) -> i64 {
    match tax_year {
        2018 => 25_000_000,
        2019 => 26_000_000,
        2020 => 26_000_000,
        2021 => 26_000_000,
        2022 => 27_000_000,
        2023 => 29_000_000,
        2024 => 30_000_000,
        2025 => 31_000_000,
        2026 => 32_000_000,
        _ => 31_000_000, // Fall back to 2025 for unknown years
    }
}

pub fn compute(input: &Section448Input) -> Section448Result {
    let threshold = gross_receipts_threshold(input.tax_year);

    let avg = (input.gross_receipts_year_minus_1_dollars
        + input.gross_receipts_year_minus_2_dollars
        + input.gross_receipts_year_minus_3_dollars)
        / 3;
    let aggregated_avg = (input.aggregated_gross_receipts_year_minus_1_dollars
        + input.aggregated_gross_receipts_year_minus_2_dollars
        + input.aggregated_gross_receipts_year_minus_3_dollars)
        / 3;

    // Test is run on AGGREGATED receipts per §448(c)(2).
    let meets_test = aggregated_avg <= threshold;

    let tax_shelter_disq = input.is_tax_shelter;
    let qualifies = meets_test && !tax_shelter_disq;

    // §448(a) mandatory accrual applies to C corps and partnerships
    // with a C corp partner UNLESS qualifying as small business.
    let mandatory_accrual_default = matches!(
        input.entity_type,
        EntityType::CCorp | EntityType::PartnershipWithCCorpPartner
    );
    let mandatory_accrual = mandatory_accrual_default && !qualifies;
    let can_use_cash = !mandatory_accrual;

    // Cascade exemptions all gate on §448(c) qualification + not tax
    // shelter.
    let exempt_263a = qualifies;
    let exempt_471 = qualifies;
    let exempt_163j = qualifies;
    let exempt_460 = qualifies;

    let note = if tax_shelter_disq {
        format!(
            "§448(a)(3) TAX SHELTER DISQUALIFICATION: gross receipts test {} (avg ${} ≤ ${} threshold) but §448(d)(3) tax-shelter status blocks small business exception; mandatory accrual under §448(a), §263A UNICAP, §471 inventory, §163(j) business interest, §460 long-term contracts ALL apply.",
            if meets_test { "satisfied" } else { "NOT satisfied" },
            aggregated_avg,
            threshold,
        )
    } else if !meets_test {
        format!(
            "§448(c) gross receipts test FAILED: 3-year aggregated average ${} > ${} threshold (tax year {}). Mandatory accrual + §263A + §471 + §163(j) + §460 ALL apply.",
            aggregated_avg, threshold, input.tax_year,
        )
    } else if qualifies && mandatory_accrual_default {
        format!(
            "§448(c) SATISFIED ({} avg ≤ ${} threshold): C-corp / partnership escapes mandatory accrual under §448(b)(3) small business exception. Cascade exemptions: §263A UNICAP, §471 inventory, §163(j) business interest, §460 long-term contracts.",
            aggregated_avg, threshold,
        )
    } else {
        format!(
            "§448(c) SATISFIED ({} avg ≤ ${} threshold): pass-through / sole proprietor — §448(a) mandatory accrual not applicable; all cascade exemptions available.",
            aggregated_avg, threshold,
        )
    };

    Section448Result {
        gross_receipts_threshold_dollars: threshold,
        average_gross_receipts_dollars: avg,
        aggregated_average_gross_receipts_dollars: aggregated_avg,
        meets_gross_receipts_test: meets_test,
        tax_shelter_disqualification: tax_shelter_disq,
        qualifies_as_small_business: qualifies,
        mandatory_accrual_under_448a: mandatory_accrual,
        can_use_cash_method: can_use_cash,
        exempt_from_263a_unicap: exempt_263a,
        exempt_from_471_inventory: exempt_471,
        exempt_from_163j_business_interest: exempt_163j,
        exempt_from_460_long_term_contracts: exempt_460,
        citation:
            "IRC §448(a) mandatory accrual for C-corps / partnerships with C-corp partner; §448(b)(3) small business exception; §448(c) gross receipts test (3-year average ≤ inflation-indexed threshold, $25M TCJA base); §448(c)(2) aggregation under §52(a)/(b); §448(a)(3) tax shelter exclusion; coordination §263A, §471, §163(j), §460"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_ccorp_base() -> Section448Input {
        Section448Input {
            tax_year: 2025,
            entity_type: EntityType::CCorp,
            gross_receipts_year_minus_1_dollars: 10_000_000,
            gross_receipts_year_minus_2_dollars: 10_000_000,
            gross_receipts_year_minus_3_dollars: 10_000_000,
            is_tax_shelter: false,
            aggregated_gross_receipts_year_minus_1_dollars: 10_000_000,
            aggregated_gross_receipts_year_minus_2_dollars: 10_000_000,
            aggregated_gross_receipts_year_minus_3_dollars: 10_000_000,
        }
    }

    // Threshold table.

    #[test]
    fn threshold_2024_30m() {
        let mut i = small_ccorp_base();
        i.tax_year = 2024;
        let r = compute(&i);
        assert_eq!(r.gross_receipts_threshold_dollars, 30_000_000);
    }

    #[test]
    fn threshold_2025_31m() {
        let r = compute(&small_ccorp_base());
        assert_eq!(r.gross_receipts_threshold_dollars, 31_000_000);
    }

    #[test]
    fn threshold_2026_32m() {
        let mut i = small_ccorp_base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert_eq!(r.gross_receipts_threshold_dollars, 32_000_000);
    }

    #[test]
    fn threshold_2018_base_25m() {
        let mut i = small_ccorp_base();
        i.tax_year = 2018;
        let r = compute(&i);
        assert_eq!(r.gross_receipts_threshold_dollars, 25_000_000);
    }

    #[test]
    fn threshold_unknown_year_falls_back_to_2025() {
        let mut i = small_ccorp_base();
        i.tax_year = 2030;
        let r = compute(&i);
        assert_eq!(r.gross_receipts_threshold_dollars, 31_000_000);
    }

    // Small business qualification.

    #[test]
    fn small_ccorp_qualifies_can_use_cash_full_cascade() {
        let r = compute(&small_ccorp_base());
        assert!(r.meets_gross_receipts_test);
        assert!(r.qualifies_as_small_business);
        assert!(r.can_use_cash_method);
        assert!(!r.mandatory_accrual_under_448a);
        assert!(r.exempt_from_263a_unicap);
        assert!(r.exempt_from_471_inventory);
        assert!(r.exempt_from_163j_business_interest);
        assert!(r.exempt_from_460_long_term_contracts);
    }

    #[test]
    fn large_ccorp_fails_test_mandatory_accrual() {
        let mut i = small_ccorp_base();
        i.aggregated_gross_receipts_year_minus_1_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 50_000_000;
        let r = compute(&i);
        assert!(!r.meets_gross_receipts_test);
        assert!(r.mandatory_accrual_under_448a);
        assert!(!r.can_use_cash_method);
        assert!(!r.exempt_from_263a_unicap);
        assert!(!r.exempt_from_163j_business_interest);
    }

    #[test]
    fn ccorp_at_exact_threshold_qualifies() {
        // Threshold is ≤, so exact match qualifies.
        let mut i = small_ccorp_base();
        i.aggregated_gross_receipts_year_minus_1_dollars = 31_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 31_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 31_000_000;
        let r = compute(&i);
        assert!(r.meets_gross_receipts_test);
    }

    #[test]
    fn ccorp_one_dollar_over_threshold_fails() {
        let mut i = small_ccorp_base();
        i.aggregated_gross_receipts_year_minus_1_dollars = 31_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 31_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 31_000_003; // avg = 31M+1
        let r = compute(&i);
        assert!(!r.meets_gross_receipts_test);
    }

    // Tax shelter disqualification.

    #[test]
    fn tax_shelter_disqualifies_even_when_test_satisfied() {
        let mut i = small_ccorp_base();
        i.is_tax_shelter = true;
        let r = compute(&i);
        assert!(r.meets_gross_receipts_test);
        assert!(r.tax_shelter_disqualification);
        assert!(!r.qualifies_as_small_business);
        assert!(r.mandatory_accrual_under_448a);
        assert!(!r.exempt_from_263a_unicap);
        assert!(r.note.contains("TAX SHELTER DISQUALIFICATION"));
    }

    // Aggregation.

    #[test]
    fn aggregation_can_blow_through_threshold() {
        // Entity-level avg $10M, but aggregated avg $40M → fails.
        let mut i = small_ccorp_base();
        i.aggregated_gross_receipts_year_minus_1_dollars = 40_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 40_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 40_000_000;
        let r = compute(&i);
        assert!(!r.meets_gross_receipts_test);
        assert_eq!(r.average_gross_receipts_dollars, 10_000_000);
        assert_eq!(r.aggregated_average_gross_receipts_dollars, 40_000_000);
    }

    // Entity type behavior.

    #[test]
    fn partnership_with_ccorp_partner_subject_to_448a() {
        let mut i = small_ccorp_base();
        i.entity_type = EntityType::PartnershipWithCCorpPartner;
        // Without §448(c) qualification → mandatory accrual.
        i.aggregated_gross_receipts_year_minus_1_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 50_000_000;
        let r = compute(&i);
        assert!(r.mandatory_accrual_under_448a);
    }

    #[test]
    fn s_corp_not_subject_to_448a_mandatory_accrual() {
        let mut i = small_ccorp_base();
        i.entity_type = EntityType::SCorp;
        i.aggregated_gross_receipts_year_minus_1_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 50_000_000;
        let r = compute(&i);
        // S-corp not in the §448(a) mandatory list — can use cash
        // even at $50M. But cascade exemptions still require §448(c).
        assert!(r.can_use_cash_method);
        assert!(!r.exempt_from_163j_business_interest);
    }

    #[test]
    fn sole_proprietor_can_use_cash_regardless() {
        let mut i = small_ccorp_base();
        i.entity_type = EntityType::SoleProprietor;
        i.aggregated_gross_receipts_year_minus_1_dollars = 100_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 100_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 100_000_000;
        let r = compute(&i);
        assert!(r.can_use_cash_method);
        assert!(!r.exempt_from_163j_business_interest); // Still subject to §163(j)
    }

    // Average computation.

    #[test]
    fn average_computed_from_3_prior_years() {
        let mut i = small_ccorp_base();
        i.gross_receipts_year_minus_1_dollars = 5_000_000;
        i.gross_receipts_year_minus_2_dollars = 10_000_000;
        i.gross_receipts_year_minus_3_dollars = 15_000_000;
        i.aggregated_gross_receipts_year_minus_1_dollars = 5_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 10_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 15_000_000;
        let r = compute(&i);
        assert_eq!(r.average_gross_receipts_dollars, 10_000_000);
        assert_eq!(r.aggregated_average_gross_receipts_dollars, 10_000_000);
    }

    // Notes.

    #[test]
    fn qualified_ccorp_note_describes_cascade_exemptions() {
        let r = compute(&small_ccorp_base());
        assert!(r.note.contains("§448(c) SATISFIED"));
        assert!(r.note.contains("§263A"));
        assert!(r.note.contains("§471"));
        assert!(r.note.contains("§163(j)"));
        assert!(r.note.contains("§460"));
    }

    #[test]
    fn failed_test_note_describes_all_provisions_apply() {
        let mut i = small_ccorp_base();
        i.aggregated_gross_receipts_year_minus_1_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 50_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 50_000_000;
        let r = compute(&i);
        assert!(r.note.contains("gross receipts test FAILED"));
        assert!(r.note.contains("ALL apply"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&small_ccorp_base());
        assert!(r.citation.contains("§448(a)"));
        assert!(r.citation.contains("§448(b)(3)"));
        assert!(r.citation.contains("§448(c)"));
        assert!(r.citation.contains("§448(c)(2)"));
        assert!(r.citation.contains("§448(a)(3)"));
        assert!(r.citation.contains("TCJA"));
        assert!(r.citation.contains("§263A"));
        assert!(r.citation.contains("§163(j)"));
    }

    // Precision / boundary.

    #[test]
    fn very_large_aggregated_receipts_handled() {
        let mut i = small_ccorp_base();
        i.aggregated_gross_receipts_year_minus_1_dollars = 1_000_000_000;
        i.aggregated_gross_receipts_year_minus_2_dollars = 1_000_000_000;
        i.aggregated_gross_receipts_year_minus_3_dollars = 1_000_000_000;
        let r = compute(&i);
        assert!(!r.meets_gross_receipts_test);
        assert_eq!(r.aggregated_average_gross_receipts_dollars, 1_000_000_000);
    }

    #[test]
    fn zero_receipts_qualifies() {
        let mut i = small_ccorp_base();
        i.gross_receipts_year_minus_1_dollars = 0;
        i.gross_receipts_year_minus_2_dollars = 0;
        i.gross_receipts_year_minus_3_dollars = 0;
        i.aggregated_gross_receipts_year_minus_1_dollars = 0;
        i.aggregated_gross_receipts_year_minus_2_dollars = 0;
        i.aggregated_gross_receipts_year_minus_3_dollars = 0;
        let r = compute(&i);
        assert!(r.meets_gross_receipts_test);
        assert!(r.qualifies_as_small_business);
    }
}
