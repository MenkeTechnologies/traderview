//! IRC § 7408 — Actions to enjoin specified conduct related to
//! tax shelters and reportable transactions.
//!
//! Completes the preparer-and-promoter enforcement cluster:
//! § 6694 (preparer substantive position, iter 254) plus § 6695
//! (preparer procedural failures, iter 256) plus § 6700 (promoter
//! penalties, iter 258) plus § 6701 (aiding and abetting, iter
//! 260) plus § 7408 (THIS MODULE — equitable injunction remedy).
//! § 7408 is the EQUITABLE ENFORCEMENT TOOL the IRS uses to STOP
//! ongoing promoter/aider conduct — not just penalize past
//! conduct. § 7408 supplements (does not replace) the penalty
//! sections; the IRS may pursue penalties and injunction in
//! parallel.
//!
//! § 7408(a) AUTHORITY — civil action in name of United States
//! to enjoin any person from further engaging in "specified
//! conduct" may be commenced at the request of the Secretary
//! of the Treasury.
//!
//! § 7408(b) TWO-PRONG INJUNCTION TEST — the court may issue
//! the injunction if BOTH prongs are satisfied:
//!   (1) Person has engaged in specified conduct (any of the
//!       enumerated penalty-triggering categories below); AND
//!   (2) Injunctive relief is APPROPRIATE TO PREVENT
//!       RECURRENCE of such conduct.
//!
//! § 7408(c) SPECIFIED CONDUCT — any action or failure to take
//! action that is subject to penalty under any of: § 6700
//! (abusive tax shelter promotion); § 6701 (aiding and abetting
//! understatement of tax liability); § 6707 (failure to furnish
//! information about reportable transactions — material advisor
//! disclosure failure); § 6708 (failure to maintain list of
//! advisees — material advisor list maintenance failure); or
//! regulations issued under 31 U.S.C. § 330 (Circular 230 —
//! practice of attorneys, CPAs, EAs, and other practitioners
//! before the IRS).
//!
//! § 7408(d) VENUE — action brought in U.S. District Court for
//! the district where the person resides, has their principal
//! place of business, or has engaged in specified conduct.
//!
//! § 7408(e) SPECIAL VENUE RULE — if any U.S. citizen or
//! resident does NOT reside in and does NOT have principal
//! place of business in any U.S. judicial district, that person
//! is treated as residing in the DISTRICT OF COLUMBIA.
//!
//! § 7408 + § 7402(a) — jurisdiction over a § 7408 action is
//! exercised SEPARATELY and apart from any other action brought
//! by the United States against the same person. Injunction
//! does not preclude criminal prosecution, civil penalty
//! assessment, or other equitable remedies.
//!
//! Citations: 26 U.S.C. § 7408 (general); 26 U.S.C. § 7408(a)
//! (Secretary request + civil action authority); 26 U.S.C.
//! § 7408(b) (two-prong test — conduct + appropriateness to
//! prevent recurrence); 26 U.S.C. § 7408(c) (specified conduct
//! categories — § 6700/§ 6701/§ 6707/§ 6708/Circular 230);
//! 26 U.S.C. § 7408(d) (district court venue); 26 U.S.C.
//! § 7408(e) (DC residency rule for non-district residents);
//! 26 U.S.C. § 7402(a) (jurisdiction independent of other
//! actions); 26 U.S.C. § 6700 (promoter penalties); 26 U.S.C.
//! § 6701 (aiding and abetting); 26 U.S.C. § 6707 (material
//! advisor failure-to-furnish); 26 U.S.C. § 6708 (material
//! advisor list-maintenance failure); 31 U.S.C. § 330 (Treasury
//! authority to regulate practice before IRS — Circular 230);
//! IRM 5.20.7 (monitoring of promoter/preparer injunctions);
//! IRM 5.20.8 (promoter/preparer investigations).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section7408Input {
    /// § 7408(a) — true if the Secretary has authorized the
    /// civil action.
    pub requested_by_secretary: bool,
    /// § 7408(c)(1) — § 6700 abusive tax shelter promotion
    /// conduct.
    pub engaged_in_section_6700_conduct: bool,
    /// § 7408(c)(2) — § 6701 aiding and abetting conduct.
    pub engaged_in_section_6701_conduct: bool,
    /// § 7408(c)(3) — § 6707 material advisor failure-to-furnish
    /// conduct.
    pub engaged_in_section_6707_conduct: bool,
    /// § 7408(c)(4) — § 6708 material advisor list-maintenance
    /// failure.
    pub engaged_in_section_6708_conduct: bool,
    /// § 7408(c)(5) — violation of 31 U.S.C. § 330 regulations
    /// (Circular 230).
    pub violated_circular_230: bool,
    /// § 7408(b)(2) — court finding that injunctive relief is
    /// appropriate to prevent recurrence.
    pub injunction_appropriate_to_prevent_recurrence: bool,
    /// True if person resides in or has principal place of
    /// business in a U.S. judicial district.
    pub person_resides_in_us_district: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7408Result {
    /// True if both § 7408(b) prongs satisfied and Secretary
    /// has requested action.
    pub injunction_available: bool,
    /// True if any of the 5 specified-conduct categories engages.
    pub specified_conduct_engaged: bool,
    /// List of penalty-statute categories the conduct triggers.
    pub conduct_categories: Vec<String>,
    /// True if person treated as residing in District of
    /// Columbia under § 7408(e).
    pub venue_treats_as_dc: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section7408Input) -> Section7408Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();
    let mut conduct_categories: Vec<String> = Vec::new();

    if input.engaged_in_section_6700_conduct {
        conduct_categories.push("§ 6700 — abusive tax shelter promotion".to_string());
    }
    if input.engaged_in_section_6701_conduct {
        conduct_categories
            .push("§ 6701 — aiding and abetting understatement of tax liability".to_string());
    }
    if input.engaged_in_section_6707_conduct {
        conduct_categories.push(
            "§ 6707 — material advisor failure to furnish information about reportable \
             transactions"
                .to_string(),
        );
    }
    if input.engaged_in_section_6708_conduct {
        conduct_categories
            .push("§ 6708 — material advisor failure to maintain advisee list".to_string());
    }
    if input.violated_circular_230 {
        conduct_categories
            .push("31 U.S.C. § 330 — violation of Circular 230 (practice before IRS)".to_string());
    }

    let specified_conduct_engaged = !conduct_categories.is_empty();

    // § 7408(b) two-prong test for injunction availability.
    let injunction_available = input.requested_by_secretary
        && specified_conduct_engaged
        && input.injunction_appropriate_to_prevent_recurrence;

    // § 7408(e) — non-district resident venue.
    let venue_treats_as_dc = !input.person_resides_in_us_district;

    if injunction_available {
        violations.push(format!(
            "§ 7408(a) + (b) — INJUNCTION AVAILABLE. Secretary has requested action; \
             person engaged in {} specified-conduct categor{}; injunction appropriate to \
             prevent recurrence. Court may permanently enjoin further conduct under \
             § 7408(b). Action proceeds in U.S. District Court ({}).",
            conduct_categories.len(),
            if conduct_categories.len() == 1 {
                "y"
            } else {
                "ies"
            },
            if venue_treats_as_dc {
                "District of Columbia per § 7408(e) — non-resident provision"
            } else {
                "person's district of residence, principal place of business, OR district \
                 where conduct occurred per § 7408(d)"
            },
        ));
    }

    // Notes.
    if !input.requested_by_secretary {
        notes.push(
            "§ 7408(a) — injunction action requires REQUEST OF THE SECRETARY of the \
             Treasury. Without Secretary authorization, no § 7408 action available; only \
             civil penalties under § 6700/§ 6701/§ 6707/§ 6708 are available."
                .to_string(),
        );
    } else if !specified_conduct_engaged {
        notes.push(
            "§ 7408(c) — no specified-conduct category engaged. Specified conduct = \
             § 6700 promotion OR § 6701 aiding/abetting OR § 6707 material-advisor \
             failure-to-furnish OR § 6708 list-maintenance failure OR 31 U.S.C. § 330 \
             Circular 230 violation. Without specified conduct, no § 7408 action \
             available."
                .to_string(),
        );
    } else if !input.injunction_appropriate_to_prevent_recurrence {
        notes.push(
            "§ 7408(b)(2) — second prong NOT satisfied. Court must find injunctive \
             relief appropriate to PREVENT RECURRENCE. Past conduct alone does not \
             trigger § 7408 injunction; risk of repeat is required. Common factors: \
             continued solicitation, ongoing scheme operation, lack of remorse, prior \
             similar conduct."
                .to_string(),
        );
    }

    // Conduct-specific notes.
    if specified_conduct_engaged {
        notes.push(format!(
            "§ 7408(c) specified-conduct categories engaged: {}.",
            conduct_categories.join("; "),
        ));
    }

    if venue_treats_as_dc && injunction_available {
        notes.push(
            "§ 7408(e) — person is a U.S. citizen or resident NOT residing in any U.S. \
             judicial district. Treated as residing in District of Columbia for venue \
             purposes; action may be brought in U.S. District Court for D.C."
                .to_string(),
        );
    }

    notes.push(
        "§ 7408 + § 7402(a) — court may exercise jurisdiction over the § 7408 action \
         SEPARATELY and APART from any other action brought by the United States. \
         Injunction does NOT preclude (1) civil penalty assessment under § 6700/§ 6701/\
         § 6707/§ 6708, (2) criminal prosecution under § 7201/§ 7203/§ 7206 + 18 U.S.C. \
         § 371 conspiracy, (3) Circular 230 disciplinary proceedings, or (4) other \
         equitable remedies. § 7408 is SUPPLEMENTAL — IRS may pursue parallel tracks."
            .to_string(),
    );

    notes.push(
        "Sibling preparer + promoter enforcement cluster — § 7408 is the EQUITABLE \
         REMEDY (injunction). Penalty cluster: § 6694 (preparer substantive position, \
         iter 254); § 6695 (preparer procedural failures, iter 256); § 6700 (promoter \
         penalties, iter 258); § 6701 (aiding and abetting, iter 260). Material advisor \
         cluster (referenced as specified conduct): § 6707 (failure to furnish — \
         distinct from § 6111 timely-disclosure-required); § 6708 (list maintenance \
         failure — distinct from § 6112 list-maintenance-required). Circular 230 (31 \
         U.S.C. § 330) governs practice of attorneys, CPAs, EAs before IRS. § 7407 is \
         the parallel injunction provision specifically for income tax return \
         preparers (vs. § 7408's broader promoter/aider scope)."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section7408Result {
        injunction_available,
        specified_conduct_engaged,
        conduct_categories,
        venue_treats_as_dc,
        compliant,
        violations,
        citation: "26 U.S.C. § 7408 (general); 26 U.S.C. § 7408(a) (Secretary request + \
                   civil action authority); 26 U.S.C. § 7408(b) (two-prong test); \
                   26 U.S.C. § 7408(b)(1) (specified-conduct engagement prong); \
                   26 U.S.C. § 7408(b)(2) (appropriateness-to-prevent-recurrence prong); \
                   26 U.S.C. § 7408(c) (specified conduct definition); 26 U.S.C. \
                   § 7408(c)(1)-(5) (§ 6700 + § 6701 + § 6707 + § 6708 + Circular 230 \
                   categories); 26 U.S.C. § 7408(d) (district court venue); 26 U.S.C. \
                   § 7408(e) (DC residency rule); 26 U.S.C. § 7402(a) (jurisdiction \
                   independent of other actions); 26 U.S.C. § 6700 (promoter); \
                   26 U.S.C. § 6701 (aiding and abetting); 26 U.S.C. § 6707 (material \
                   advisor failure-to-furnish); 26 U.S.C. § 6708 (material advisor list \
                   maintenance); 31 U.S.C. § 330 (Treasury Circular 230); 26 U.S.C. \
                   § 7407 (parallel preparer-specific injunction); IRM 5.20.7 + IRM \
                   5.20.8 (promoter/preparer investigation procedures)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section7408Input {
        Section7408Input {
            requested_by_secretary: false,
            engaged_in_section_6700_conduct: false,
            engaged_in_section_6701_conduct: false,
            engaged_in_section_6707_conduct: false,
            engaged_in_section_6708_conduct: false,
            violated_circular_230: false,
            injunction_appropriate_to_prevent_recurrence: false,
            person_resides_in_us_district: true,
        }
    }

    // ── Three-element gating — Secretary request + conduct + appropriate ─

    #[test]
    fn no_secretary_request_no_injunction() {
        let mut b = input();
        b.engaged_in_section_6700_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(!r.injunction_available);
        assert!(r.compliant);
    }

    #[test]
    fn no_specified_conduct_no_injunction() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(!r.injunction_available);
        assert!(!r.specified_conduct_engaged);
    }

    #[test]
    fn no_appropriateness_no_injunction() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = false;
        let r = compute(&b);
        assert!(!r.injunction_available);
        assert!(r.specified_conduct_engaged);
    }

    #[test]
    fn all_three_elements_engages_injunction() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert!(!r.compliant);
    }

    // ── Each specified-conduct category engages independently ─

    #[test]
    fn section_6700_conduct_engages() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert!(r.conduct_categories.iter().any(|c| c.contains("§ 6700")));
    }

    #[test]
    fn section_6701_conduct_engages() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6701_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert!(r.conduct_categories.iter().any(|c| c.contains("§ 6701")));
    }

    #[test]
    fn section_6707_conduct_engages() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6707_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert!(r.conduct_categories.iter().any(|c| c.contains("§ 6707")));
    }

    #[test]
    fn section_6708_conduct_engages() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6708_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert!(r.conduct_categories.iter().any(|c| c.contains("§ 6708")));
    }

    #[test]
    fn circular_230_violation_engages() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.violated_circular_230 = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert!(r
            .conduct_categories
            .iter()
            .any(|c| c.contains("31 U.S.C. § 330")));
    }

    // ── Multiple categories aggregate ─────────────────────────

    #[test]
    fn multiple_categories_listed_individually() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.engaged_in_section_6701_conduct = true;
        b.engaged_in_section_6708_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert_eq!(r.conduct_categories.len(), 3);
    }

    #[test]
    fn all_five_categories_engaged() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.engaged_in_section_6701_conduct = true;
        b.engaged_in_section_6707_conduct = true;
        b.engaged_in_section_6708_conduct = true;
        b.violated_circular_230 = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        let r = compute(&b);
        assert!(r.injunction_available);
        assert_eq!(r.conduct_categories.len(), 5);
    }

    // ── § 7408(e) venue rule ─────────────────────────────────

    #[test]
    fn non_resident_treated_as_dc_venue() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        b.person_resides_in_us_district = false;
        let r = compute(&b);
        assert!(r.venue_treats_as_dc);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("District of Columbia")));
    }

    #[test]
    fn resident_uses_normal_venue() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        b.injunction_appropriate_to_prevent_recurrence = true;
        b.person_resides_in_us_district = true;
        let r = compute(&b);
        assert!(!r.venue_treats_as_dc);
    }

    // ── Multi-regime invariants ─────────────────────────────

    #[test]
    fn three_required_elements_truth_table() {
        // 8-cell sweep: Secretary × conduct × appropriateness.
        let cells = [
            (false, false, false, false),
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (true, true, false, false),
            (true, false, true, false),
            (false, true, true, false),
            (true, true, true, true), // only all-three engages
        ];
        for (sec, conduct, app, expected) in cells.iter() {
            let mut b = input();
            b.requested_by_secretary = *sec;
            b.engaged_in_section_6700_conduct = *conduct;
            b.injunction_appropriate_to_prevent_recurrence = *app;
            let r = compute(&b);
            assert_eq!(
                r.injunction_available, *expected,
                "sec={} conduct={} app={}",
                sec, conduct, app
            );
        }
    }

    #[test]
    fn each_conduct_category_independently_engages_invariant() {
        // 5-prong sweep — each specified-conduct category alone
        // satisfies § 7408(b)(1) when other elements present.
        let setters: Vec<Box<dyn Fn(&mut Section7408Input)>> = vec![
            Box::new(|b| b.engaged_in_section_6700_conduct = true),
            Box::new(|b| b.engaged_in_section_6701_conduct = true),
            Box::new(|b| b.engaged_in_section_6707_conduct = true),
            Box::new(|b| b.engaged_in_section_6708_conduct = true),
            Box::new(|b| b.violated_circular_230 = true),
        ];
        for setter in &setters {
            let mut b = input();
            b.requested_by_secretary = true;
            b.injunction_appropriate_to_prevent_recurrence = true;
            setter(&mut b);
            let r = compute(&b);
            assert!(r.specified_conduct_engaged);
            assert!(r.injunction_available);
        }
    }

    #[test]
    fn venue_dc_only_when_not_us_resident_invariant() {
        for in_district in [true, false] {
            let mut b = input();
            b.person_resides_in_us_district = in_district;
            let r = compute(&b);
            assert_eq!(r.venue_treats_as_dc, !in_district);
        }
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 7408"));
        assert!(r.citation.contains("§ 7408(a)"));
        assert!(r.citation.contains("§ 7408(b)"));
        assert!(r.citation.contains("§ 7408(b)(1)"));
        assert!(r.citation.contains("§ 7408(b)(2)"));
        assert!(r.citation.contains("§ 7408(c)"));
        assert!(r.citation.contains("§ 7408(c)(1)-(5)"));
        assert!(r.citation.contains("§ 7408(d)"));
        assert!(r.citation.contains("§ 7408(e)"));
        assert!(r.citation.contains("§ 7402(a)"));
        assert!(r.citation.contains("§ 6700"));
        assert!(r.citation.contains("§ 6701"));
        assert!(r.citation.contains("§ 6707"));
        assert!(r.citation.contains("§ 6708"));
        assert!(r.citation.contains("31 U.S.C. § 330"));
        assert!(r.citation.contains("§ 7407"));
        assert!(r.citation.contains("IRM 5.20.7"));
        assert!(r.citation.contains("IRM 5.20.8"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6694")
                && n.contains("§ 6695")
                && n.contains("§ 6700")
                && n.contains("§ 6701")
                && n.contains("§ 6707")
                && n.contains("§ 6708")
                && n.contains("§ 7407")
                && n.contains("EQUITABLE REMEDY")),
            "sibling cluster note must reference full preparer + promoter cluster + § 7407 parallel + equitable-remedy distinction"
        );
    }

    #[test]
    fn supplemental_jurisdiction_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("SUPPLEMENTAL")
                && n.contains("§ 7402(a)")
                && n.contains("criminal prosecution")
                && n.contains("Circular 230")),
            "supplemental-jurisdiction note must reference § 7402(a) + parallel-track availability"
        );
    }

    // ── Missing-element specific notes ─────────────────────

    #[test]
    fn no_secretary_request_specific_note() {
        let r = compute(&input());
        assert!(r.notes.iter().any(|n| n.contains("§ 7408(a)")));
    }

    #[test]
    fn no_conduct_specific_note() {
        let mut b = input();
        b.requested_by_secretary = true;
        let r = compute(&b);
        assert!(r.notes.iter().any(|n| n.contains("§ 7408(c)")));
    }

    #[test]
    fn no_appropriateness_specific_note() {
        let mut b = input();
        b.requested_by_secretary = true;
        b.engaged_in_section_6700_conduct = true;
        let r = compute(&b);
        assert!(r.notes.iter().any(|n| n.contains("§ 7408(b)(2)")));
        assert!(r.notes.iter().any(|n| n.contains("PREVENT RECURRENCE")));
    }
}
