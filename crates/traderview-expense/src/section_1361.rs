//! IRC §1361 — S corporation defined / eligibility requirements.
//!
//! Trader-relevant for active traders who structure their trading
//! activity through an S corporation (commonly paired with §475(f)
//! trader election). §1361(b)(1) sets out the 6-prong "small
//! business corporation" eligibility test; failing ANY prong
//! results in either ineligibility to elect or termination of an
//! existing S election. §1361(b)(2) carves out 4 categorically
//! ineligible corporation types.
//!
//! **§1361(b)(1) eligibility prongs** — all 6 must be met:
//!
//! - **(A)** Must be a **domestic** corporation (organized in U.S.)
//! - **(B)** Must NOT be an "ineligible corporation" under
//!   §1361(b)(2): financial institutions using the reserve method,
//!   insurance companies subject to Subchapter L, foreign sales
//!   corporations (§922), DISCs
//! - **(C)** ≤ **100 shareholders** (after applying §1361(c)(1)
//!   family attribution — family treated as 1 shareholder)
//! - **(D)** Shareholders limited to individuals + qualifying
//!   estates + qualifying trusts + certain exempt organizations
//!   (NO partnerships, NO non-S corporations as shareholders)
//! - **(E)** No nonresident alien (NRA) shareholders — even ONE
//!   share owned by an NRA terminates the election
//! - **(F)** Only ONE class of stock outstanding (voting-rights
//!   differences ARE permitted; economic differences NOT)
//!
//! **§1361(c)(1) family attribution**: "members of a family" =
//! common ancestor + any lineal descendant + any spouse or former
//! spouse of common ancestor or any lineal descendant. The family
//! is treated as ONE shareholder for the 100-shareholder cap, but
//! each family member must independently meet the
//! permissible-shareholder requirements.
//!
//! **§1361(c)(2) permitted trusts**: grantor trust, ESBT
//! (electing small business trust), QSST (qualified subchapter S
//! trust), voting trust, certain testamentary trusts.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 1361](https://www.law.cornell.edu/uscode/text/26/1361),
//! [Cornell LII 26 CFR § 1.1361-1 S corporation defined](https://www.law.cornell.edu/cfr/text/26/1.1361-1),
//! [Accounting Insights — IRC 1361 and S Corporation definition](https://accountinginsights.org/what-is-irc-1361-and-how-does-it-define-s-corporations/),
//! [Foster Tax Law — Subchapter S Part VIII Shareholder Eligibility](https://www.foster.com/larry-s-tax-law/subchapter-s-part-8-shareholder-eligibility).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IneligibilityReason {
    /// §1361(b)(1)(A) failed — not a domestic corporation.
    NotDomesticCorporation,
    /// §1361(b)(2) — financial institution using reserve method.
    IneligibleFinancialInstitution,
    /// §1361(b)(2) — insurance company subject to Subchapter L.
    IneligibleInsuranceCompany,
    /// §1361(b)(2) — foreign sales corporation or DISC.
    IneligibleForeignSalesOrDisc,
    /// §1361(b)(1)(B) — other ineligible corporation type.
    OtherIneligibleCorporationType,
    /// §1361(b)(1)(C) — more than 100 shareholders after family
    /// attribution.
    TooManyShareholders,
    /// §1361(b)(1)(D) — has a non-individual / non-qualifying
    /// shareholder (partnership / non-S corporation).
    IneligibleNonIndividualShareholder,
    /// §1361(b)(1)(E) — has a nonresident alien shareholder.
    NonresidentAlienShareholder,
    /// §1361(b)(1)(F) — has more than one class of stock.
    MoreThanOneClassOfStock,
    /// No ineligibility — corporation qualifies.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1361Input {
    /// True if the corporation is organized in the United States
    /// (§1361(b)(1)(A) domestic prong).
    pub is_domestic_corporation: bool,
    /// True if the corporation is a financial institution using
    /// the §585 reserve method for bad debt (§1361(b)(2)(A)).
    pub is_financial_institution_with_reserve_method: bool,
    /// True if the corporation is an insurance company subject
    /// to Subchapter L (§1361(b)(2)(B)).
    pub is_insurance_company_subchapter_l: bool,
    /// True if the corporation is a foreign sales corp (§922)
    /// or DISC (§1361(b)(2)(C)).
    pub is_foreign_sales_corp_or_disc: bool,
    /// Number of shareholders BEFORE applying §1361(c)(1) family
    /// attribution.
    pub total_shareholders_pre_attribution: u32,
    /// Number of distinct families represented (each family is
    /// treated as 1 shareholder under §1361(c)(1)).
    pub distinct_families_for_attribution: u32,
    /// Number of shareholders OUTSIDE any family group (counted
    /// individually).
    pub non_family_shareholders: u32,
    /// True if any shareholder is a partnership, non-S corporation,
    /// or other non-individual / non-qualifying-trust entity.
    pub has_partnership_or_non_qualifying_corp_shareholder: bool,
    /// True if any shareholder is a nonresident alien.
    pub has_nonresident_alien_shareholder: bool,
    /// True if the corporation has more than one class of stock
    /// outstanding (differences in voting rights are NOT counted
    /// as separate classes).
    pub has_more_than_one_class_of_stock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1361Result {
    pub qualifies_as_s_corporation: bool,
    pub ineligibility_reason: IneligibilityReason,
    /// Total shareholder count AFTER §1361(c)(1) family attribution.
    pub effective_shareholder_count_post_attribution: u32,
    /// The 6 prong checks individually:
    pub passes_domestic_prong: bool,
    pub passes_not_ineligible_corp_prong: bool,
    pub passes_shareholder_count_prong: bool,
    pub passes_eligible_shareholder_type_prong: bool,
    pub passes_no_nra_prong: bool,
    pub passes_one_class_of_stock_prong: bool,
    pub citation: String,
    pub note: String,
}

