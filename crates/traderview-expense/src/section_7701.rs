//! IRC §7701 — Entity classification "check-the-box" regulations.
//!
//! Foundational for any trader forming an LLC, LP, or other
//! pass-through entity. Treasury Regulations §§ 301.7701-2 and
//! 301.7701-3 (the "check-the-box" or CTB regulations, effective
//! 1997-01-01) let eligible entities choose their U.S. tax
//! classification: disregarded entity, partnership, or
//! corporation. Pairs with `section_1361` (S-corp eligibility
//! gate) and `section_351` (formation non-recognition).
//!
//! **Treas. Reg. § 301.7701-2 default classifications**:
//! - Single-member domestic LLC → **disregarded entity** (treated
//!   as part of owner for tax purposes; activity reported on
//!   owner's tax return).
//! - Multi-member domestic LLC / LP / general partnership →
//!   **partnership** (Form 1065).
//! - Federal / state corporate statute entity → **corporation**
//!   automatically (no election available).
//! - § 301.7701-2(b)(8) foreign per-se corporations → corporation
//!   automatically.
//!
//! **Treas. Reg. § 301.7701-3 election mechanism**: eligible
//! entities (those NOT per-se corporations) may file Form 8832 to
//! elect a different classification:
//! - Single-member disregarded entity → may elect to be taxed as
//!   a corporation.
//! - Multi-member partnership → may elect to be taxed as a
//!   corporation.
//! - Once made, the election is effective for the date specified
//!   on Form 8832 (or up to 75 days retroactive / 12 months
//!   prospective).
//!
//! **§ 301.7701-3(c)(1)(iv) 60-month lockout**: once an entity
//! changes its classification by election, it cannot change again
//! for **60 months** after the effective date of the prior
//! election. The lockout is waived if there is a > 50% change in
//! ownership.
//!
//! **Per-se corporation list** (Treas. Reg. § 301.7701-2(b)):
//! includes any entity formed under a federal or state law
//! describing it as "incorporated" or as a "corporation," joint-
//! stock company, banks subject to Title 12 of US Code, insurance
//! companies, business entities wholly owned by States, certain
//! foreign entities listed at (b)(8).
//!
//! Sources:
//! [IRS Practice Unit — Overview of Entity Classification Regulations](https://www.irs.gov/pub/fatca/int_practice_units/ore_c_19_02_01.pdf),
//! [Cornell LII 26 CFR § 301.7701-3](https://www.law.cornell.edu/cfr/text/26/301.7701-3),
//! [IRS Form 8832 — Entity Classification Election](https://www.irs.gov/forms-pubs/about-form-8832),
//! [The Tax Adviser — Classifying business entities under the check-the-box regulations](https://www.thetaxadviser.com/issues/2020/may/classifying-business-entities-check-box-regulations/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Domestic LLC (single-member or multi-member).
    DomesticLlc,
    /// Domestic limited partnership (LP).
    DomesticLimitedPartnership,
    /// Domestic general partnership.
    DomesticGeneralPartnership,
    /// Domestic corporation formed under federal or state
    /// corporate statute — per-se corporation.
    DomesticCorporationStatute,
    /// Foreign entity on the Treas. Reg. § 301.7701-2(b)(8)
    /// per-se corporation list.
    ForeignPerSeCorporation,
    /// Other foreign business entity (potentially eligible).
    ForeignEligibleEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxClassification {
    DisregardedEntity,
    Partnership,
    CCorporation,
    PerSeCorporation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section7701Input {
    pub entity_type: EntityType,
    /// Number of members / owners of the entity.
    pub number_of_members: u32,
    /// True if the entity has filed Form 8832 electing to be
    /// taxed as a corporation.
    pub form_8832_election_to_corporation: bool,
    /// Months elapsed since the most recent prior classification
    /// change election. Used for § 301.7701-3(c)(1)(iv) 60-month
    /// lockout analysis.
    pub months_since_prior_classification_change: u32,
    /// True if there has been a > 50% change in ownership since
    /// the prior election (waives the 60-month lockout).
    pub more_than_50_pct_ownership_change_since_prior_election: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section7701Result {
    pub default_classification: TaxClassification,
    pub current_classification: TaxClassification,
    pub is_per_se_corporation: bool,
    pub eligible_to_elect: bool,
    /// True if the entity can change its classification today
    /// (no 60-month lockout binding OR ownership change waived).
    pub can_change_classification_now: bool,
    /// Months remaining in the 60-month lockout window. Zero if
    /// no lockout binding.
    pub months_remaining_in_lockout: u32,
    pub citation: String,
    pub note: String,
}

const SIXTY_MONTH_LOCKOUT: u32 = 60;

pub fn compute(input: &Section7701Input) -> Section7701Result {
    // Per-se corporation determination.
    let is_per_se = matches!(
        input.entity_type,
        EntityType::DomesticCorporationStatute | EntityType::ForeignPerSeCorporation
    );

    // Default classification under § 301.7701-2(b).
    let default = if is_per_se {
        TaxClassification::PerSeCorporation
    } else if input.number_of_members <= 1 {
        TaxClassification::DisregardedEntity
    } else {
        TaxClassification::Partnership
    };

    // Eligibility to elect (per-se corps cannot elect).
    let eligible_to_elect = !is_per_se;

    // Current classification — Form 8832 election overrides
    // default for eligible entities.
    let current = if !eligible_to_elect {
        TaxClassification::PerSeCorporation
    } else if input.form_8832_election_to_corporation {
        TaxClassification::CCorporation
    } else {
        default
    };

    // 60-month lockout analysis (§ 301.7701-3(c)(1)(iv)).
    let lockout_binding = input.months_since_prior_classification_change < SIXTY_MONTH_LOCKOUT
        && !input.more_than_50_pct_ownership_change_since_prior_election;
    let months_remaining = if lockout_binding {
        SIXTY_MONTH_LOCKOUT - input.months_since_prior_classification_change
    } else {
        0
    };

    // Can change now: must be eligible + not locked out.
    let can_change_now = eligible_to_elect && !lockout_binding;

    let entity_label = match input.entity_type {
        EntityType::DomesticLlc => "domestic LLC",
        EntityType::DomesticLimitedPartnership => "domestic limited partnership",
        EntityType::DomesticGeneralPartnership => "domestic general partnership",
        EntityType::DomesticCorporationStatute => {
            "domestic corporation (federal/state corporate statute)"
        }
        EntityType::ForeignPerSeCorporation => {
            "foreign per-se corporation (Treas. Reg. § 301.7701-2(b)(8))"
        }
        EntityType::ForeignEligibleEntity => "foreign eligible entity",
    };

    let classification_label = |c: TaxClassification| match c {
        TaxClassification::DisregardedEntity => "disregarded entity",
        TaxClassification::Partnership => "partnership (Form 1065)",
        TaxClassification::CCorporation => "C corporation (Form 1120)",
        TaxClassification::PerSeCorporation => "per-se corporation",
    };

    let note = format!(
        "Entity type: {}; members: {}; per-se corp: {}; default classification: {}; Form 8832 election to corporation: {}; current classification: {}; eligible to elect: {}; 60-month lockout binding: {} ({} months remaining); > 50% ownership change waiver: {}.",
        entity_label,
        input.number_of_members,
        is_per_se,
        classification_label(default),
        input.form_8832_election_to_corporation,
        classification_label(current),
        eligible_to_elect,
        lockout_binding,
        months_remaining,
        input.more_than_50_pct_ownership_change_since_prior_election,
    );

    Section7701Result {
        default_classification: default,
        current_classification: current,
        is_per_se_corporation: is_per_se,
        eligible_to_elect,
        can_change_classification_now: can_change_now,
        months_remaining_in_lockout: months_remaining,
        citation:
            "IRC §7701 + Treas. Reg. § 301.7701-2 default classifications (single-member → disregarded entity; multi-member → partnership; federal/state corporate statute → corporation; § 301.7701-2(b)(8) foreign per-se list → corporation) + Treas. Reg. § 301.7701-3 check-the-box election via Form 8832 (eligible entities only) + § 301.7701-3(c)(1)(iv) 60-month (5-year) lockout after classification change (waived if > 50% ownership change); CTB regs effective 1997-01-01; pairs with §1361 S-corp eligibility + §351 formation non-recognition"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_single_member_llc() -> Section7701Input {
        Section7701Input {
            entity_type: EntityType::DomesticLlc,
            number_of_members: 1,
            form_8832_election_to_corporation: false,
            months_since_prior_classification_change: 100,
            more_than_50_pct_ownership_change_since_prior_election: false,
        }
    }

    fn base_multi_member_llc() -> Section7701Input {
        Section7701Input {
            entity_type: EntityType::DomesticLlc,
            number_of_members: 3,
            form_8832_election_to_corporation: false,
            months_since_prior_classification_change: 100,
            more_than_50_pct_ownership_change_since_prior_election: false,
        }
    }

    fn base_per_se_corp() -> Section7701Input {
        Section7701Input {
            entity_type: EntityType::DomesticCorporationStatute,
            number_of_members: 5,
            form_8832_election_to_corporation: false,
            months_since_prior_classification_change: 100,
            more_than_50_pct_ownership_change_since_prior_election: false,
        }
    }

    // ── Default classifications ────────────────────────────────────

    #[test]
    fn single_member_llc_defaults_to_disregarded() {
        let r = compute(&base_single_member_llc());
        assert_eq!(
            r.default_classification,
            TaxClassification::DisregardedEntity
        );
        assert_eq!(
            r.current_classification,
            TaxClassification::DisregardedEntity
        );
        assert!(!r.is_per_se_corporation);
        assert!(r.eligible_to_elect);
    }

    #[test]
    fn multi_member_llc_defaults_to_partnership() {
        let r = compute(&base_multi_member_llc());
        assert_eq!(r.default_classification, TaxClassification::Partnership);
        assert_eq!(r.current_classification, TaxClassification::Partnership);
    }

    #[test]
    fn limited_partnership_defaults_to_partnership() {
        let mut i = base_multi_member_llc();
        i.entity_type = EntityType::DomesticLimitedPartnership;
        let r = compute(&i);
        assert_eq!(r.default_classification, TaxClassification::Partnership);
    }

    #[test]
    fn general_partnership_defaults_to_partnership() {
        let mut i = base_multi_member_llc();
        i.entity_type = EntityType::DomesticGeneralPartnership;
        let r = compute(&i);
        assert_eq!(r.default_classification, TaxClassification::Partnership);
    }

    // ── Per-se corporations ────────────────────────────────────────

    #[test]
    fn domestic_corporation_statute_is_per_se() {
        let r = compute(&base_per_se_corp());
        assert!(r.is_per_se_corporation);
        assert_eq!(
            r.current_classification,
            TaxClassification::PerSeCorporation
        );
        assert!(!r.eligible_to_elect);
        assert!(!r.can_change_classification_now);
    }

    #[test]
    fn foreign_per_se_list_is_per_se() {
        let mut i = base_per_se_corp();
        i.entity_type = EntityType::ForeignPerSeCorporation;
        let r = compute(&i);
        assert!(r.is_per_se_corporation);
        assert!(!r.eligible_to_elect);
    }

    #[test]
    fn per_se_corp_form_8832_election_irrelevant() {
        // Per-se corp's election is irrelevant — already corporation.
        let mut i = base_per_se_corp();
        i.form_8832_election_to_corporation = true;
        let r = compute(&i);
        assert_eq!(
            r.current_classification,
            TaxClassification::PerSeCorporation
        );
    }

    // ── Form 8832 election to corporation ─────────────────────────

    #[test]
    fn single_member_llc_elects_corporation() {
        let mut i = base_single_member_llc();
        i.form_8832_election_to_corporation = true;
        let r = compute(&i);
        assert_eq!(
            r.default_classification,
            TaxClassification::DisregardedEntity
        );
        assert_eq!(r.current_classification, TaxClassification::CCorporation);
    }

    #[test]
    fn multi_member_llc_elects_corporation() {
        let mut i = base_multi_member_llc();
        i.form_8832_election_to_corporation = true;
        let r = compute(&i);
        assert_eq!(r.current_classification, TaxClassification::CCorporation);
    }

    #[test]
    fn foreign_eligible_entity_can_elect() {
        let mut i = base_multi_member_llc();
        i.entity_type = EntityType::ForeignEligibleEntity;
        i.form_8832_election_to_corporation = true;
        let r = compute(&i);
        assert!(r.eligible_to_elect);
        assert_eq!(r.current_classification, TaxClassification::CCorporation);
    }

    // ── 60-month lockout ───────────────────────────────────────────

    #[test]
    fn lockout_not_binding_after_60_months() {
        let mut i = base_single_member_llc();
        i.months_since_prior_classification_change = 60;
        let r = compute(&i);
        assert!(r.can_change_classification_now);
        assert_eq!(r.months_remaining_in_lockout, 0);
    }

    #[test]
    fn lockout_binding_at_59_months() {
        let mut i = base_single_member_llc();
        i.months_since_prior_classification_change = 59;
        let r = compute(&i);
        assert!(!r.can_change_classification_now);
        assert_eq!(r.months_remaining_in_lockout, 1);
    }

    #[test]
    fn lockout_binding_at_0_months() {
        let mut i = base_single_member_llc();
        i.months_since_prior_classification_change = 0;
        let r = compute(&i);
        assert!(!r.can_change_classification_now);
        assert_eq!(r.months_remaining_in_lockout, 60);
    }

    #[test]
    fn ownership_change_50_pct_waives_lockout() {
        let mut i = base_single_member_llc();
        i.months_since_prior_classification_change = 0;
        i.more_than_50_pct_ownership_change_since_prior_election = true;
        let r = compute(&i);
        assert!(r.can_change_classification_now);
        assert_eq!(r.months_remaining_in_lockout, 0);
    }

    // ── Eligibility ────────────────────────────────────────────────

    #[test]
    fn per_se_corp_cannot_change_classification() {
        let r = compute(&base_per_se_corp());
        assert!(!r.eligible_to_elect);
        assert!(!r.can_change_classification_now);
    }

    #[test]
    fn eligible_entity_with_no_recent_election_can_elect() {
        let r = compute(&base_single_member_llc());
        assert!(r.eligible_to_elect);
        assert!(r.can_change_classification_now);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base_single_member_llc());
        assert!(r.citation.contains("§ 301.7701-2"));
        assert!(r.citation.contains("§ 301.7701-3"));
        assert!(r.citation.contains("Form 8832"));
        assert!(r.citation.contains("60-month"));
        assert!(r.citation.contains("1997-01-01"));
        assert!(r.citation.contains("§1361"));
        assert!(r.citation.contains("§351"));
    }

    #[test]
    fn citation_mentions_per_se_foreign_list() {
        let r = compute(&base_single_member_llc());
        assert!(r.citation.contains("§ 301.7701-2(b)(8)"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_describes_default_path() {
        let r = compute(&base_single_member_llc());
        assert!(r.note.contains("disregarded entity"));
    }

    #[test]
    fn note_describes_multi_member_path() {
        let r = compute(&base_multi_member_llc());
        assert!(r.note.contains("partnership"));
    }

    #[test]
    fn note_describes_election_path() {
        let mut i = base_multi_member_llc();
        i.form_8832_election_to_corporation = true;
        let r = compute(&i);
        assert!(r.note.contains("C corporation (Form 1120)"));
    }

    #[test]
    fn note_describes_lockout() {
        let mut i = base_single_member_llc();
        i.months_since_prior_classification_change = 30;
        let r = compute(&i);
        assert!(r.note.contains("60-month lockout binding: true"));
        assert!(r.note.contains("30 months remaining"));
    }
}
