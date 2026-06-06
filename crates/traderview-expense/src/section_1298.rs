//! IRC § 1298 — PFIC attribution rules + special rules + annual
//! reporting.
//!
//! Direct companion to `section_1297` (PFIC classification — which
//! cross-references § 1298(b)(1) purging election in § 1297(d)).
//! § 1298 governs:
//!   (1) WHEN a U.S. person is treated as owning PFIC stock
//!       INDIRECTLY through corporations, partnerships, trusts,
//!       estates, or options;
//!   (2) The PURGING ELECTION mechanism for shedding PFIC taint;
//!   (3) Special deemed-disposition rule for PFIC stock pledged as
//!       loan security; and
//!   (4) Form 8621 ANNUAL REPORTING by U.S. PFIC shareholders.
//!
//! § 1298(a) — attribution rules:
//!
//!   § 1298(a)(2) — CORPORATION attribution: If 50% or more in VALUE
//!     of a corporation's stock is owned (directly or indirectly) by
//!     a person, that person is treated as owning the corporation's
//!     PFIC stock in the proportion of person-owned-value to total
//!     stock value.
//!
//!   § 1298(a)(3) — PARTNERSHIP / ESTATE / TRUST attribution: Stock
//!     owned by a partnership, estate, or trust is considered owned
//!     PROPORTIONATELY by its partners or beneficiaries.
//!
//!   § 1298(a)(4) — OPTIONS attribution: Options to acquire PFIC
//!     stock are treated as ownership to the extent provided in
//!     Treasury regulations.
//!
//! § 1298(b) — special rules:
//!
//!   § 1298(b)(1) — PURGING ELECTION: Taxpayer may elect gain
//!     recognition under rules similar to § 1291(d)(2) — pay current
//!     tax on accumulated PFIC gain to shed PFIC taint going forward.
//!     Pairs with § 1297(d) qualified-portion exception once corp
//!     ceases to be PFIC.
//!
//!   § 1298(b)(6) — PLEDGE AS SECURITY DEEMED DISPOSITION: If a
//!     taxpayer uses any PFIC stock as security for a loan, the
//!     taxpayer is treated as having DISPOSED of such stock — i.e.,
//!     a deemed sale triggering § 1291 excess-distribution
//!     consequences immediately, even though the stock has not
//!     actually changed hands.
//!
//! § 1298(f) — ANNUAL REPORTING: Every U.S. person who is a
//! shareholder of a PFIC must file an annual report (Form 8621)
//! containing such information as the Secretary may require.
//!
//! Citations: 26 U.S.C. § 1298(a)(2) (50% value corporation
//! attribution); § 1298(a)(3) (partnership/estate/trust proportionate
//! attribution); § 1298(a)(4) (options attribution per regulations);
//! § 1298(b)(1) (purging election under § 1291(d)(2)); § 1298(b)(6)
//! (pledge-as-security deemed disposition); § 1298(f) (annual
//! Form 8621 reporting); § 1291 (excess-distribution regime);
//! § 1291(d)(2) (purging-election gain-recognition mechanics);
//! § 1297(d) (once-a-PFIC qualified-portion exception cross-
//! referenced from § 1298(b)(1)).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipType {
    /// Direct U.S.-person ownership of PFIC stock.
    Direct,
    /// Indirect through a 50%+ value corporation (§ 1298(a)(2)).
    ThroughCorporation50PctValue,
    /// Indirect through a corporation owned LESS than 50% by value
    /// — § 1298(a)(2) attribution does NOT engage.
    ThroughCorporationBelow50Pct,
    /// Indirect through a partnership (§ 1298(a)(3)).
    ThroughPartnership,
    /// Indirect through a trust (§ 1298(a)(3)).
    ThroughTrust,
    /// Indirect through an estate (§ 1298(a)(3)).
    ThroughEstate,
    /// Through an option to acquire PFIC stock (§ 1298(a)(4)).
    ThroughOption,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1298Input {
    pub ownership_type: OwnershipType,
    /// Person's ownership in the upstream entity in basis points
    /// × 100 (e.g., 5000 = 50.00%, 7500 = 75.00%). Used for
    /// proportionate attribution under § 1298(a).
    pub upstream_value_ownership_bp: u32,
    /// PFIC stock value held by the upstream entity (cents).
    pub pfic_stock_value_cents: i64,
    /// Whether the taxpayer used the PFIC stock as security for a
    /// loan (§ 1298(b)(6) deemed disposition trigger).
    pub stock_pledged_as_security: bool,
    /// Whether the taxpayer made the § 1298(b)(1) purging election
    /// (under § 1291(d)(2) gain-recognition rules).
    pub purging_election_made: bool,
    /// Whether the U.S. shareholder filed the § 1298(f) annual
    /// Form 8621 report for the taxable year.
    pub annual_form_8621_filed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1298Result {
    /// Attributed PFIC stock ownership value (cents) after applying
    /// § 1298(a) attribution rules.
    pub attributed_pfic_ownership_cents: i64,
    /// True if § 1298(a)(2) 50%+ value corporation attribution
    /// engages.
    pub corporation_attribution_engages: bool,
    /// True if § 1298(a)(3) partnership/trust/estate proportionate
    /// attribution engages.
    pub partnership_trust_estate_attribution_engages: bool,
    /// True if § 1298(a)(4) options attribution engages.
    pub option_attribution_engages: bool,
    /// True if § 1298(b)(6) pledge-as-security deemed-disposition
    /// trigger engages.
    pub deemed_disposition_under_1298b6: bool,
    /// True if § 1298(b)(1) purging election has been made.
    pub purging_election_made: bool,
    /// True if § 1298(f) annual Form 8621 reporting is required.
    pub annual_form_8621_reporting_required: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 1298(a)(2) — 50% value threshold for corporation attribution.
pub const SECTION_1298A2_VALUE_THRESHOLD_BP: u32 = 5_000;

pub fn compute(input: &Section1298Input) -> Section1298Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    // § 1298(a) attribution rules.
    let (corporation_engages, partnership_engages, option_engages, attributed_value) = match input
        .ownership_type
    {
        OwnershipType::Direct => (false, false, false, input.pfic_stock_value_cents.max(0)),
        OwnershipType::ThroughCorporation50PctValue => {
            let engages = input.upstream_value_ownership_bp >= SECTION_1298A2_VALUE_THRESHOLD_BP;
            if engages {
                notes.push(format!(
                    "§ 1298(a)(2) — 50%+ value corporation attribution ENGAGES; person owns \
                         {} basis points ({}%) of corporation's stock; proportionate share of \
                         corp's PFIC stock attributed.",
                    input.upstream_value_ownership_bp,
                    input.upstream_value_ownership_bp as f64 / 100.0,
                ));
            }
            let value = if engages {
                input
                    .pfic_stock_value_cents
                    .max(0)
                    .saturating_mul(input.upstream_value_ownership_bp as i64)
                    / 10_000
            } else {
                0
            };
            (engages, false, false, value)
        }
        OwnershipType::ThroughCorporationBelow50Pct => {
            notes.push(format!(
                "§ 1298(a)(2) attribution does NOT engage — corporation ownership of {} \
                     basis points ({}%) is below 50% value threshold.",
                input.upstream_value_ownership_bp,
                input.upstream_value_ownership_bp as f64 / 100.0,
            ));
            (false, false, false, 0)
        }
        OwnershipType::ThroughPartnership
        | OwnershipType::ThroughTrust
        | OwnershipType::ThroughEstate => {
            let entity_label = match input.ownership_type {
                OwnershipType::ThroughPartnership => "partnership",
                OwnershipType::ThroughTrust => "trust",
                OwnershipType::ThroughEstate => "estate",
                _ => unreachable!(),
            };
            notes.push(format!(
                "§ 1298(a)(3) — {} proportionate attribution ENGAGES regardless of \
                     percentage; partner/beneficiary owns proportionate share of {}'s PFIC \
                     stock at {} basis points ({}%).",
                entity_label,
                entity_label,
                input.upstream_value_ownership_bp,
                input.upstream_value_ownership_bp as f64 / 100.0,
            ));
            let value = input
                .pfic_stock_value_cents
                .max(0)
                .saturating_mul(input.upstream_value_ownership_bp as i64)
                / 10_000;
            (false, true, false, value)
        }
        OwnershipType::ThroughOption => {
            notes.push(
                "§ 1298(a)(4) — option to acquire PFIC stock treated as ownership to the \
                     extent provided in Treasury regulations."
                    .to_string(),
            );
            let value = input
                .pfic_stock_value_cents
                .max(0)
                .saturating_mul(input.upstream_value_ownership_bp as i64)
                / 10_000;
            (false, false, true, value)
        }
    };

    // § 1298(b)(6) pledge-as-security deemed disposition.
    let deemed_disposition = input.stock_pledged_as_security;
    if deemed_disposition {
        notes.push(
            "§ 1298(b)(6) — taxpayer used PFIC stock as security for a loan; treated as having \
             DISPOSED of such stock — deemed sale triggers § 1291 excess-distribution \
             consequences immediately even though stock has not actually changed hands."
                .to_string(),
        );
    }

    // § 1298(b)(1) purging election cross-reference note.
    if input.purging_election_made {
        notes.push(
            "§ 1298(b)(1) — purging election made under rules similar to § 1291(d)(2); \
             taxpayer pays current tax on accumulated PFIC gain to shed PFIC taint going \
             forward. Pairs with § 1297(d) once-a-PFIC qualified-portion exception."
                .to_string(),
        );
    }

    // § 1298(f) annual reporting — required for any direct or
    // attributed PFIC ownership.
    let annual_reporting_required = matches!(
        input.ownership_type,
        OwnershipType::Direct
            | OwnershipType::ThroughCorporation50PctValue
            | OwnershipType::ThroughPartnership
            | OwnershipType::ThroughTrust
            | OwnershipType::ThroughEstate
            | OwnershipType::ThroughOption,
    ) && attributed_value > 0;

    if annual_reporting_required && !input.annual_form_8621_filed {
        violations.push(
            "§ 1298(f) — U.S. PFIC shareholder must file annual Form 8621 report; report not \
             filed for this taxable year."
                .to_string(),
        );
    }

    notes.push(
        "§ 1298 is the attribution + special-rules companion to § 1297 (PFIC classification). \
         See section_1297 for the 75% income test + 50% asset test that determine PFIC \
         status; § 1295 for QEF election; § 1296 for mark-to-market election; § 1291 for the \
         punitive excess-distribution regime."
            .to_string(),
    );

    let citation = "26 U.S.C. § 1298(a)(2) (50% value corporation attribution); § 1298(a)(3) \
                    (partnership/estate/trust proportionate attribution); § 1298(a)(4) (options \
                    attribution per regulations); § 1298(b)(1) (purging election under \
                    § 1291(d)(2)); § 1298(b)(6) (pledge-as-security deemed disposition); \
                    § 1298(f) (annual Form 8621 reporting); § 1291 (excess-distribution \
                    regime); § 1297 (PFIC classification); § 1295 (QEF election); § 1296 \
                    (mark-to-market election)";

    Section1298Result {
        attributed_pfic_ownership_cents: attributed_value,
        corporation_attribution_engages: corporation_engages,
        partnership_trust_estate_attribution_engages: partnership_engages,
        option_attribution_engages: option_engages,
        deemed_disposition_under_1298b6: deemed_disposition,
        purging_election_made: input.purging_election_made,
        annual_form_8621_reporting_required: annual_reporting_required,
        compliant: violations.is_empty(),
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        ownership: OwnershipType,
        bp: u32,
        pfic_value: i64,
        pledged: bool,
        purging: bool,
        filed: bool,
    ) -> Section1298Input {
        Section1298Input {
            ownership_type: ownership,
            upstream_value_ownership_bp: bp,
            pfic_stock_value_cents: pfic_value,
            stock_pledged_as_security: pledged,
            purging_election_made: purging,
            annual_form_8621_filed: filed,
        }
    }

    // ── § 1298(a)(2) 50% value corporation attribution ─────────

    #[test]
    fn corporation_50_pct_attribution_engages_at_boundary() {
        // 50% ownership in corp owning $1000 PFIC stock → $500
        // attributed.
        let r = compute(&input(
            OwnershipType::ThroughCorporation50PctValue,
            5_000,
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.corporation_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 50_000);
    }

    #[test]
    fn corporation_75_pct_attribution_proportional() {
        let r = compute(&input(
            OwnershipType::ThroughCorporation50PctValue,
            7_500,
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.corporation_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 75_000);
    }

    #[test]
    fn corporation_49_pct_below_threshold_no_attribution() {
        let r = compute(&input(
            OwnershipType::ThroughCorporationBelow50Pct,
            4_999,
            100_000,
            false,
            false,
            true,
        ));
        assert!(!r.corporation_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1298(a)(2)") && n.contains("does NOT engage")));
    }

    // ── § 1298(a)(3) partnership/trust/estate attribution ──────

    #[test]
    fn partnership_proportionate_attribution_engages_regardless_of_percentage() {
        // Partnership attribution engages at ANY percentage (unlike
        // 50% corp threshold).
        let r = compute(&input(
            OwnershipType::ThroughPartnership,
            1_000, // 10%
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.partnership_trust_estate_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 10_000);
    }

    #[test]
    fn trust_proportionate_attribution() {
        let r = compute(&input(
            OwnershipType::ThroughTrust,
            2_500,
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.partnership_trust_estate_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 25_000);
    }

    #[test]
    fn estate_proportionate_attribution() {
        let r = compute(&input(
            OwnershipType::ThroughEstate,
            3_300,
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.partnership_trust_estate_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 33_000);
    }

    // ── § 1298(a)(4) options attribution ────────────────────────

    #[test]
    fn option_attribution_engages() {
        let r = compute(&input(
            OwnershipType::ThroughOption,
            10_000,
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.option_attribution_engages);
        assert_eq!(r.attributed_pfic_ownership_cents, 100_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1298(a)(4)") && n.contains("option")));
    }

    // ── Direct ownership ───────────────────────────────────────

    #[test]
    fn direct_ownership_full_value_attributed() {
        let r = compute(&input(
            OwnershipType::Direct,
            10_000,
            100_000,
            false,
            false,
            true,
        ));
        assert_eq!(r.attributed_pfic_ownership_cents, 100_000);
        assert!(!r.corporation_attribution_engages);
        assert!(!r.partnership_trust_estate_attribution_engages);
        assert!(!r.option_attribution_engages);
    }

    // ── § 1298(b)(6) pledge-as-security deemed disposition ─────

    #[test]
    fn pledge_as_security_triggers_deemed_disposition() {
        let r = compute(&input(
            OwnershipType::Direct,
            10_000,
            100_000,
            true,
            false,
            true,
        ));
        assert!(r.deemed_disposition_under_1298b6);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1298(b)(6)") && n.to_lowercase().contains("disposed")));
    }

    #[test]
    fn no_pledge_no_deemed_disposition() {
        let r = compute(&input(
            OwnershipType::Direct,
            10_000,
            100_000,
            false,
            false,
            true,
        ));
        assert!(!r.deemed_disposition_under_1298b6);
    }

    // ── § 1298(b)(1) purging election ──────────────────────────

    #[test]
    fn purging_election_made_note_present() {
        let r = compute(&input(
            OwnershipType::Direct,
            10_000,
            100_000,
            false,
            true,
            true,
        ));
        assert!(r.purging_election_made);
        assert!(r.notes.iter().any(|n| n.contains("§ 1298(b)(1)")
            && n.contains("§ 1291(d)(2)")
            && n.contains("§ 1297(d)")));
    }

    // ── § 1298(f) annual reporting ─────────────────────────────

    #[test]
    fn annual_reporting_required_for_direct_ownership() {
        let mut i = input(OwnershipType::Direct, 10_000, 100_000, false, false, false);
        let r = compute(&i);
        assert!(r.annual_form_8621_reporting_required);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1298(f)") && v.contains("Form 8621")));
        i.annual_form_8621_filed = true;
        assert!(compute(&i).compliant);
    }

    #[test]
    fn annual_reporting_required_for_partnership_attribution() {
        let mut i = input(
            OwnershipType::ThroughPartnership,
            5_000,
            100_000,
            false,
            false,
            false,
        );
        let r = compute(&i);
        assert!(r.annual_form_8621_reporting_required);
        assert!(!r.compliant);
        i.annual_form_8621_filed = true;
        assert!(compute(&i).compliant);
    }

    #[test]
    fn zero_attributed_value_no_reporting_required() {
        // Below-50% corp attribution → 0 attributed → no reporting.
        let r = compute(&input(
            OwnershipType::ThroughCorporationBelow50Pct,
            4_999,
            100_000,
            false,
            false,
            false,
        ));
        assert_eq!(r.attributed_pfic_ownership_cents, 0);
        assert!(!r.annual_form_8621_reporting_required);
        assert!(r.compliant);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn corporation_attribution_only_at_or_above_50_pct_value_invariant() {
        // 4-cell truth table at 50% boundary.
        for (bp, expected_engages) in [
            (4_999_u32, false),
            (5_000, true),
            (5_001, true),
            (10_000, true),
        ] {
            let ownership_type = if bp >= SECTION_1298A2_VALUE_THRESHOLD_BP {
                OwnershipType::ThroughCorporation50PctValue
            } else {
                OwnershipType::ThroughCorporationBelow50Pct
            };
            let r = compute(&input(ownership_type, bp, 100_000, false, false, true));
            assert_eq!(
                r.corporation_attribution_engages, expected_engages,
                "bp={} expected_engages={}",
                bp, expected_engages,
            );
        }
    }

    #[test]
    fn partnership_trust_estate_engages_at_any_percentage_invariant() {
        // Unlike corp (50% threshold), partnership/trust/estate
        // attribution engages at any ownership.
        for &ownership_type in &[
            OwnershipType::ThroughPartnership,
            OwnershipType::ThroughTrust,
            OwnershipType::ThroughEstate,
        ] {
            for bp in [1_u32, 100, 1_000, 5_000, 10_000] {
                let r = compute(&input(ownership_type, bp, 100_000, false, false, true));
                assert!(
                    r.partnership_trust_estate_attribution_engages,
                    "{:?} at {}bp: must engage",
                    ownership_type, bp,
                );
            }
        }
    }

    #[test]
    fn proportionate_attribution_calculation_invariant() {
        // Attributed value = pfic_value × ownership_bp / 10_000.
        for bp in [1_000_u32, 2_500, 5_000, 7_500, 10_000] {
            let r = compute(&input(
                OwnershipType::ThroughPartnership,
                bp,
                100_000,
                false,
                false,
                true,
            ));
            let expected = 100_000_i64 * (bp as i64) / 10_000;
            assert_eq!(
                r.attributed_pfic_ownership_cents, expected,
                "bp={} expected={}",
                bp, expected,
            );
        }
    }

    #[test]
    fn deemed_disposition_only_triggered_by_pledge_invariant() {
        // Deemed disposition fires ONLY when stock_pledged_as_security.
        for pledged in [false, true] {
            let r = compute(&input(
                OwnershipType::Direct,
                10_000,
                100_000,
                pledged,
                false,
                true,
            ));
            assert_eq!(r.deemed_disposition_under_1298b6, pledged);
        }
    }

    #[test]
    fn annual_reporting_violation_iff_required_and_not_filed_invariant() {
        // 4-cell truth table.
        for (required_via_attribution, filed, expected_compliant) in [
            (true, true, true),   // required + filed = compliant
            (true, false, false), // required + not filed = violation
            (false, true, true),  // not required + filed = compliant
            (false, false, true), // not required + not filed = compliant
        ] {
            let ownership_type = if required_via_attribution {
                OwnershipType::Direct
            } else {
                OwnershipType::ThroughCorporationBelow50Pct
            };
            let r = compute(&input(ownership_type, 4_999, 100_000, false, false, filed));
            assert_eq!(
                r.compliant, expected_compliant,
                "required_via_attribution={} filed={} expected_compliant={}",
                required_via_attribution, filed, expected_compliant,
            );
        }
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(
            OwnershipType::Direct,
            10_000,
            100_000,
            false,
            false,
            true,
        ));
        assert!(r.citation.contains("§ 1298(a)(2)"));
        assert!(r.citation.contains("§ 1298(a)(3)"));
        assert!(r.citation.contains("§ 1298(a)(4)"));
        assert!(r.citation.contains("§ 1298(b)(1)"));
        assert!(r.citation.contains("§ 1298(b)(6)"));
        assert!(r.citation.contains("§ 1298(f)"));
        assert!(r.citation.contains("§ 1291"));
        assert!(r.citation.contains("§ 1297"));
        assert!(r.citation.contains("§ 1295"));
        assert!(r.citation.contains("§ 1296"));
    }

    #[test]
    fn sibling_module_note_present_across_all_ownership_types() {
        for ownership_type in [
            OwnershipType::Direct,
            OwnershipType::ThroughCorporation50PctValue,
            OwnershipType::ThroughCorporationBelow50Pct,
            OwnershipType::ThroughPartnership,
            OwnershipType::ThroughTrust,
            OwnershipType::ThroughEstate,
            OwnershipType::ThroughOption,
        ] {
            let r = compute(&input(ownership_type, 5_000, 100_000, false, false, true));
            assert!(
                r.notes.iter().any(|n| n.contains("section_1297")
                    && n.contains("§ 1295")
                    && n.contains("§ 1296")
                    && n.contains("§ 1291")),
                "{:?}: sibling-module note must be present",
                ownership_type,
            );
        }
    }

    #[test]
    fn pledge_as_security_works_with_direct_ownership() {
        let r = compute(&input(
            OwnershipType::Direct,
            10_000,
            100_000,
            true,
            false,
            true,
        ));
        assert!(r.deemed_disposition_under_1298b6);
        assert_eq!(r.attributed_pfic_ownership_cents, 100_000);
    }

    #[test]
    fn pledge_as_security_works_with_attributed_ownership() {
        let r = compute(&input(
            OwnershipType::ThroughPartnership,
            5_000,
            100_000,
            true,
            false,
            true,
        ));
        assert!(r.deemed_disposition_under_1298b6);
        assert_eq!(r.attributed_pfic_ownership_cents, 50_000);
    }
}