const SHAREHOLDER_CAP: u32 = 100;

pub fn compute(input: &Section1361Input) -> Section1361Result {
    // §1361(c)(1) attribution: each family = 1 shareholder; plus
    // non-family shareholders counted individually.
    let effective_shareholders = input
        .distinct_families_for_attribution
        .saturating_add(input.non_family_shareholders);

    let passes_domestic = input.is_domestic_corporation;
    let passes_not_ineligible = !input.is_financial_institution_with_reserve_method
        && !input.is_insurance_company_subchapter_l
        && !input.is_foreign_sales_corp_or_disc;
    let passes_count = effective_shareholders <= SHAREHOLDER_CAP;
    let passes_eligible_type = !input.has_partnership_or_non_qualifying_corp_shareholder;
    let passes_no_nra = !input.has_nonresident_alien_shareholder;
    let passes_one_class = !input.has_more_than_one_class_of_stock;

    // Priority order for ineligibility reason reporting:
    // (A) → (B) financial → (B) insurance → (B) FSC/DISC → (C)
    // count → (D) type → (E) NRA → (F) classes.
    let reason = if !passes_domestic {
        IneligibilityReason::NotDomesticCorporation
    } else if input.is_financial_institution_with_reserve_method {
        IneligibilityReason::IneligibleFinancialInstitution
    } else if input.is_insurance_company_subchapter_l {
        IneligibilityReason::IneligibleInsuranceCompany
    } else if input.is_foreign_sales_corp_or_disc {
        IneligibilityReason::IneligibleForeignSalesOrDisc
    } else if !passes_count {
        IneligibilityReason::TooManyShareholders
    } else if !passes_eligible_type {
        IneligibilityReason::IneligibleNonIndividualShareholder
    } else if !passes_no_nra {
        IneligibilityReason::NonresidentAlienShareholder
    } else if !passes_one_class {
        IneligibilityReason::MoreThanOneClassOfStock
    } else {
        IneligibilityReason::None
    };

    let qualifies = matches!(reason, IneligibilityReason::None);

    let reason_label = match reason {
        IneligibilityReason::None => "qualifies as S corporation",
        IneligibilityReason::NotDomesticCorporation => {
            "§1361(b)(1)(A) failed — not a domestic corporation"
        }
        IneligibilityReason::IneligibleFinancialInstitution => {
            "§1361(b)(2)(A) failed — financial institution using §585 reserve method"
        }
        IneligibilityReason::IneligibleInsuranceCompany => {
            "§1361(b)(2)(B) failed — insurance company subject to Subchapter L"
        }
        IneligibilityReason::IneligibleForeignSalesOrDisc => {
            "§1361(b)(2)(C) failed — foreign sales corporation or DISC"
        }
        IneligibilityReason::OtherIneligibleCorporationType => {
            "§1361(b)(1)(B) failed — ineligible corporation type"
        }
        IneligibilityReason::TooManyShareholders => {
            "§1361(b)(1)(C) failed — more than 100 shareholders after family attribution"
        }
        IneligibilityReason::IneligibleNonIndividualShareholder => {
            "§1361(b)(1)(D) failed — partnership / non-S corporation / non-qualifying shareholder"
        }
        IneligibilityReason::NonresidentAlienShareholder => {
            "§1361(b)(1)(E) failed — nonresident alien shareholder (even ONE share terminates)"
        }
        IneligibilityReason::MoreThanOneClassOfStock => {
            "§1361(b)(1)(F) failed — more than one class of stock outstanding"
        }
    };

    let note = format!(
        "Total shareholders pre-attribution: {}; distinct families: {}; non-family: {}; effective shareholders post-§1361(c)(1) attribution: {} (cap {}); ineligible-corp tests {}; non-individual shareholder {}; NRA shareholder {}; > 1 class of stock {}; result: {}.",
        input.total_shareholders_pre_attribution,
        input.distinct_families_for_attribution,
        input.non_family_shareholders,
        effective_shareholders,
        SHAREHOLDER_CAP,
        if passes_not_ineligible { "PASS" } else { "FAIL" },
        if passes_eligible_type { "absent" } else { "PRESENT" },
        if passes_no_nra { "absent" } else { "PRESENT" },
        if passes_one_class { "absent" } else { "PRESENT" },
        reason_label,
    );

    Section1361Result {
        qualifies_as_s_corporation: qualifies,
        ineligibility_reason: reason,
        effective_shareholder_count_post_attribution: effective_shareholders,
        passes_domestic_prong: passes_domestic,
        passes_not_ineligible_corp_prong: passes_not_ineligible,
        passes_shareholder_count_prong: passes_count,
        passes_eligible_shareholder_type_prong: passes_eligible_type,
        passes_no_nra_prong: passes_no_nra,
        passes_one_class_of_stock_prong: passes_one_class,
        citation:
            "IRC §1361(b)(1) S-corp eligibility 6 prongs: (A) domestic corporation + (B) not ineligible corp under §1361(b)(2) + (C) ≤ 100 shareholders after §1361(c)(1) family attribution + (D) shareholders limited to individuals / qualifying estates / qualifying trusts / certain exempt orgs (no partnerships / no non-S corps) + (E) no nonresident alien shareholders + (F) only one class of stock (voting-rights differences ARE permitted; economic differences are NOT); §1361(b)(2) ineligible corporations include financial institutions using §585 reserve method + insurance companies subject to Subchapter L + foreign sales corps + DISCs; §1361(c)(1) family attribution treats common ancestor + lineal descendants + spouses as ONE shareholder; §1361(c)(2) permitted trusts (grantor / ESBT / QSST / voting)"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section1361Input {
        Section1361Input {
            is_domestic_corporation: true,
            is_financial_institution_with_reserve_method: false,
            is_insurance_company_subchapter_l: false,
            is_foreign_sales_corp_or_disc: false,
            total_shareholders_pre_attribution: 50,
            distinct_families_for_attribution: 10,
            non_family_shareholders: 40,
            has_partnership_or_non_qualifying_corp_shareholder: false,
            has_nonresident_alien_shareholder: false,
            has_more_than_one_class_of_stock: false,
        }
    }

    // ── Baseline qualifies ─────────────────────────────────────────

    #[test]
    fn baseline_qualifies() {
        let r = compute(&base());
        assert!(r.qualifies_as_s_corporation);
        assert_eq!(r.ineligibility_reason, IneligibilityReason::None);
        assert_eq!(r.effective_shareholder_count_post_attribution, 50);
    }

    // ── §1361(b)(1)(A) domestic prong ──────────────────────────────

    #[test]
    fn foreign_corporation_fails_domestic_prong() {
        let mut i = base();
        i.is_domestic_corporation = false;
        let r = compute(&i);
        assert!(!r.qualifies_as_s_corporation);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::NotDomesticCorporation
        );
    }

    // ── §1361(b)(2) ineligible corporation types ──────────────────

    #[test]
    fn financial_institution_reserve_method_fails() {
        let mut i = base();
        i.is_financial_institution_with_reserve_method = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::IneligibleFinancialInstitution
        );
    }

    #[test]
    fn insurance_company_subchapter_l_fails() {
        let mut i = base();
        i.is_insurance_company_subchapter_l = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::IneligibleInsuranceCompany
        );
    }

    #[test]
    fn foreign_sales_corp_fails() {
        let mut i = base();
        i.is_foreign_sales_corp_or_disc = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::IneligibleForeignSalesOrDisc
        );
    }

    // ── §1361(b)(1)(C) shareholder count + §1361(c)(1) attribution ─

    #[test]
    fn exactly_100_shareholders_qualifies() {
        let mut i = base();
        i.distinct_families_for_attribution = 0;
        i.non_family_shareholders = 100;
        let r = compute(&i);
        assert!(r.qualifies_as_s_corporation);
    }

    #[test]
    fn one_over_100_fails() {
        let mut i = base();
        i.distinct_families_for_attribution = 0;
        i.non_family_shareholders = 101;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::TooManyShareholders
        );
    }

    #[test]
    fn family_attribution_collapses_50_members_to_1() {
        // 1 family with 50 members + 49 non-family = 1 + 49 = 50.
        let mut i = base();
        i.total_shareholders_pre_attribution = 99;
        i.distinct_families_for_attribution = 1;
        i.non_family_shareholders = 49;
        let r = compute(&i);
        assert_eq!(r.effective_shareholder_count_post_attribution, 50);
        assert!(r.qualifies_as_s_corporation);
    }

    #[test]
    fn family_attribution_lets_large_family_avoid_cap() {
        // 1 family with 200 members + 50 non-family = 1 + 50 = 51.
        let mut i = base();
        i.total_shareholders_pre_attribution = 250;
        i.distinct_families_for_attribution = 1;
        i.non_family_shareholders = 50;
        let r = compute(&i);
        assert_eq!(r.effective_shareholder_count_post_attribution, 51);
        assert!(r.qualifies_as_s_corporation);
    }

    // ── §1361(b)(1)(D) eligible shareholder type ──────────────────

    #[test]
    fn partnership_shareholder_fails() {
        let mut i = base();
        i.has_partnership_or_non_qualifying_corp_shareholder = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::IneligibleNonIndividualShareholder
        );
    }

    // ── §1361(b)(1)(E) no NRA ──────────────────────────────────────

    #[test]
    fn nonresident_alien_shareholder_fails() {
        let mut i = base();
        i.has_nonresident_alien_shareholder = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::NonresidentAlienShareholder
        );
    }

    // ── §1361(b)(1)(F) one class of stock ──────────────────────────

    #[test]
    fn two_classes_of_stock_fails() {
        let mut i = base();
        i.has_more_than_one_class_of_stock = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::MoreThanOneClassOfStock
        );
    }

    // ── Priority ordering of failures ──────────────────────────────

    #[test]
    fn domestic_failure_short_circuits_other_failures() {
        // Failing domestic + NRA + too many shareholders → reports
        // domestic only (highest priority).
        let mut i = base();
        i.is_domestic_corporation = false;
        i.has_nonresident_alien_shareholder = true;
        i.distinct_families_for_attribution = 0;
        i.non_family_shareholders = 200;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::NotDomesticCorporation
        );
    }

    #[test]
    fn financial_institution_short_circuits_count_failure() {
        let mut i = base();
        i.is_financial_institution_with_reserve_method = true;
        i.distinct_families_for_attribution = 0;
        i.non_family_shareholders = 200;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::IneligibleFinancialInstitution
        );
    }

    #[test]
    fn count_failure_reported_before_nra_failure() {
        let mut i = base();
        i.distinct_families_for_attribution = 0;
        i.non_family_shareholders = 200;
        i.has_nonresident_alien_shareholder = true;
        let r = compute(&i);
        assert_eq!(
            r.ineligibility_reason,
            IneligibilityReason::TooManyShareholders
        );
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_6_prongs() {
        let r = compute(&base());
        assert!(r.citation.contains("§1361(b)(1)(A)") || r.citation.contains("(A) domestic"));
        assert!(r.citation.contains("(B) not ineligible"));
        assert!(r.citation.contains("(C) ≤ 100 shareholders"));
        assert!(r.citation.contains("(D) shareholders"));
        assert!(r.citation.contains("(E) no nonresident alien"));
        assert!(r.citation.contains("(F) only one class"));
        assert!(r.citation.contains("§1361(c)(1)"));
        assert!(r.citation.contains("§1361(c)(2)"));
    }

    #[test]
    fn citation_mentions_voting_rights_differences_permitted() {
        let r = compute(&base());
        assert!(r.citation.contains("voting-rights differences ARE permitted"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_qualifying_path_mentions_qualifies() {
        let r = compute(&base());
        assert!(r.note.contains("qualifies as S corporation"));
    }

    #[test]
    fn note_failure_path_describes_reason() {
        let mut i = base();
        i.has_nonresident_alien_shareholder = true;
        let r = compute(&i);
        assert!(r.note.contains("§1361(b)(1)(E)"));
        assert!(r.note.contains("nonresident alien"));
    }

    #[test]
    fn note_reports_effective_shareholder_count() {
        let mut i = base();
        i.distinct_families_for_attribution = 5;
        i.non_family_shareholders = 20;
        let r = compute(&i);
        assert!(r.note.contains("effective shareholders post-§1361(c)(1) attribution: 25"));
    }
}
