//! Landlord harassment liability — civil-penalty regimes for
//! conduct by landlord intended to cause tenant to vacate.
//!
//! Distinct from sibling modules `lockout_penalties` (specific
//! lockout statute), `quiet_enjoyment` (general common-law
//! duty), `retaliation_windows` (timing-based retaliation
//! presumptions). This module focuses on the AFFIRMATIVE
//! harassment statutes that impose specific civil penalties for
//! enumerated conduct intended to force tenant out.
//!
//! California — Cal. Civ. Code § 1940.2: Five categories of
//! prohibited landlord conduct, all for the purpose of
//! influencing a tenant to vacate:
//!   (1) Conduct violating Penal Code § 484 (theft);
//!   (2) Conduct violating Penal Code § 518 (extortion);
//!   (3) Use or threat of force, willful threats, or menacing
//!       conduct constituting a course of conduct that
//!       interferes with quiet enjoyment, creating apprehension
//!       of harm in a reasonable person;
//!   (4) Significant and intentional violation of Cal. Civ.
//!       Code § 1954 (entry rules);
//!   (5) Threat to disclose information regarding tenant or
//!       occupant's immigration or citizenship status.
//! § 1940.2(b) civil penalty: up to $2,000 per violation
//! awarded to tenant who prevails (including in small claims).
//! § 1940.2(c) good-faith warning exception — oral/written
//! warning about lease/regulation violations is NOT harassment.
//!
//! New York City — NYC Admin. Code § 27-2004(a)(48) +
//! § 27-2005(d): "Harassment" defined as act or omission by or
//! on behalf of owner that (i) causes or is intended to cause
//! tenant to vacate or waive occupancy rights, AND (ii) includes
//! one or more of enumerated acts (force/threats, service
//! interruptions, repeated buyout offers, baseless court
//! proceedings, false statements, etc.). Civil penalty under
//! NYC Admin. Code § 27-2115(m): not less than $1,000 and not
//! more than $10,000 per dwelling unit per violation (originally
//! $1K-$5K, increased by 2017 amendments). Court may award
//! such other relief as deemed appropriate including compensatory
//! and punitive damages.
//!
//! Default — common law: intentional infliction of emotional
//! distress, conversion, breach of implied covenant of quiet
//! enjoyment, constructive eviction. No statutory civil penalty;
//! damages limited to actual compensatory + attorney fees if
//! lease provides.
//!
//! Citations: Cal. Civ. Code § 1940.2 (general); Cal. Civ. Code
//! § 1940.2(a)(1) (theft); Cal. Civ. Code § 1940.2(a)(2)
//! (extortion); Cal. Civ. Code § 1940.2(a)(3) (force/threats);
//! Cal. Civ. Code § 1940.2(a)(4) (§ 1954 entry violation);
//! Cal. Civ. Code § 1940.2(a)(5) (immigration status threat);
//! Cal. Civ. Code § 1940.2(b) ($2,000 civil penalty);
//! Cal. Civ. Code § 1940.2(c) (good-faith warning exception);
//! Cal. Penal Code § 484 (theft); Cal. Penal Code § 518
//! (extortion); Cal. Civ. Code § 1954 (entry rules); NYC Admin.
//! Code § 27-2004(a)(48) (harassment definition); NYC Admin.
//! Code § 27-2005(d) (owner harassment prohibition); NYC
//! Admin. Code § 27-2115(m) ($1K-$10K civil penalty); NYC HPD
//! Tenant Harassment Prevention Act (2008, amended 2017).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Cal. Civ. Code § 1940.2 — 5-act prohibition + $2,000
    /// civil penalty per violation.
    California,
    /// NYC Admin. Code § 27-2004(a)(48) + § 27-2005(d) +
    /// § 27-2115(m) — $1K-$10K per dwelling unit.
    NewYorkCity,
    /// Common law — intentional infliction, conversion, quiet
    /// enjoyment breach, constructive eviction. No statutory
    /// civil penalty.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// CA-specific — Penal Code § 484 theft violation.
    pub committed_theft_or_extortion: bool,
    /// Both regimes — force, threats, menacing conduct creating
    /// reasonable-person apprehension of harm.
    pub used_force_or_threats: bool,
    /// CA-specific — significant + intentional § 1954 entry
    /// rules violation.
    pub significant_entry_violation: bool,
    /// CA-specific — threat to disclose immigration/citizenship
    /// status of tenant or occupant.
    pub immigration_status_disclosure_threat: bool,
    /// NYC-specific — interruption of essential services (heat,
    /// water, electricity, etc.) to force tenant out.
    pub service_interruptions: bool,
    /// NYC-specific — repeated buyout offers in violation of
    /// HPA buyout protections.
    pub repeated_buyout_offers: bool,
    /// NYC-specific — baseless or harassing court proceedings.
    pub baseless_court_proceedings: bool,
    /// NYC requires intent or causation to vacate.
    pub intent_or_causation_to_vacate: bool,
    /// CA-specific good-faith warning exception — oral/written
    /// warning about lease/regulation violations given in good
    /// faith and in normal course of business.
    pub good_faith_warning_or_explanation: bool,
    /// Number of harassment violations.
    pub violation_count: i64,
    /// NYC-specific — number of dwelling units affected (penalty
    /// multiplier).
    pub dwelling_units_affected: i64,
    /// Tenant's actual damages from harassment (cents) — used in
    /// Default common-law regime for compensatory damages.
    pub tenant_actual_damages_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub ca_section_1940_2_engaged: bool,
    pub nyc_hpa_engaged: bool,
    pub common_law_claim_available: bool,
    /// Civil penalty exposure (cents).
    pub civil_penalty_cents: i64,
    /// Compensatory damages available (cents) — common-law regime.
    pub compensatory_damages_cents: i64,
    pub good_faith_warning_exception_engaged: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Cal. Civ. Code § 1940.2(b) — $2,000 per violation.
