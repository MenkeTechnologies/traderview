//! State written-vs-oral lease requirement compliance check.
//!
//! Universal U.S. floor: the Statute of Frauds requires leases of
//! real property for a term exceeding ONE YEAR to be in writing
//! and signed by the party to be charged. Leases of ONE YEAR or
//! LESS may be oral and remain enforceable. State variations add
//! consequences for oral leases that exceed the threshold and
//! provide part-performance escape valves.
//!
//! Five regimes:
//!
//!   - **NewYork** — N.Y. Gen. Oblig. Law § 5-703 (Statute of
//!     Frauds for leases; > 1 year requires writing); part-
//!     performance exception (oral lease enforceable where one
//!     party has taken substantial steps in reliance on the
//!     agreement); N.Y. Gen. Bus. Law § 5-702 (Plain Language Law
//!     — separate content requirement for written consumer
//!     leases; covered separately by plain_language_lease).
//!
//!   - **Illinois** — 740 ILCS 80/2 (Statute of Frauds for leases;
//!     leases exceeding 1 year require writing); UNIQUE
//!     consequence: oral lease for term exceeding 1 year is
//!     treated as a year-to-year tenancy terminable on 60-day
//!     written notice (instead of being entirely unenforceable).
//!
//!   - **California** — Cal. Civ. Code § 1624(a)(3) (Statute of
//!     Frauds — lease for longer than 1 year must be in writing);
//!     Cal. Civ. Code § 1971 (codifies the writing requirement
//!     for property leases).
//!
//!   - **Washington** — RCW 59.18 (Residential Landlord-Tenant
//!     Act; landlord may use oral lease for terms ≤ 1 year); RCW
//!     64.04.010 (Statute of Frauds — > 1 year must be in
//!     writing). RCW 59.18.230 specifically authorizes oral
//!     tenancies for terms ≤ 1 year.
//!
//!   - **Default** — universal Statute of Frauds one-year rule
//!     (UCC § 2A-201 + state common law equivalents); oral lease
//!     exceeding 1 year unenforceable beyond the first year;
//!     part-performance exception varies by state.
//!
//! Citations: N.Y. Gen. Oblig. Law § 5-703 (NY Statute of Frauds
//! for leases); N.Y. Gen. Bus. Law § 5-702 (NY Plain Language Law
//! — content requirement); 740 ILCS 80/2 (IL Statute of Frauds);
//! IL year-to-year + 60-day notice rule (case law); Cal. Civ. Code
//! § 1624(a)(3) + § 1971 (CA Statute of Frauds); RCW 59.18.230 +
//! RCW 64.04.010 (WA Residential Landlord-Tenant Act + Statute of
//! Frauds); UCC § 2A-201 + state common law (Default one-year
//! rule).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYork,
    Illinois,
    California,
    Washington,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Lease term in days.
    pub lease_term_days: u32,
    /// Whether the lease is in writing.
    pub lease_is_written: bool,
    /// Whether one party has taken substantial steps in reliance
    /// on the agreement (NY + most-state part-performance
    /// exception).
    pub substantial_part_performance: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the Statute of Frauds requires the lease to be in
    /// writing (lease term > 1 year).
    pub statute_of_frauds_requires_writing: bool,
    /// True if the lease as-presented is enforceable (written, OR
    /// ≤ 1 year oral, OR part-performance exception applies).
    pub lease_enforceable: bool,
    /// True if the Illinois oral-lease year-to-year conversion
    /// applies (oral lease > 1 year in IL → treated as year-to-
    /// year tenancy).
    pub illinois_year_to_year_conversion_applies: bool,
    /// True if part-performance exception lets an otherwise-
    /// unenforceable oral lease be enforced.
    pub part_performance_exception_applies: bool,
    /// Minimum termination notice required (days) for the IL
    /// year-to-year converted tenancy. Zero where conversion does
    /// not apply.
    pub minimum_termination_notice_days: u32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// One-year threshold for Statute of Frauds writing requirement
