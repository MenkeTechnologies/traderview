//! IRC § 7433 — Civil damages for certain unauthorized
//! collection actions. Trader-relevant when IRS officer or
//! employee recklessly + intentionally + by negligence
//! disregards any IRC provision or regulation in connection
//! with collection of federal tax (wrongful levy beyond
//! statutory limits + lien filed without notice + collection
//! during § 6330 CDP appeal + violations of § 6331/§ 6332/§
//! 6334 limitations). Procedural-companion to § 7421 (Anti-
//! Injunction Act exceptions including § 7426 wrongful
//! levy), § 7430 (litigation costs), § 7521 (interview
//! procedure), § 7525 (FATP privilege), and § 7811 (TAOs).
//!
//! **§ 7433(a) cause of action** — if in connection with
//! any collection of federal tax with respect to a taxpayer,
//! any officer or employee of the IRS recklessly OR
//! intentionally OR by reason of negligence disregards any
//! provision of the Internal Revenue Code or any regulation
//! promulgated under it, such taxpayer may bring civil
//! action for damages against the United States in district
//! court.
//!
//! **§ 7433(b)(1) damages cap** — lesser of:
//! - $1,000,000 (reckless or intentional)
//! - $100,000 (negligence)
//!
//! OR the sum of actual direct economic damages sustained
//! by the plaintiff as proximate result + costs of the
//! action.
//!
//! **§ 7433(d)(1) exhaustion of administrative remedies
//! required** — judgment for damages NOT awarded unless
//! court determines plaintiff has exhausted administrative
//! remedies within the IRS.
//!
//! **§ 7433(d)(2) mitigation reduction** — damages awarded
//! reduced by amount of damages that could have been
//! reasonably mitigated by plaintiff.
//!
//! **§ 7433(d)(3) two-year statute of limitations** —
//! action may be brought only within **2 years** after the
//! date the right of action accrues. Without regard to
//! amount in controversy.
//!
//! **§ 7433A parallel provision** — same regime applies to
//! private tax collection contractors under qualified tax
//! collection contracts (post-2015 ATA reforms).
//!
//! Citations: 26 USC § 7433(a), (b)(1), (d)(1), (d)(2),
//! (d)(3); 26 USC § 7433A (qualified tax collection
//! contractors); 26 CFR § 301.7433-1 (final regulations);
//! IRM 25.3.3 (Suits Against the United States).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Culpability {
    /// § 7433(b)(1)(A) — $100,000 cap.
    Negligence,
    /// § 7433(b)(1)(B) — $1,000,000 cap.
    Reckless,
    /// § 7433(b)(1)(B) — $1,000,000 cap.
    Intentional,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7433Input {
    pub culpability: Culpability,
    /// Whether IRS officer / employee disregarded IRC
    /// provision or regulation in connection with collection.
    pub disregard_of_irc_provision: bool,
    /// Whether the alleged disregard occurred in connection
    /// with collection of federal tax (vs assessment).
    pub in_connection_with_collection: bool,
    /// Whether plaintiff exhausted administrative remedies
    /// within the IRS (§ 7433(d)(1) gate).
    pub administrative_remedies_exhausted: bool,
    /// Whether suit filed within 2-year SOL from accrual.
    pub within_2_year_sol: bool,
    /// Actual direct economic damages sustained by plaintiff
    /// (excluding costs of action).
    pub actual_damages_cents: i64,
    /// Costs of the action (§ 7433(b)(1)).
    pub costs_of_action_cents: i64,
    /// Damages that could have been reasonably mitigated by
    /// plaintiff (§ 7433(d)(2) reduction).
    pub mitigable_damages_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7433Result {
    pub action_maintainable: bool,
    pub damages_cents: i64,
    pub cap_cents: i64,
    pub cap_engaged: bool,
    pub gross_damages_cents: i64,
    pub mitigation_reduction_cents: i64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7433Input) -> Section7433Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if !input.disregard_of_irc_provision {
        failure_reasons.push(
            "26 USC § 7433(a) — no disregard of IRC provision or regulation by IRS officer or employee".to_string(),
        );
    }
    if !input.in_connection_with_collection {
        failure_reasons.push(
            "26 USC § 7433(a) — alleged conduct not in connection with collection of federal tax (assessment / examination / other context not covered)".to_string(),
        );
    }
    if !input.administrative_remedies_exhausted {
        failure_reasons.push(
            "26 USC § 7433(d)(1) — administrative remedies within IRS not exhausted (judgment cannot be awarded)".to_string(),
        );
    }
    if !input.within_2_year_sol {
        failure_reasons.push(
            "26 USC § 7433(d)(3) — action not brought within 2-year statute of limitations from accrual".to_string(),
        );
    }

    let cap: i64 = match input.culpability {
        Culpability::Negligence => 10_000_000_000,
        Culpability::Reckless | Culpability::Intentional => 100_000_000_000,
    };

    let actual = input.actual_damages_cents.max(0);
    let costs = input.costs_of_action_cents.max(0);
    let mitigable = input.mitigable_damages_cents.max(0).min(actual);

    let net_actual_damages = actual.saturating_sub(mitigable);
    let gross = net_actual_damages.saturating_add(costs);
    let damages = gross.min(cap);

    let cap_engaged = gross > cap;
    let action_maintainable = failure_reasons.is_empty();

    let notes: Vec<String> = vec![
        "26 USC § 7433(a) — cause of action requires reckless OR intentional OR negligent disregard of IRC provision or regulation in connection with collection of federal tax"
            .to_string(),
        "26 USC § 7433(b)(1) — damages cap: $1,000,000 (reckless or intentional) / $100,000 (negligence); OR sum of actual direct economic damages + costs of action"
            .to_string(),
        "26 USC § 7433(d)(1) — exhaustion of administrative remedies within IRS required; § 7433(d)(2) — damages reduced by amount reasonably mitigable; § 7433(d)(3) — 2-year SOL from accrual"
            .to_string(),
        "26 USC § 7433A — parallel regime applies to private tax collection contractors under qualified tax collection contracts (post-2015 ATA reforms)"
            .to_string(),
    ];

    Section7433Result {
        action_maintainable,
        damages_cents: if action_maintainable { damages } else { 0 },
        cap_cents: cap,
        cap_engaged: action_maintainable && cap_engaged,
        gross_damages_cents: if action_maintainable { gross } else { 0 },
        mitigation_reduction_cents: if action_maintainable { mitigable } else { 0 },
        failure_reasons,
        citation: "26 USC § 7433(a), (b)(1), (d)(1), (d)(2), (d)(3); 26 USC § 7433A; 26 CFR § 301.7433-1; IRM 25.3.3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reckless_base() -> Section7433Input {
        Section7433Input {
            culpability: Culpability::Reckless,
            disregard_of_irc_provision: true,
            in_connection_with_collection: true,
            administrative_remedies_exhausted: true,
            within_2_year_sol: true,
            actual_damages_cents: 5_000_000,
            costs_of_action_cents: 500_000,
            mitigable_damages_cents: 0,
        }
    }

    fn negligence_base() -> Section7433Input {
        let mut i = reckless_base();
        i.culpability = Culpability::Negligence;
        i
    }

    fn intentional_base() -> Section7433Input {
        let mut i = reckless_base();
        i.culpability = Culpability::Intentional;
        i
    }

    #[test]
    fn reckless_base_passes_below_cap() {
        let r = check(&reckless_base());
        assert!(r.action_maintainable);
        assert_eq!(r.damages_cents, 5_500_000);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn reckless_cap_1m_engages_high_damages() {
        let mut i = reckless_base();
        i.actual_damages_cents = 200_000_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert!(r.cap_engaged);
        assert_eq!(r.damages_cents, 100_000_000_000);
        assert_eq!(r.cap_cents, 100_000_000_000);
    }

    #[test]
    fn negligence_cap_100k_engages_high_damages() {
        let mut i = negligence_base();
        i.actual_damages_cents = 50_000_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert!(r.cap_engaged);
        assert_eq!(r.damages_cents, 10_000_000_000);
        assert_eq!(r.cap_cents, 10_000_000_000);
    }

    #[test]
    fn intentional_cap_1m_same_as_reckless() {
        let mut i = intentional_base();
        i.actual_damages_cents = 200_000_000_000;
        let r = check(&i);
        assert_eq!(r.cap_cents, 100_000_000_000);
        assert_eq!(r.damages_cents, 100_000_000_000);
    }

    #[test]
    fn no_disregard_no_action() {
        let mut i = reckless_base();
        i.disregard_of_irc_provision = false;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert_eq!(r.damages_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7433(a)") && f.contains("no disregard")));
    }

    #[test]
    fn assessment_context_not_collection_no_action() {
        let mut i = reckless_base();
        i.in_connection_with_collection = false;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7433(a)") && f.contains("not in connection with collection")));
    }

    #[test]
    fn exhaustion_required_blocks_judgment() {
        let mut i = reckless_base();
        i.administrative_remedies_exhausted = false;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert_eq!(r.damages_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7433(d)(1)") && f.contains("administrative remedies")));
    }

    #[test]
    fn outside_2_year_sol_blocks_judgment() {
        let mut i = reckless_base();
        i.within_2_year_sol = false;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7433(d)(3)") && f.contains("2-year")));
    }

    #[test]
    fn mitigation_reduction_applies() {
        let mut i = reckless_base();
        i.actual_damages_cents = 8_000_000;
        i.mitigable_damages_cents = 3_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert_eq!(r.mitigation_reduction_cents, 3_000_000);
        assert_eq!(r.gross_damages_cents, 5_500_000);
        assert_eq!(r.damages_cents, 5_500_000);
    }

    #[test]
    fn mitigation_capped_at_actual_damages() {
        let mut i = reckless_base();
        i.actual_damages_cents = 1_000_000;
        i.mitigable_damages_cents = 10_000_000;
        let r = check(&i);
        assert_eq!(r.mitigation_reduction_cents, 1_000_000);
        assert_eq!(r.gross_damages_cents, 500_000);
    }

    #[test]
    fn costs_of_action_added_to_damages() {
        let mut i = reckless_base();
        i.actual_damages_cents = 2_000_000;
        i.costs_of_action_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(r.damages_cents, 3_000_000);
    }

    #[test]
    fn negligence_cap_uniquely_lower_than_reckless_intentional() {
        let mut i_negligence = negligence_base();
        i_negligence.actual_damages_cents = 100_000_000_000;
        let r_negligence = check(&i_negligence);

        let mut i_reckless = reckless_base();
        i_reckless.actual_damages_cents = 100_000_000_000;
        let r_reckless = check(&i_reckless);

        assert!(r_negligence.cap_cents < r_reckless.cap_cents);
        assert_eq!(r_negligence.cap_cents, 10_000_000_000);
        assert_eq!(r_reckless.cap_cents, 100_000_000_000);
        assert_eq!(r_reckless.cap_cents, r_negligence.cap_cents * 10);
    }

    #[test]
    fn culpability_truth_table() {
        for (culp, exp_cap) in [
            (Culpability::Negligence, 10_000_000_000_i64),
            (Culpability::Reckless, 100_000_000_000_i64),
            (Culpability::Intentional, 100_000_000_000_i64),
        ] {
            let mut i = reckless_base();
            i.culpability = culp;
            i.actual_damages_cents = 200_000_000_000;
            let r = check(&i);
            assert_eq!(r.cap_cents, exp_cap);
        }
    }

    #[test]
    fn all_4_gates_failed_stacks_4_reasons() {
        let i = Section7433Input {
            culpability: Culpability::Reckless,
            disregard_of_irc_provision: false,
            in_connection_with_collection: false,
            administrative_remedies_exhausted: false,
            within_2_year_sol: false,
            actual_damages_cents: 5_000_000,
            costs_of_action_cents: 500_000,
            mitigable_damages_cents: 0,
        };
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert_eq!(r.failure_reasons.len(), 4);
    }

    #[test]
    fn cap_at_negligence_boundary_engages() {
        let mut i = negligence_base();
        i.actual_damages_cents = 9_500_000_000;
        i.costs_of_action_cents = 500_000_000;
        let r = check(&i);
        assert_eq!(r.gross_damages_cents, 10_000_000_000);
        assert!(!r.cap_engaged);
        assert_eq!(r.damages_cents, 10_000_000_000);
    }

    #[test]
    fn cap_1_cent_over_negligence_boundary_engages() {
        let mut i = negligence_base();
        i.actual_damages_cents = 10_000_000_001;
        i.costs_of_action_cents = 0;
        let r = check(&i);
        assert!(r.cap_engaged);
        assert_eq!(r.damages_cents, 10_000_000_000);
    }

    #[test]
    fn defensive_negative_damages_clamped() {
        let mut i = reckless_base();
        i.actual_damages_cents = -5_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert_eq!(r.gross_damages_cents, 500_000);
    }

    #[test]
    fn defensive_negative_costs_clamped() {
        let mut i = reckless_base();
        i.actual_damages_cents = 5_000_000;
        i.costs_of_action_cents = -1_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert_eq!(r.gross_damages_cents, 5_000_000);
    }

    #[test]
    fn defensive_negative_mitigable_treated_as_zero() {
        let mut i = reckless_base();
        i.mitigable_damages_cents = -2_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert_eq!(r.mitigation_reduction_cents, 0);
    }

    #[test]
    fn defensive_overflow_gross_saturating() {
        let mut i = reckless_base();
        i.actual_damages_cents = i64::MAX - 1_000;
        i.costs_of_action_cents = i64::MAX - 1_000;
        let r = check(&i);
        assert!(r.cap_engaged);
        assert_eq!(r.damages_cents, 100_000_000_000);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&reckless_base());
        assert!(r.citation.contains("§ 7433(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(d)(1)"));
        assert!(r.citation.contains("(d)(2)"));
        assert!(r.citation.contains("(d)(3)"));
        assert!(r.citation.contains("§ 7433A"));
        assert!(r.citation.contains("§ 301.7433-1"));
        assert!(r.citation.contains("IRM 25.3.3"));
    }

    #[test]
    fn note_pins_culpability_tiers() {
        let r = check(&reckless_base());
        assert!(r.notes.iter().any(|n| n.contains("reckless")
            && n.contains("intentional")
            && n.contains("negligent")));
    }

    #[test]
    fn note_pins_cap_amounts() {
        let r = check(&reckless_base());
        assert!(r.notes.iter().any(|n| n.contains("$1,000,000")
            && n.contains("$100,000")
            && n.contains("negligence")));
    }

    #[test]
    fn note_pins_exhaustion_mitigation_sol() {
        let r = check(&reckless_base());
        assert!(r.notes.iter().any(|n| n.contains("(d)(1)")
            && n.contains("(d)(2)")
            && n.contains("(d)(3)")
            && n.contains("2-year SOL")));
    }

    #[test]
    fn note_pins_7433a_parallel_regime() {
        let r = check(&reckless_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7433A") && n.contains("qualified tax collection contracts")));
    }

    #[test]
    fn exhaustion_failure_zeros_damages() {
        let mut i = reckless_base();
        i.administrative_remedies_exhausted = false;
        let r = check(&i);
        assert_eq!(r.damages_cents, 0);
        assert_eq!(r.gross_damages_cents, 0);
    }

    #[test]
    fn mitigation_full_offset_drives_to_costs_only() {
        let mut i = reckless_base();
        i.actual_damages_cents = 5_000_000;
        i.mitigable_damages_cents = 5_000_000;
        i.costs_of_action_cents = 1_000_000;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert_eq!(r.damages_cents, 1_000_000);
    }

    #[test]
    fn negligence_uniquely_smaller_cap_invariant() {
        let mut i_negligence = negligence_base();
        i_negligence.actual_damages_cents = 50_000_000_000;
        let r_negligence = check(&i_negligence);
        assert!(r_negligence.cap_engaged);

        let mut i_reckless = reckless_base();
        i_reckless.actual_damages_cents = 50_000_000_000;
        let r_reckless = check(&i_reckless);
        assert!(!r_reckless.cap_engaged);
    }

    #[test]
    fn reckless_and_intentional_same_cap_invariant() {
        let mut i_reckless = reckless_base();
        i_reckless.actual_damages_cents = 200_000_000_000;
        let r_reckless = check(&i_reckless);

        let mut i_intentional = intentional_base();
        i_intentional.actual_damages_cents = 200_000_000_000;
        let r_intentional = check(&i_intentional);

        assert_eq!(r_reckless.cap_cents, r_intentional.cap_cents);
        assert_eq!(r_reckless.damages_cents, r_intentional.damages_cents);
    }

    #[test]
    fn precise_actual_5500_plus_costs_500_eq_6000() {
        let mut i = reckless_base();
        i.actual_damages_cents = 5_500_000;
        i.costs_of_action_cents = 500_000;
        i.mitigable_damages_cents = 0;
        let r = check(&i);
        assert_eq!(r.damages_cents, 6_000_000);
    }

    #[test]
    fn zero_damages_zero_costs_yields_zero() {
        let mut i = reckless_base();
        i.actual_damages_cents = 0;
        i.costs_of_action_cents = 0;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert_eq!(r.damages_cents, 0);
    }
}