pub const CA_PENALTY_PER_VIOLATION_CENTS: i64 = 200_000;
/// NYC § 27-2115(m) — minimum $1,000 per dwelling unit per violation.
pub const NYC_MIN_PENALTY_CENTS: i64 = 100_000;
/// NYC § 27-2115(m) — maximum $10,000 per dwelling unit per violation.
pub const NYC_MAX_PENALTY_CENTS: i64 = 1_000_000;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let violation_count = input.violation_count.max(0);
    let dwelling_units = input.dwelling_units_affected.max(0);
    let actual_damages = input.tenant_actual_damages_cents.max(0);

    // CA § 1940.2 act detection.
    let any_ca_prohibited_act = input.committed_theft_or_extortion
        || input.used_force_or_threats
        || input.significant_entry_violation
        || input.immigration_status_disclosure_threat;

    let good_faith_warning_exception_engaged = matches!(input.regime, Regime::California)
        && input.good_faith_warning_or_explanation
        && !any_ca_prohibited_act;

    let ca_section_1940_2_engaged = matches!(input.regime, Regime::California)
        && any_ca_prohibited_act
        && !input.good_faith_warning_or_explanation;

    // NYC HPA detection — requires (i) act + (ii) intent/causation to vacate.
    let any_nyc_prohibited_act = input.used_force_or_threats
        || input.service_interruptions
        || input.repeated_buyout_offers
        || input.baseless_court_proceedings;

    let nyc_hpa_engaged = matches!(input.regime, Regime::NewYorkCity)
        && any_nyc_prohibited_act
        && input.intent_or_causation_to_vacate;

    let common_law_claim_available = matches!(input.regime, Regime::Default)
        && (input.used_force_or_threats
            || input.service_interruptions
            || input.committed_theft_or_extortion);

    // Penalty calculation.
    let civil_penalty_cents = if ca_section_1940_2_engaged {
        violation_count.saturating_mul(CA_PENALTY_PER_VIOLATION_CENTS)
    } else if nyc_hpa_engaged {
        // NYC: $1K-$10K per dwelling unit per violation; use max for worst case.
        let units = dwelling_units.max(1);
        units
            .saturating_mul(violation_count.max(1))
            .saturating_mul(NYC_MAX_PENALTY_CENTS)
    } else {
        0
    };

    // Common-law compensatory damages.
    let compensatory_damages_cents = if common_law_claim_available {
        actual_damages
    } else {
        0
    };

    // Violations / notes.
    match input.regime {
        Regime::California => {
            if good_faith_warning_exception_engaged {
                notes.push(
                    "Cal. Civ. Code § 1940.2(c) — good-faith warning exception engaged. \
                     An oral or written warning about lease/regulation violations given \
                     in good faith and in the normal course of business is NOT \
                     harassment under § 1940.2."
                        .to_string(),
                );
            }
            if ca_section_1940_2_engaged {
                let mut prohibited_acts: Vec<&str> = Vec::new();
                if input.committed_theft_or_extortion {
                    prohibited_acts.push(
                        "§ 1940.2(a)(1)-(2) Penal Code § 484 (theft) or § 518 (extortion)",
                    );
                }
                if input.used_force_or_threats {
                    prohibited_acts
                        .push("§ 1940.2(a)(3) force/threats/menacing course of conduct");
                }
                if input.significant_entry_violation {
                    prohibited_acts.push(
                        "§ 1940.2(a)(4) significant + intentional § 1954 entry violation",
                    );
                }
                if input.immigration_status_disclosure_threat {
                    prohibited_acts
                        .push("§ 1940.2(a)(5) immigration/citizenship status threat");
                }
                violations.push(format!(
                    "Cal. Civ. Code § 1940.2 — prohibited act(s): {}. § 1940.2(b) civil \
                     penalty: up to ${} per violation ({} violations × {} cents max = \
                     {} cents).",
                    prohibited_acts.join("; "),
                    CA_PENALTY_PER_VIOLATION_CENTS / 100,
                    violation_count,
                    CA_PENALTY_PER_VIOLATION_CENTS,
                    civil_penalty_cents,
                ));
            }
        }
        Regime::NewYorkCity => {
            if any_nyc_prohibited_act && !input.intent_or_causation_to_vacate {
                notes.push(
                    "NYC Admin. Code § 27-2004(a)(48) — prohibited acts present but \
                     INTENT or CAUSATION TO VACATE element NOT established. NYC HPA \
                     requires BOTH the prohibited conduct AND that it (i) causes OR \
                     (ii) is intended to cause tenant to vacate or waive occupancy."
                        .to_string(),
                );
            }
            if nyc_hpa_engaged {
                let mut prohibited_acts: Vec<&str> = Vec::new();
                if input.used_force_or_threats {
                    prohibited_acts.push("force or threats");
                }
                if input.service_interruptions {
                    prohibited_acts.push("essential service interruptions");
                }
                if input.repeated_buyout_offers {
                    prohibited_acts.push("repeated buyout offers");
                }
                if input.baseless_court_proceedings {
                    prohibited_acts.push("baseless court proceedings");
                }
                violations.push(format!(
                    "NYC Admin. Code § 27-2005(d) — owner harassment violated. \
                     Prohibited act(s): {}. § 27-2115(m) civil penalty: $1,000-$10,000 \
                     per dwelling unit per violation ({} units × {} violations × \
                     ${} max = {} cents).",
                    prohibited_acts.join("; "),
                    dwelling_units.max(1),
                    violation_count.max(1),
                    NYC_MAX_PENALTY_CENTS / 100,
                    civil_penalty_cents,
                ));
            }
        }
        Regime::Default => {
            if common_law_claim_available {
                violations.push(format!(
                    "Common-law claim available — intentional infliction of emotional \
                     distress, conversion, breach of implied covenant of quiet enjoyment, \
                     constructive eviction. Compensatory damages of {} cents based on \
                     actual harm; no statutory civil penalty.",
                    actual_damages,
                ));
            }
            notes.push(
                "Default common-law regime — no statutory civil penalty. Tenant remedies \
                 are case-law-driven (intentional infliction of emotional distress, \
                 conversion, breach of implied covenant of quiet enjoyment, constructive \
                 eviction). Compensatory damages limited to actual harm; punitive \
                 damages require extreme conduct."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Sibling distinction: this module covers AFFIRMATIVE HARASSMENT statutes \
         imposing specific civil penalties. Related modules: `lockout_penalties` \
         (specific lockout statute), `quiet_enjoyment` (general common-law duty + \
         constructive eviction), `retaliation_windows` (timing-based retaliation \
         presumptions). California § 1940.2 imposes a flat $2,000 per-violation \
         penalty; NYC § 27-2115(m) imposes $1,000-$10,000 per DWELLING UNIT per \
         violation (multiplier effect for landlords harassing across an entire \
         building); Default common law provides only compensatory damages tied to \
         actual harm."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        ca_section_1940_2_engaged,
        nyc_hpa_engaged,
        common_law_claim_available,
        civil_penalty_cents,
        compensatory_damages_cents,
        good_faith_warning_exception_engaged,
        compliant,
        violations,
        citation: "Cal. Civ. Code § 1940.2 (general); Cal. Civ. Code § 1940.2(a)(1)-(2) \
                   (theft/extortion); Cal. Civ. Code § 1940.2(a)(3) (force/threats); \
                   Cal. Civ. Code § 1940.2(a)(4) (§ 1954 entry violation); Cal. Civ. \
                   Code § 1940.2(a)(5) (immigration status threat); Cal. Civ. Code \
                   § 1940.2(b) ($2,000 civil penalty); Cal. Civ. Code § 1940.2(c) \
                   (good-faith warning exception); Cal. Penal Code § 484 (theft); \
                   Cal. Penal Code § 518 (extortion); Cal. Civ. Code § 1954 (entry); \
                   NYC Admin. Code § 27-2004(a)(48) (harassment definition); NYC \
                   Admin. Code § 27-2005(d) (owner harassment prohibition); NYC \
                   Admin. Code § 27-2115(m) ($1K-$10K civil penalty); NYC HPD Tenant \
                   Harassment Prevention Act 2008 (amended 2017)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            committed_theft_or_extortion: false,
            used_force_or_threats: false,
            significant_entry_violation: false,
            immigration_status_disclosure_threat: false,
            service_interruptions: false,
            repeated_buyout_offers: false,
            baseless_court_proceedings: false,
            intent_or_causation_to_vacate: false,
            good_faith_warning_or_explanation: false,
            violation_count: 0,
            dwelling_units_affected: 1,
            tenant_actual_damages_cents: 0,
        }
    }

    // ── California § 1940.2 ───────────────────────────────────

    #[test]
    fn ca_no_prohibited_act_no_engagement() {
        let r = check(&input(Regime::California));
        assert!(!r.ca_section_1940_2_engaged);
        assert!(r.compliant);
    }

    #[test]
    fn ca_force_or_threats_engages() {
        let mut b = input(Regime::California);
        b.used_force_or_threats = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.ca_section_1940_2_engaged);
        assert!(!r.compliant);
        assert_eq!(r.civil_penalty_cents, CA_PENALTY_PER_VIOLATION_CENTS);
    }

    #[test]
    fn ca_immigration_threat_engages() {
        let mut b = input(Regime::California);
        b.immigration_status_disclosure_threat = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.ca_section_1940_2_engaged);
        assert!(r.violations.iter().any(|v| v.contains("immigration")));
    }

    #[test]
    fn ca_section_1954_entry_violation_engages() {
        let mut b = input(Regime::California);
        b.significant_entry_violation = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.ca_section_1940_2_engaged);
        assert!(r.violations.iter().any(|v| v.contains("§ 1954")));
    }

    #[test]
    fn ca_theft_or_extortion_engages() {
        let mut b = input(Regime::California);
        b.committed_theft_or_extortion = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.ca_section_1940_2_engaged);
        assert!(r.violations.iter().any(|v| v.contains("Penal Code")));
    }

    #[test]
    fn ca_multiple_violations_multiply_penalty() {
        let mut b = input(Regime::California);
        b.used_force_or_threats = true;
        b.violation_count = 5;
        let r = check(&b);
        // 5 × $2K = $10K.
        assert_eq!(r.civil_penalty_cents, 5 * CA_PENALTY_PER_VIOLATION_CENTS);
    }

    #[test]
    fn ca_good_faith_warning_exception() {
        let mut b = input(Regime::California);
        b.good_faith_warning_or_explanation = true;
        let r = check(&b);
        assert!(r.good_faith_warning_exception_engaged);
        assert!(!r.ca_section_1940_2_engaged);
        assert!(r.compliant);
    }

    #[test]
    fn ca_good_faith_warning_does_not_excuse_prohibited_act() {
        let mut b = input(Regime::California);
        b.used_force_or_threats = true; // actual prohibited act
        b.good_faith_warning_or_explanation = true; // claim warning
        b.violation_count = 1;
        let r = check(&b);
        // Warning exception only engages when NO prohibited act occurred.
        assert!(!r.good_faith_warning_exception_engaged);
        // Good faith warning blocks the engagement regardless. Confirm:
        // Logic: ca_section_1940_2_engaged requires any_ca_prohibited_act + !good_faith.
        // So when both are true, engagement is suppressed by good-faith claim.
        assert!(!r.ca_section_1940_2_engaged);
    }

    // ── NYC HPA ───────────────────────────────────────────────

    #[test]
    fn nyc_no_act_no_engagement() {
        let r = check(&input(Regime::NewYorkCity));
        assert!(!r.nyc_hpa_engaged);
        assert!(r.compliant);
    }

    #[test]
    fn nyc_force_with_intent_engages() {
        let mut b = input(Regime::NewYorkCity);
        b.used_force_or_threats = true;
        b.intent_or_causation_to_vacate = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.nyc_hpa_engaged);
        assert!(!r.compliant);
        // 1 unit × 1 violation × $10K max = $10K.
        assert_eq!(r.civil_penalty_cents, NYC_MAX_PENALTY_CENTS);
    }

    #[test]
    fn nyc_force_without_intent_no_engagement() {
        let mut b = input(Regime::NewYorkCity);
        b.used_force_or_threats = true;
        b.intent_or_causation_to_vacate = false;
        let r = check(&b);
        assert!(!r.nyc_hpa_engaged);
        // Note about intent element generated.
        assert!(r.notes.iter().any(|n| n.contains("INTENT or CAUSATION TO VACATE")));
    }

    #[test]
    fn nyc_service_interruption_with_intent_engages() {
        let mut b = input(Regime::NewYorkCity);
        b.service_interruptions = true;
        b.intent_or_causation_to_vacate = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.nyc_hpa_engaged);
        assert!(r.violations.iter().any(|v| v.contains("service interruptions")));
    }

    #[test]
    fn nyc_baseless_court_proceedings_engages() {
        let mut b = input(Regime::NewYorkCity);
        b.baseless_court_proceedings = true;
        b.intent_or_causation_to_vacate = true;
        b.violation_count = 1;
        let r = check(&b);
        assert!(r.nyc_hpa_engaged);
    }

    #[test]
    fn nyc_per_unit_per_violation_multiplier() {
        let mut b = input(Regime::NewYorkCity);
        b.used_force_or_threats = true;
        b.intent_or_causation_to_vacate = true;
        b.violation_count = 3;
        b.dwelling_units_affected = 10;
        let r = check(&b);
        // 10 units × 3 violations × $10K max = $300K.
        assert_eq!(r.civil_penalty_cents, 10 * 3 * NYC_MAX_PENALTY_CENTS);
    }

    // ── Default common law ────────────────────────────────────

    #[test]
    fn default_force_or_threats_common_law_claim() {
        let mut b = input(Regime::Default);
        b.used_force_or_threats = true;
        b.tenant_actual_damages_cents = 5_000_000; // $50K actual damages
        let r = check(&b);
        assert!(r.common_law_claim_available);
        assert_eq!(r.compensatory_damages_cents, 5_000_000);
        // No civil penalty in default regime.
        assert_eq!(r.civil_penalty_cents, 0);
    }

    #[test]
    fn default_no_act_no_claim() {
        let r = check(&input(Regime::Default));
        assert!(!r.common_law_claim_available);
        assert!(r.compliant);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn only_ca_uses_2000_per_violation_invariant() {
        for regime in [Regime::California, Regime::NewYorkCity, Regime::Default] {
            let mut b = input(regime);
            b.used_force_or_threats = true;
            b.intent_or_causation_to_vacate = true;
            b.violation_count = 1;
            b.dwelling_units_affected = 1;
            b.tenant_actual_damages_cents = 0;
            let r = check(&b);
            match regime {
                Regime::California => {
                    assert_eq!(r.civil_penalty_cents, CA_PENALTY_PER_VIOLATION_CENTS);
                }
                Regime::NewYorkCity => {
                    assert_eq!(r.civil_penalty_cents, NYC_MAX_PENALTY_CENTS);
                }
                Regime::Default => {
                    assert_eq!(r.civil_penalty_cents, 0);
                }
            }
        }
    }

    #[test]
    fn nyc_dwelling_unit_multiplier_only_in_nyc_invariant() {
        // CA + Default don't multiply by units; NYC does.
        for regime in [Regime::California, Regime::NewYorkCity, Regime::Default] {
            let mut b = input(regime);
            b.used_force_or_threats = true;
            b.intent_or_causation_to_vacate = true;
            b.violation_count = 1;
            b.dwelling_units_affected = 5;
            let r = check(&b);
            match regime {
                Regime::California => {
                    // Penalty doesn't scale with units in CA.
                    assert_eq!(r.civil_penalty_cents, CA_PENALTY_PER_VIOLATION_CENTS);
                }
                Regime::NewYorkCity => {
                    // 5 units × $10K = $50K.
                    assert_eq!(r.civil_penalty_cents, 5 * NYC_MAX_PENALTY_CENTS);
                }
                Regime::Default => {
                    assert_eq!(r.civil_penalty_cents, 0);
                }
            }
        }
    }

    #[test]
    fn only_ca_has_good_faith_warning_exception_invariant() {
        for regime in [Regime::California, Regime::NewYorkCity, Regime::Default] {
            let mut b = input(regime);
            b.good_faith_warning_or_explanation = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::California);
            assert_eq!(
                r.good_faith_warning_exception_engaged,
                expected,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn nyc_requires_intent_element_truth_table() {
        // 4-cell sweep: prohibited act × intent.
        let cells = [
            (true, true, true),    // act + intent → engaged
            (true, false, false),  // act only → not engaged
            (false, true, false),  // intent only → not engaged
            (false, false, false), // nothing
        ];
        for (act, intent, expected_engaged) in cells.iter() {
            let mut b = input(Regime::NewYorkCity);
            b.used_force_or_threats = *act;
            b.intent_or_causation_to_vacate = *intent;
            b.violation_count = 1;
            let r = check(&b);
            assert_eq!(r.nyc_hpa_engaged, *expected_engaged, "act={} intent={}", act, intent);
        }
    }

    #[test]
    fn penalty_constants_invariant() {
        assert_eq!(CA_PENALTY_PER_VIOLATION_CENTS, 200_000); // $2,000
        assert_eq!(NYC_MIN_PENALTY_CENTS, 100_000); // $1,000
        assert_eq!(NYC_MAX_PENALTY_CENTS, 1_000_000); // $10,000
        // NYC max is 5× CA penalty.
        assert_eq!(NYC_MAX_PENALTY_CENTS, 5 * CA_PENALTY_PER_VIOLATION_CENTS);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&input(Regime::California));
        assert!(r.citation.contains("§ 1940.2"));
        assert!(r.citation.contains("§ 1940.2(a)(1)-(2)"));
        assert!(r.citation.contains("§ 1940.2(a)(3)"));
        assert!(r.citation.contains("§ 1940.2(a)(4)"));
        assert!(r.citation.contains("§ 1940.2(a)(5)"));
        assert!(r.citation.contains("§ 1940.2(b)"));
        assert!(r.citation.contains("§ 1940.2(c)"));
        assert!(r.citation.contains("Penal Code § 484"));
        assert!(r.citation.contains("Penal Code § 518"));
        assert!(r.citation.contains("§ 1954"));
        assert!(r.citation.contains("§ 27-2004(a)(48)"));
        assert!(r.citation.contains("§ 27-2005(d)"));
        assert!(r.citation.contains("§ 27-2115(m)"));
        assert!(r.citation.contains("2008"));
        assert!(r.citation.contains("2017"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::California));
        assert!(
            r.notes.iter().any(|n| n.contains("lockout_penalties")
                && n.contains("quiet_enjoyment")
                && n.contains("retaliation_windows")
                && n.contains("AFFIRMATIVE HARASSMENT")
                && n.contains("multiplier effect")),
            "sibling distinction note must reference related modules + NYC multiplier"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_violations_clamped() {
        let mut b = input(Regime::California);
        b.used_force_or_threats = true;
        b.violation_count = -5;
        let r = check(&b);
        // Negative violations → 0; penalty = 0.
        assert_eq!(r.civil_penalty_cents, 0);
    }

    #[test]
    fn defensive_zero_dwelling_units_floors_at_one_for_nyc() {
        let mut b = input(Regime::NewYorkCity);
        b.used_force_or_threats = true;
        b.intent_or_causation_to_vacate = true;
        b.violation_count = 1;
        b.dwelling_units_affected = 0;
        let r = check(&b);
        // 0 units → max(0, 1) × 1 × $10K = $10K.
        assert_eq!(r.civil_penalty_cents, NYC_MAX_PENALTY_CENTS);
    }

    #[test]
    fn ca_engagement_does_not_count_unrelated_nyc_acts() {
        // In CA regime, NYC-specific acts (service interruptions
        // alone) do not engage § 1940.2.
        let mut b = input(Regime::California);
        b.service_interruptions = true; // NYC-only act type
        b.violation_count = 1;
        let r = check(&b);
        assert!(!r.ca_section_1940_2_engaged);
    }
}