/// (365 days). Universal across all 50 states.
pub const STATUTE_OF_FRAUDS_ONE_YEAR_DAYS: u32 = 365;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();

    let exceeds_one_year = input.lease_term_days > STATUTE_OF_FRAUDS_ONE_YEAR_DAYS;
    let statute_of_frauds_requires_writing = exceeds_one_year;

    // Illinois year-to-year conversion for oral > 1-year lease.
    let illinois_conversion =
        matches!(input.regime, Regime::Illinois) && exceeds_one_year && !input.lease_is_written;
    let minimum_termination_notice_days = if illinois_conversion { 60 } else { 0 };

    // Part-performance exception engages where Statute of Frauds
    // would otherwise bar enforcement.
    let part_performance_engages =
        exceeds_one_year && !input.lease_is_written && input.substantial_part_performance;

    // Enforceability.
    // Enforceable if (a) written, (b) ≤ 1 year oral, (c) Illinois
    // year-to-year conversion, or (d) part-performance exception.
    let lease_enforceable = input.lease_is_written
        || !exceeds_one_year
        || illinois_conversion
        || part_performance_engages;

    // Notes.
    if exceeds_one_year && !input.lease_is_written {
        if matches!(input.regime, Regime::Illinois) {
            notes.push(
                "Illinois — oral lease for term exceeding 1 year is treated as a year-to-year \
                 tenancy terminable on 60-day written notice, rather than being entirely \
                 unenforceable. Case law conversion applies."
                    .to_string(),
            );
        } else if part_performance_engages {
            notes.push(
                "Part-performance exception applies — one party has taken substantial steps \
                 in reliance on the oral agreement; courts may enforce the agreement \
                 notwithstanding the Statute of Frauds writing requirement."
                    .to_string(),
            );
        } else {
            notes.push(
                "Statute of Frauds — oral lease for term exceeding 1 year is unenforceable \
                 beyond the first year. Tenant may rely on month-to-month or year-to-year \
                 tenancy at landlord-tenant common-law default."
                    .to_string(),
            );
        }
    } else if !input.lease_is_written && !exceeds_one_year {
        notes.push(
            "Statute of Frauds — oral lease of 1 year or less is binding and enforceable."
                .to_string(),
        );
    }

    if matches!(input.regime, Regime::NewYork) {
        notes.push(
            "N.Y. Gen. Bus. Law § 5-702 Plain Language Law — separate content requirement for \
             written consumer leases (≤ $250,000 dollar value involved); covered separately \
             by plain_language_lease module."
                .to_string(),
        );
    }

    let citation = match input.regime {
        Regime::NewYork => {
            "N.Y. Gen. Oblig. Law § 5-703 (NY Statute of Frauds for leases — > 1 year requires \
             writing); part-performance exception under NY common law; N.Y. Gen. Bus. Law \
             § 5-702 (Plain Language Law — content requirement, separate module)"
        }
        Regime::Illinois => {
            "740 ILCS 80/2 (Illinois Statute of Frauds — > 1 year requires writing); IL case-\
             law conversion: oral lease for term exceeding 1 year is treated as a year-to-year \
             tenancy terminable on 60-day written notice"
        }
        Regime::California => {
            "Cal. Civ. Code § 1624(a)(3) (California Statute of Frauds — lease for longer than \
             1 year must be in writing); § 1971 (codifies writing requirement for property \
             leases)"
        }
        Regime::Washington => {
            "RCW 59.18.230 (Washington Residential Landlord-Tenant Act — landlord may use \
             oral lease for terms ≤ 1 year); RCW 64.04.010 (Statute of Frauds — > 1 year \
             must be in writing)"
        }
        Regime::Default => {
            "Universal U.S. Statute of Frauds (UCC § 2A-201 + state common-law equivalents) — \
             > 1 year requires writing; ≤ 1 year may be oral; part-performance exception \
             varies by state"
        }
    };

    notes.push(
        "Sibling to plain_language_lease (NY GBL § 5-702 + NJ + PA + CT + MN — written-lease \
         content requirement) and lease_translation (CA + IL + NV non-English-language \
         translation requirement). This module addresses the threshold WRITTEN-VS-ORAL \
         question; siblings address content / translation requirements applied to written \
         leases."
            .to_string(),
    );

    CheckResult {
        statute_of_frauds_requires_writing,
        lease_enforceable,
        illinois_year_to_year_conversion_applies: illinois_conversion,
        part_performance_exception_applies: part_performance_engages,
        minimum_termination_notice_days,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime, days: u32, written: bool) -> Input {
        Input {
            regime,
            lease_term_days: days,
            lease_is_written: written,
            substantial_part_performance: false,
        }
    }

    // ── Universal Statute of Frauds one-year rule ──────────────

    #[test]
    fn lease_30_days_oral_enforceable_under_all_regimes() {
        for &regime in &[
            Regime::NewYork,
            Regime::Illinois,
            Regime::California,
            Regime::Washington,
            Regime::Default,
        ] {
            let r = check(&base(regime, 30, false));
            assert!(!r.statute_of_frauds_requires_writing);
            assert!(r.lease_enforceable);
        }
    }

    #[test]
    fn lease_365_days_at_boundary_oral_enforceable() {
        // Statute reads ">1 year" — exactly 365 days is enforceable
        // orally.
        let r = check(&base(Regime::Default, 365, false));
        assert!(!r.statute_of_frauds_requires_writing);
        assert!(r.lease_enforceable);
    }

    #[test]
    fn lease_366_days_above_boundary_requires_writing() {
        let r = check(&base(Regime::Default, 366, false));
        assert!(r.statute_of_frauds_requires_writing);
        assert!(!r.lease_enforceable);
    }

    #[test]
    fn lease_2_year_written_enforceable() {
        let r = check(&base(Regime::Default, 730, true));
        assert!(r.statute_of_frauds_requires_writing);
        assert!(r.lease_enforceable);
    }

    // ── Illinois year-to-year conversion ───────────────────────

    #[test]
    fn illinois_oral_2_year_lease_converts_to_year_to_year() {
        let r = check(&base(Regime::Illinois, 730, false));
        assert!(r.statute_of_frauds_requires_writing);
        assert!(r.illinois_year_to_year_conversion_applies);
        assert!(r.lease_enforceable);
        assert_eq!(r.minimum_termination_notice_days, 60);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Illinois") && n.contains("year-to-year") && n.contains("60-day")));
    }

    #[test]
    fn illinois_oral_6_month_lease_no_conversion() {
        let r = check(&base(Regime::Illinois, 180, false));
        assert!(!r.illinois_year_to_year_conversion_applies);
        assert_eq!(r.minimum_termination_notice_days, 0);
        assert!(r.lease_enforceable);
    }

    #[test]
    fn illinois_written_2_year_lease_no_conversion() {
        let r = check(&base(Regime::Illinois, 730, true));
        assert!(!r.illinois_year_to_year_conversion_applies);
        assert!(r.lease_enforceable);
    }

    // ── Part-performance exception ─────────────────────────────

    #[test]
    fn part_performance_engages_for_oral_lease_exceeding_1_year() {
        let mut i = base(Regime::NewYork, 730, false);
        i.substantial_part_performance = true;
        let r = check(&i);
        assert!(r.part_performance_exception_applies);
        assert!(r.lease_enforceable);
        assert!(r.notes.iter().any(|n| n.contains("Part-performance")));
    }

    #[test]
    fn no_part_performance_oral_lease_exceeding_1_year_unenforceable() {
        let r = check(&base(Regime::NewYork, 730, false));
        assert!(!r.part_performance_exception_applies);
        assert!(!r.lease_enforceable);
    }

    #[test]
    fn part_performance_irrelevant_when_lease_within_one_year() {
        // Oral 30-day lease is enforceable; part-performance is
        // a no-op.
        let mut i = base(Regime::California, 30, false);
        i.substantial_part_performance = true;
        let r = check(&i);
        assert!(r.lease_enforceable);
        assert!(!r.part_performance_exception_applies);
    }

    // ── New York Plain Language Law note ───────────────────────

    #[test]
    fn new_york_plain_language_note_present() {
        let r = check(&base(Regime::NewYork, 365, true));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 5-702") && n.contains("plain_language_lease")));
    }

    // ── California § 1624(a)(3) ────────────────────────────────

    #[test]
    fn california_2_year_oral_unenforceable() {
        let r = check(&base(Regime::California, 730, false));
        assert!(!r.lease_enforceable);
        assert!(r.citation.contains("§ 1624(a)(3)"));
        assert!(r.citation.contains("§ 1971"));
    }

    #[test]
    fn california_oral_within_year_enforceable() {
        let r = check(&base(Regime::California, 200, false));
        assert!(r.lease_enforceable);
    }

    // ── Washington RCW 59.18.230 ────────────────────────────────

    #[test]
    fn washington_oral_lease_within_year_authorized() {
        let r = check(&base(Regime::Washington, 200, false));
        assert!(r.lease_enforceable);
        assert!(r.citation.contains("RCW 59.18.230"));
    }

    #[test]
    fn washington_oral_2_year_unenforceable() {
        let r = check(&base(Regime::Washington, 730, false));
        assert!(!r.lease_enforceable);
    }

    // ── Default Statute of Frauds ──────────────────────────────

    #[test]
    fn default_one_year_rule() {
        let r = check(&base(Regime::Default, 366, false));
        assert!(r.statute_of_frauds_requires_writing);
        assert!(!r.lease_enforceable);
        assert!(r.citation.contains("UCC § 2A-201"));
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn one_year_threshold_strict_boundary_invariant() {
        // 365 days = at boundary = NOT requiring writing (≤ 1 year
        // permitted oral). 366 days = above boundary = requires
        // writing.
        for (days, expected_requires_writing) in
            [(1_u32, false), (365, false), (366, true), (730, true)]
        {
            let r = check(&base(Regime::Default, days, false));
            assert_eq!(
                r.statute_of_frauds_requires_writing, expected_requires_writing,
                "days={} expected_requires_writing={}",
                days, expected_requires_writing,
            );
        }
    }

    #[test]
    fn only_illinois_converts_oral_to_year_to_year_invariant() {
        // Illinois converts; other regimes leave oral > 1-year
        // unenforceable.
        let il = check(&base(Regime::Illinois, 730, false));
        assert!(il.illinois_year_to_year_conversion_applies);

        for &regime in &[
            Regime::NewYork,
            Regime::California,
            Regime::Washington,
            Regime::Default,
        ] {
            let r = check(&base(regime, 730, false));
            assert!(
                !r.illinois_year_to_year_conversion_applies,
                "{:?}: must NOT convert to year-to-year",
                regime,
            );
        }
    }

    #[test]
    fn written_lease_always_enforceable_regardless_of_term_invariant() {
        for term in [30_u32, 365, 730, 3650] {
            for &regime in &[
                Regime::NewYork,
                Regime::Illinois,
                Regime::California,
                Regime::Washington,
                Regime::Default,
            ] {
                let r = check(&base(regime, term, true));
                assert!(
                    r.lease_enforceable,
                    "{:?} term={}: written lease must be enforceable",
                    regime, term,
                );
            }
        }
    }

    #[test]
    fn oral_lease_within_one_year_always_enforceable_invariant() {
        for term in [1_u32, 30, 180, 365] {
            for &regime in &[
                Regime::NewYork,
                Regime::Illinois,
                Regime::California,
                Regime::Washington,
                Regime::Default,
            ] {
                let r = check(&base(regime, term, false));
                assert!(
                    r.lease_enforceable,
                    "{:?} term={}: oral lease ≤ 1 year must be enforceable",
                    regime, term,
                );
            }
        }
    }

    #[test]
    fn oral_lease_above_one_year_unenforceable_unless_il_or_part_performance_invariant() {
        for term in [366_u32, 730, 3650] {
            // Other-state oral > 1 year without part performance →
            // unenforceable.
            for &regime in &[
                Regime::NewYork,
                Regime::California,
                Regime::Washington,
                Regime::Default,
            ] {
                let r = check(&base(regime, term, false));
                assert!(
                    !r.lease_enforceable,
                    "{:?} term={}: oral lease > 1 year without IL conversion or part \
                     performance must NOT be enforceable",
                    regime, term,
                );
            }
            // Illinois always enforces via year-to-year conversion.
            let il = check(&base(Regime::Illinois, term, false));
            assert!(il.lease_enforceable);
        }
    }

    #[test]
    fn part_performance_only_engages_when_oral_and_exceeding_one_year_invariant() {
        // 4-cell truth table: (written, exceeds_year) →
        // part_performance applies iff both NOT-written AND exceeds.
        for written in [false, true] {
            for term in [180_u32, 730] {
                let exceeds = term > 365;
                let mut i = base(Regime::NewYork, term, written);
                i.substantial_part_performance = true;
                let r = check(&i);
                let expected = !written && exceeds;
                assert_eq!(
                    r.part_performance_exception_applies, expected,
                    "written={} exceeds={} expected={}",
                    written, exceeds, expected,
                );
            }
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::NewYork, 730, true))
            .citation
            .contains("§ 5-703"));
        assert!(check(&base(Regime::Illinois, 730, true))
            .citation
            .contains("740 ILCS 80/2"));
        assert!(check(&base(Regime::California, 730, true))
            .citation
            .contains("§ 1624(a)(3)"));
        assert!(check(&base(Regime::Washington, 730, true))
            .citation
            .contains("RCW 59.18.230"));
        assert!(check(&base(Regime::Default, 730, true))
            .citation
            .contains("UCC § 2A-201"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::NewYork,
            Regime::Illinois,
            Regime::California,
            Regime::Washington,
            Regime::Default,
        ] {
            let r = check(&base(regime, 365, true));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("plain_language_lease") && n.contains("lease_translation")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }
}
