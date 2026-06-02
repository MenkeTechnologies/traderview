//! Multi-jurisdictional landlord self-help eviction
//! prohibition framework — among the steepest direct-
//! exposure statutes in residential landlord-tenant law,
//! because every state criminalizes or imposes per-day
//! liquidated-damage liability for utility shutoff, lock
//! changes, or property removal performed WITHOUT a court
//! order and marshal's warrant. Trader-landlord critical
//! because the trader-landlord pattern (out-of-state owner
//! AND non-paying tenant AND lost rental income AND emotional
//! decision to "just turn off the water") matches precisely
//! the fact pattern these statutes were designed to deter.
//!
//! Companion modules: landlord_retaliation_damages /
//! landlord_emergency_entry_notice / landlord_lien_prohibition /
//! tenant_relocation_assistance.
//!
//! **California Civ. Code § 789.3** — landlord SHALL NOT
//! **with intent to terminate occupancy** willfully cause,
//! directly or indirectly, the **interruption or
//! termination** of any utility service (water, heat,
//! light, electricity, gas, telephone, elevator,
//! refrigeration) OR **prevent reasonable access** to the
//! dwelling by changing locks, OR **remove tenant's
//! personal property** from the dwelling. Damages: **$100
//! per day** for each day the violation continues + **$250
//! statutory minimum per violation** + **ACTUAL damages**
//! (hotel costs, restaurant meals, spoiled food, medical
//! expenses) + **reasonable attorney's fees**. Knowing
//! "intent to terminate occupancy" element required — mere
//! utility lapse from non-payment of bill by landlord
//! insufficient if not intended to force tenant out.
//!
//! **New York RPL § 235 + RPAPL § 853 + RPAPL § 768** —
//! self-help eviction is a **CLASS A MISDEMEANOR** (RPAPL
//! § 768; max 1 year imprisonment + fine). Civil **TREBLE
//! DAMAGES** under RPAPL § 853 for any forcible/unlawful
//! entry, ouster, or refusal to restore possession.
//! Landlord may not remove tenant by changing locks,
//! removing doors/windows, OR shutting off essential
//! utilities (heat, hot water, electricity). Court order
//! AND marshal's warrant required for every residential
//! eviction.
//!
//! **Florida Stat. § 83.67** — landlord prohibited from
//! **interrupting/terminating any utility service** (water,
//! heat, light, electricity, gas, elevator, garbage
//! collection, refrigeration) AND from **preventing
//! reasonable access** by changing locks or bootlock device
//! AND from **removing outside doors, locks, roof, walls,
//! windows**. Damages = greater of **ACTUAL +
//! CONSEQUENTIAL damages OR 3 MONTHS' RENT** + **attorney
//! fees** + costs. **Subsequent or repeated non-
//! contemporaneous violations are SEPARATELY recoverable**.
//! Violation = **IRREPARABLE HARM for injunctive relief**.
//!
//! **Texas Tex. Prop. Code § 92.0081 + § 92.008** —
//! landlord may NOT exclude tenant from rental except by
//! (1) court eviction judgment + writ of possession; (2)
//! tenant abandonment per § 92.0081(c); (3) lease bona-
//! fide-repairs clause with 24-hour notice. Damages =
//! **actual damages + $1,000 + 1 MONTH'S RENT (less actual
//! damages) + attorney fees** per violation under
//! § 92.0081(h).
//!
//! **Default — common law wrongful eviction tort** —
//! breach of warranty of quiet enjoyment + tortious
//! interference; recovery includes **actual damages**
//! (relocation costs + property loss + emotional distress)
//! and in some states **PUNITIVE damages** for malicious
//! conduct.
//!
//! Citations: Cal. Civ. Code § 789.3; N.Y. Real Prop. Law
//! § 235; N.Y. RPAPL § 853; N.Y. RPAPL § 768; Fla. Stat.
//! § 83.67; Tex. Prop. Code § 92.0081 and § 92.008.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Florida,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SelfHelpAction {
    /// Utility service interruption (water/heat/light/gas/
    /// electricity/telephone/elevator/refrigeration).
    UtilityShutoff,
    /// Lock change / bootlock / removal of doors or
    /// windows preventing reasonable access.
    LockoutOrAccessDenial,
    /// Removal of tenant's personal property from
    /// dwelling.
    PersonalPropertyRemoval,
    /// Removal of outside doors, locks, roof, walls, or
    /// windows.
    StructuralRemoval,
    /// No prohibited action taken.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelfHelpEvictionInput {
    pub jurisdiction: Jurisdiction,
    pub action: SelfHelpAction,
    /// Whether landlord obtained court eviction judgment +
    /// marshal's writ of possession before acting (gate
    /// that lifts every prohibition).
    pub court_eviction_judgment_obtained: bool,
    /// Whether landlord acted with intent to terminate
    /// occupancy (California Civ. Code § 789.3 element).
    pub acted_with_intent_to_terminate_occupancy: bool,
    /// Days the violation continues (per-day damages
    /// multiplier in CA § 789.3).
    pub days_violation_continues: u32,
    /// Actual damages claimed by tenant in cents (hotel
    /// costs, restaurant meals, spoiled food, medical
    /// expenses, relocation costs).
    pub actual_damages_cents: u64,
    /// Monthly rent in cents (relevant for FL 3-month +
    /// TX 1-month formulas).
    pub monthly_rent_cents: u64,
    /// Whether this is a subsequent non-contemporaneous
    /// violation (FL § 83.67 separate-award trigger).
    pub subsequent_non_contemporaneous_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SelfHelpEvictionResult {
    pub jurisdiction: Jurisdiction,
    pub violation_engaged: bool,
    pub statutory_damages_cents: u64,
    pub actual_damages_cents: u64,
    pub attorney_fees_recoverable: bool,
    pub criminal_misdemeanor_exposure: bool,
    pub injunctive_relief_available: bool,
    pub treble_damages_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &SelfHelpEvictionInput) -> SelfHelpEvictionResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let action_engaged = !matches!(input.action, SelfHelpAction::None);
    let intent_required = input.jurisdiction == Jurisdiction::California;
    let intent_satisfied =
        !intent_required || input.acted_with_intent_to_terminate_occupancy;

    let violation_engaged =
        action_engaged && !input.court_eviction_judgment_obtained && intent_satisfied;

    let mut statutory_damages_cents: u64 = 0;
    let mut criminal_misdemeanor_exposure = false;
    let mut injunctive_relief_available = false;
    let mut treble_damages_engaged = false;
    let attorney_fees_recoverable = violation_engaged;

    if violation_engaged {
        match input.jurisdiction {
            Jurisdiction::California => {
                let per_day = 10_000_u64;
                let day_damages = per_day.saturating_mul(input.days_violation_continues as u64);
                let minimum = 25_000_u64;
                statutory_damages_cents = day_damages.max(minimum);
                failure_reasons.push(
                    "Cal. Civ. Code § 789.3 — landlord prohibited from utility shutoff / lockout / personal property removal WITH INTENT to terminate occupancy; statutory damages: $100/day plus $250 minimum + actual damages + attorney's fees".to_string(),
                );
            }
            Jurisdiction::NewYork => {
                statutory_damages_cents = input.actual_damages_cents.saturating_mul(3);
                criminal_misdemeanor_exposure = true;
                treble_damages_engaged = true;
                failure_reasons.push(
                    "N.Y. Real Prop. Law § 235 + RPAPL § 853 + RPAPL § 768 — self-help eviction is a CLASS A MISDEMEANOR (max 1 year imprisonment + fine); TREBLE DAMAGES under RPAPL § 853 for any forcible/unlawful entry, ouster, or refusal to restore possession + attorney's fees".to_string(),
                );
            }
            Jurisdiction::Florida => {
                let three_months_rent = input.monthly_rent_cents.saturating_mul(3);
                statutory_damages_cents = three_months_rent.max(input.actual_damages_cents);
                injunctive_relief_available = true;
                let separate_award_note = if input.subsequent_non_contemporaneous_violation {
                    " (SUBSEQUENT non-contemporaneous violation triggers SEPARATE statutory damages award)"
                } else {
                    ""
                };
                failure_reasons.push(format!(
                    "Fla. Stat. § 83.67 — landlord prohibited from utility shutoff / lockout / structural removal; statutory damages: greater of ACTUAL + CONSEQUENTIAL OR 3 MONTHS' RENT + attorney's fees + costs; violation constitutes IRREPARABLE HARM for injunctive relief{}",
                    separate_award_note
                ));
            }
            Jurisdiction::Texas => {
                let one_month_rent = input.monthly_rent_cents;
                let net_one_month = one_month_rent.saturating_sub(input.actual_damages_cents);
                statutory_damages_cents = input
                    .actual_damages_cents
                    .saturating_add(100_000)
                    .saturating_add(net_one_month);
                failure_reasons.push(
                    "Tex. Prop. Code § 92.0081(h) — landlord may NOT exclude tenant except by court eviction judgment + writ of possession; statutory damages: ACTUAL damages + $1,000 + 1 MONTH'S RENT (less actual damages) + attorney's fees per violation".to_string(),
                );
            }
            Jurisdiction::Default => {
                statutory_damages_cents = input.actual_damages_cents;
                failure_reasons.push(
                    "Common law wrongful eviction tort — breach of warranty of quiet enjoyment + tortious interference; recovery includes actual damages (relocation costs + property loss + emotional distress); some states permit PUNITIVE damages for malicious conduct".to_string(),
                );
            }
        }
    }

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 789.3 — landlord SHALL NOT willfully cause, directly or indirectly, the interruption or termination of any utility service furnished the tenant (water, heat, light, electricity, gas, telephone, elevator, refrigeration); WITH INTENT TO TERMINATE OCCUPANCY element required".to_string(),
        "Cal. Civ. Code § 789.3 damages — $100 PER DAY + $250 statutory minimum per violation + ACTUAL damages (hotel costs, restaurant meals, spoiled food, medical expenses) + reasonable attorney's fees".to_string(),
        "N.Y. Real Prop. Law § 235 + N.Y. RPAPL § 853 + N.Y. RPAPL § 768 — self-help eviction is a CLASS A MISDEMEANOR (max 1 year imprisonment + fine); civil TREBLE DAMAGES under RPAPL § 853 for any forcible/unlawful entry, ouster, or refusal to restore possession".to_string(),
        "NY remedies include immediate court order of restoration of access + return of belongings + restoration of utilities + treble damages + attorney's fees; court order AND marshal's warrant required for EVERY residential eviction".to_string(),
        "Fla. Stat. § 83.67 — landlord prohibited from interrupting/terminating any utility service (water, heat, light, electricity, gas, elevator, garbage collection, refrigeration) AND from preventing reasonable access by changing locks or bootlock device AND from removing outside doors, locks, roof, walls, windows".to_string(),
        "Fla. Stat. § 83.67 damages — greater of ACTUAL + CONSEQUENTIAL damages OR 3 MONTHS' RENT + attorney's fees + costs; SUBSEQUENT or repeated non-contemporaneous violations are SEPARATELY recoverable; violation constitutes IRREPARABLE HARM for purposes of injunctive relief".to_string(),
        "Tex. Prop. Code § 92.0081 + § 92.008 — landlord may NOT exclude tenant except (1) court eviction judgment + writ of possession; (2) tenant abandonment per § 92.0081(c); (3) lease bona-fide-repairs clause with 24-hour notice".to_string(),
        "Tex. Prop. Code § 92.0081(h) damages — actual damages + $1,000 + 1 MONTH'S RENT (less actual damages) + attorney's fees per violation".to_string(),
        "Default — Common law wrongful eviction tort — breach of warranty of quiet enjoyment + tortious interference; recovery includes actual damages (relocation costs + property loss + emotional distress); some states permit PUNITIVE damages for malicious conduct".to_string(),
        "Across ALL jurisdictions: court eviction judgment + marshal's writ of possession is the ONLY lawful pathway to remove a residential tenant — no exceptions for non-payment of rent, lease violation, or abandonment without proper notice".to_string(),
    ];

    SelfHelpEvictionResult {
        jurisdiction: input.jurisdiction,
        violation_engaged,
        statutory_damages_cents,
        actual_damages_cents: if violation_engaged {
            input.actual_damages_cents
        } else {
            0
        },
        attorney_fees_recoverable,
        criminal_misdemeanor_exposure,
        injunctive_relief_available,
        treble_damages_engaged,
        failure_reasons,
        citation: "Cal. Civ. Code § 789.3; N.Y. Real Prop. Law § 235; N.Y. RPAPL § 853; N.Y. RPAPL § 768; Fla. Stat. § 83.67; Tex. Prop. Code § 92.0081 and § 92.008",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_violation() -> SelfHelpEvictionInput {
        SelfHelpEvictionInput {
            jurisdiction: Jurisdiction::California,
            action: SelfHelpAction::UtilityShutoff,
            court_eviction_judgment_obtained: false,
            acted_with_intent_to_terminate_occupancy: true,
            days_violation_continues: 10,
            actual_damages_cents: 500_000,
            monthly_rent_cents: 200_000,
            subsequent_non_contemporaneous_violation: false,
        }
    }

    #[test]
    fn california_utility_shutoff_with_intent_violation_engaged() {
        let r = check(&ca_violation());
        assert!(r.violation_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 789.3")
            && f.contains("$100/day")
            && f.contains("$250 minimum")));
    }

    #[test]
    fn california_10_day_violation_damages_equal_1000_dollars() {
        let r = check(&ca_violation());
        assert_eq!(r.statutory_damages_cents, 100_000);
    }

    #[test]
    fn california_2_day_violation_floor_locks_at_250_minimum() {
        let mut i = ca_violation();
        i.days_violation_continues = 2;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 25_000);
    }

    #[test]
    fn california_no_intent_no_violation() {
        let mut i = ca_violation();
        i.acted_with_intent_to_terminate_occupancy = false;
        let r = check(&i);
        assert!(!r.violation_engaged);
        assert_eq!(r.statutory_damages_cents, 0);
    }

    #[test]
    fn california_court_order_obtained_no_violation() {
        let mut i = ca_violation();
        i.court_eviction_judgment_obtained = true;
        let r = check(&i);
        assert!(!r.violation_engaged);
    }

    #[test]
    fn california_lockout_action_violation_engaged() {
        let mut i = ca_violation();
        i.action = SelfHelpAction::LockoutOrAccessDenial;
        let r = check(&i);
        assert!(r.violation_engaged);
    }

    #[test]
    fn california_property_removal_action_violation_engaged() {
        let mut i = ca_violation();
        i.action = SelfHelpAction::PersonalPropertyRemoval;
        let r = check(&i);
        assert!(r.violation_engaged);
    }

    #[test]
    fn california_no_action_no_violation() {
        let mut i = ca_violation();
        i.action = SelfHelpAction::None;
        let r = check(&i);
        assert!(!r.violation_engaged);
    }

    #[test]
    fn new_york_treble_damages_engaged_class_a_misdemeanor() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.violation_engaged);
        assert!(r.treble_damages_engaged);
        assert!(r.criminal_misdemeanor_exposure);
        assert_eq!(r.statutory_damages_cents, 1_500_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("RPAPL § 853")
            && f.contains("CLASS A MISDEMEANOR")
            && f.contains("TREBLE DAMAGES")));
    }

    #[test]
    fn new_york_intent_not_required_for_violation() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::NewYork;
        i.acted_with_intent_to_terminate_occupancy = false;
        let r = check(&i);
        assert!(r.violation_engaged);
    }

    #[test]
    fn florida_3_month_rent_floor_overrides_lower_actual_damages() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Florida;
        i.monthly_rent_cents = 200_000;
        i.actual_damages_cents = 100_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 600_000);
        assert!(r.injunctive_relief_available);
    }

    #[test]
    fn florida_higher_actual_damages_overrides_3_month_rent_floor() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Florida;
        i.monthly_rent_cents = 200_000;
        i.actual_damages_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 1_000_000);
    }

    #[test]
    fn florida_subsequent_violation_note_engages() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Florida;
        i.subsequent_non_contemporaneous_violation = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 83.67")
            && f.contains("SUBSEQUENT non-contemporaneous violation")));
    }

    #[test]
    fn florida_irreparable_harm_engages_injunctive_relief() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Florida;
        let r = check(&i);
        assert!(r.injunctive_relief_available);
        assert!(r.failure_reasons.iter().any(|f| f.contains("IRREPARABLE HARM")));
    }

    #[test]
    fn texas_damages_formula_actual_plus_1000_plus_net_one_month() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Texas;
        i.monthly_rent_cents = 200_000;
        i.actual_damages_cents = 50_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 50_000 + 100_000 + 150_000);
    }

    #[test]
    fn texas_actual_damages_exceeding_monthly_rent_clamps_net_to_zero() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Texas;
        i.monthly_rent_cents = 200_000;
        i.actual_damages_cents = 300_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 300_000 + 100_000);
    }

    #[test]
    fn default_jurisdiction_actual_damages_only() {
        let mut i = ca_violation();
        i.jurisdiction = Jurisdiction::Default;
        i.actual_damages_cents = 500_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 500_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("Common law wrongful eviction")
            && f.contains("PUNITIVE damages")));
    }

    #[test]
    fn attorney_fees_recoverable_when_violation_engaged() {
        let r = check(&ca_violation());
        assert!(r.attorney_fees_recoverable);
    }

    #[test]
    fn attorney_fees_not_recoverable_when_no_violation() {
        let mut i = ca_violation();
        i.court_eviction_judgment_obtained = true;
        let r = check(&i);
        assert!(!r.attorney_fees_recoverable);
    }

    #[test]
    fn citation_pins_all_five_jurisdictions() {
        let r = check(&ca_violation());
        assert!(r.citation.contains("Cal. Civ. Code § 789.3"));
        assert!(r.citation.contains("N.Y. Real Prop. Law § 235"));
        assert!(r.citation.contains("N.Y. RPAPL § 853"));
        assert!(r.citation.contains("N.Y. RPAPL § 768"));
        assert!(r.citation.contains("Fla. Stat. § 83.67"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.0081"));
        assert!(r.citation.contains("§ 92.008"));
    }

    #[test]
    fn note_pins_california_per_day_and_minimum() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 789.3")
            && n.contains("$100 PER DAY")
            && n.contains("$250 statutory minimum")));
    }

    #[test]
    fn note_pins_california_intent_element() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 789.3")
            && n.contains("INTENT TO TERMINATE OCCUPANCY")));
    }

    #[test]
    fn note_pins_new_york_class_a_misdemeanor_treble() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("RPAPL § 853")
            && n.contains("CLASS A MISDEMEANOR")
            && n.contains("TREBLE DAMAGES")));
    }

    #[test]
    fn note_pins_new_york_marshal_warrant_requirement() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("marshal's warrant")
            && n.contains("EVERY residential eviction")));
    }

    #[test]
    fn note_pins_florida_3_month_rent_or_actual_greater() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 83.67")
            && n.contains("3 MONTHS' RENT")
            && n.contains("IRREPARABLE HARM")));
    }

    #[test]
    fn note_pins_florida_subsequent_separately_recoverable() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 83.67")
            && n.contains("SUBSEQUENT or repeated non-contemporaneous violations")
            && n.contains("SEPARATELY recoverable")));
    }

    #[test]
    fn note_pins_texas_actual_plus_1000_plus_one_month() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 92.0081(h)")
            && n.contains("$1,000")
            && n.contains("1 MONTH'S RENT")));
    }

    #[test]
    fn note_pins_default_punitive_damages() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("Common law wrongful eviction")
            && n.contains("PUNITIVE damages")));
    }

    #[test]
    fn note_pins_court_order_only_lawful_pathway_invariant() {
        let r = check(&ca_violation());
        assert!(r.notes.iter().any(|n| n.contains("court eviction judgment")
            && n.contains("ONLY lawful pathway")
            && n.contains("no exceptions")));
    }

    #[test]
    fn jurisdiction_action_truth_table() {
        let jurs = [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Florida,
            Jurisdiction::Texas,
            Jurisdiction::Default,
        ];
        let actions = [
            SelfHelpAction::UtilityShutoff,
            SelfHelpAction::LockoutOrAccessDenial,
            SelfHelpAction::PersonalPropertyRemoval,
            SelfHelpAction::StructuralRemoval,
        ];
        for j in jurs {
            for a in actions {
                let mut i = ca_violation();
                i.jurisdiction = j;
                i.action = a;
                let r = check(&i);
                assert!(r.violation_engaged, "jur={:?} action={:?}", j, a);
            }
        }
    }

    #[test]
    fn ny_treble_uniquely_highest_multiplier_invariant() {
        let make = |jur| SelfHelpEvictionInput {
            jurisdiction: jur,
            action: SelfHelpAction::UtilityShutoff,
            court_eviction_judgment_obtained: false,
            acted_with_intent_to_terminate_occupancy: true,
            days_violation_continues: 10,
            actual_damages_cents: 500_000,
            monthly_rent_cents: 200_000,
            subsequent_non_contemporaneous_violation: false,
        };
        let ca = check(&make(Jurisdiction::California));
        let ny = check(&make(Jurisdiction::NewYork));
        let fl = check(&make(Jurisdiction::Florida));
        let tx = check(&make(Jurisdiction::Texas));
        assert!(ny.treble_damages_engaged);
        assert!(!ca.treble_damages_engaged);
        assert!(!fl.treble_damages_engaged);
        assert!(!tx.treble_damages_engaged);
    }

    #[test]
    fn criminal_exposure_only_in_ny_invariant() {
        let make = |jur| SelfHelpEvictionInput {
            jurisdiction: jur,
            action: SelfHelpAction::UtilityShutoff,
            court_eviction_judgment_obtained: false,
            acted_with_intent_to_terminate_occupancy: true,
            days_violation_continues: 10,
            actual_damages_cents: 500_000,
            monthly_rent_cents: 200_000,
            subsequent_non_contemporaneous_violation: false,
        };
        let ca = check(&make(Jurisdiction::California));
        let ny = check(&make(Jurisdiction::NewYork));
        let fl = check(&make(Jurisdiction::Florida));
        let tx = check(&make(Jurisdiction::Texas));
        let de = check(&make(Jurisdiction::Default));
        assert!(ny.criminal_misdemeanor_exposure);
        assert!(!ca.criminal_misdemeanor_exposure);
        assert!(!fl.criminal_misdemeanor_exposure);
        assert!(!tx.criminal_misdemeanor_exposure);
        assert!(!de.criminal_misdemeanor_exposure);
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = ca_violation();
        i.days_violation_continues = u32::MAX;
        let r = check(&i);
        let _ = r.statutory_damages_cents;
        assert!(r.violation_engaged);
    }

    #[test]
    fn ca_court_judgment_no_violation_even_with_intent() {
        let mut i = ca_violation();
        i.court_eviction_judgment_obtained = true;
        let r = check(&i);
        assert!(!r.violation_engaged);
        assert_eq!(r.statutory_damages_cents, 0);
        assert!(!r.attorney_fees_recoverable);
    }
}
