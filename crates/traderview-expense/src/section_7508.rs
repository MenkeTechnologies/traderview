//! IRC § 7508 — Time for performing certain acts postponed
//! by reason of service in combat zone or contingency
//! operation. Trader-relevant for active-duty military
//! traders + spouses serving in designated combat zones,
//! contingency operations, or qualified hazardous duty
//! areas. Distinct from § 7508A (presidentially-declared
//! disaster postponement) — § 7508 covers active-duty
//! military service, while § 7508A covers federally-
//! declared disasters and significant fires.
//!
//! Procedural-companion to § 7508A (disaster postponement),
//! § 7421 (Anti-Injunction Act), § 7422 (refund suit), §
//! 7426 (third-party wrongful levy), § 7433 (civil damages
//! for unauthorized collection), § 7430 (litigation costs),
//! § 6212/§ 6213 (deficiency procedures), and § 6511
//! (refund-claim limitations).
//!
//! **§ 7508(a) basic postponement framework**:
//! IRS DISREGARDS time during:
//! 1. Service in a combat zone designated by Executive
//!    Order, OR
//! 2. Service in a Secretary of Defense designated
//!    contingency operation outside the United States, OR
//! 3. Service in a qualified hazardous duty area, OR
//! 4. Hospitalization inside or outside the United States
//!    as a result of injury received while serving in such
//!    area or operation,
//!
//! PLUS **180 days after the last day in the combat zone /
//! contingency operation / qualified hazardous duty area
//! (or qualified hospitalization)**.
//!
//! **§ 7508(a) hospitalization 5-year cap** — hospitalization
//! INSIDE the United States is treated as a qualified
//! hospitalization only to the extent the period does not
//! exceed 5 years.
//!
//! **§ 7508(b) military spouse extension** — same 180-day
//! benefit applies to spouse of qualifying service member.
//!
//! **§ 7508(c) qualified hazardous duty area** — Sinai
//! Peninsula of Egypt (if any member entitled to special
//! pay) plus other Secretary-of-Defense designated areas.
//!
//! Postponed acts (per § 7508(a)(1)) include filing returns
//! and paying tax and filing amended returns and Tax Court
//! petitions and § 6511 refund claims and § 6212 SNOD
//! responses and § 6213 deficiency challenges.
//!
//! Citations: 26 USC § 7508(a)(1)-(3), (b)(1)-(2), (c)(1)-
//! (2); 26 CFR § 301.7508-1; IRS Notice 2003-21; IRS Form
//! 15109 (Combat Zone Tax Deadline Relief); IRS Pub. 3
//! (Armed Forces' Tax Guide).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    /// § 7508(a)(1)(A) — combat zone designated by Executive
    /// Order.
    CombatZone,
    /// § 7508(a)(1)(B) — Secretary of Defense designated
    /// contingency operation outside the United States.
    ContingencyOperation,
    /// § 7508(c) — qualified hazardous duty area (e.g.,
    /// Sinai Peninsula).
    QualifiedHazardousDutyArea,
    /// Not a qualifying service.
    NotApplicable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerCategory {
    /// Service member directly engaged.
    ServiceMember,
    /// § 7508(b) — military spouse.
    MilitarySpouse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7508Input {
    pub service_type: ServiceType,
    pub taxpayer_category: TaxpayerCategory,
    /// Days of service in qualifying area or operation.
    pub days_of_qualifying_service: u32,
    /// Days of qualified hospitalization (resulting from
    /// injury during qualifying service).
    pub days_of_qualifying_hospitalization: u32,
    /// Whether hospitalization was inside the United States
    /// (for § 7508(a) 5-year = 1825-day cap).
    pub hospitalization_inside_united_states: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7508Result {
    pub postponement_engaged: bool,
    pub total_postponement_days: u32,
    pub hospitalization_capped: bool,
    pub bonus_180_days: u32,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7508Input) -> Section7508Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let qualifying_event = !matches!(input.service_type, ServiceType::NotApplicable);

    if !qualifying_event {
        failure_reasons.push(
            "26 USC § 7508(a) — postponement requires service in (1) combat zone, (2) contingency operation, or (3) qualified hazardous duty area"
                .to_string(),
        );
    }

    let raw_hospitalization = input.days_of_qualifying_hospitalization;
    let hospitalization_cap: u32 = 1825;
    let hospitalization_capped =
        input.hospitalization_inside_united_states && raw_hospitalization > hospitalization_cap;
    let effective_hospitalization = if input.hospitalization_inside_united_states {
        raw_hospitalization.min(hospitalization_cap)
    } else {
        raw_hospitalization
    };

    let bonus_180 = 180_u32;

    let total_postponement = if qualifying_event {
        input
            .days_of_qualifying_service
            .saturating_add(effective_hospitalization)
            .saturating_add(bonus_180)
    } else {
        0
    };

    let notes: Vec<String> = vec![
        "26 USC § 7508(a) — IRS DISREGARDS time during (1) combat zone service (Executive Order designated), (2) Secretary of Defense designated contingency operation, or (3) qualified hazardous duty area, PLUS 180 days after last day in such area / operation / qualified hospitalization"
            .to_string(),
        "26 USC § 7508(a) hospitalization extension — postponement continues through qualified hospitalization (then plus 180 days); hospitalization INSIDE the United States capped at 5 years (1825 days)"
            .to_string(),
        "26 USC § 7508(b) military spouse extension — same 180-day benefit applies to spouse of qualifying service member"
            .to_string(),
        "26 USC § 7508(c) qualified hazardous duty area — includes Sinai Peninsula of Egypt (if member entitled to special pay) plus other Secretary-of-Defense designated areas"
            .to_string(),
        "26 CFR § 301.7508-1 — postponed acts include filing returns + paying tax + filing amended returns + Tax Court petitions + § 6511 refund claims + § 6212 SNOD responses + § 6213 deficiency challenges; IRS Notice 2003-21; IRS Form 15109; IRS Pub. 3 Armed Forces' Tax Guide"
            .to_string(),
    ];

    Section7508Result {
        postponement_engaged: failure_reasons.is_empty() && qualifying_event,
        total_postponement_days: total_postponement,
        hospitalization_capped,
        bonus_180_days: bonus_180,
        failure_reasons,
        citation: "26 USC § 7508(a)(1)-(3), (b)(1)-(2), (c)(1)-(2); 26 CFR § 301.7508-1; IRS Notice 2003-21; IRS Form 15109; IRS Pub. 3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combat_zone_base() -> Section7508Input {
        Section7508Input {
            service_type: ServiceType::CombatZone,
            taxpayer_category: TaxpayerCategory::ServiceMember,
            days_of_qualifying_service: 365,
            days_of_qualifying_hospitalization: 0,
            hospitalization_inside_united_states: false,
        }
    }

    #[test]
    fn combat_zone_365_days_plus_180() {
        let r = check(&combat_zone_base());
        assert!(r.postponement_engaged);
        assert_eq!(r.total_postponement_days, 545);
        assert_eq!(r.bonus_180_days, 180);
        assert!(!r.hospitalization_capped);
    }

    #[test]
    fn contingency_operation_postponement() {
        let mut i = combat_zone_base();
        i.service_type = ServiceType::ContingencyOperation;
        let r = check(&i);
        assert!(r.postponement_engaged);
        assert_eq!(r.total_postponement_days, 545);
    }

    #[test]
    fn qualified_hazardous_duty_area_postponement() {
        let mut i = combat_zone_base();
        i.service_type = ServiceType::QualifiedHazardousDutyArea;
        let r = check(&i);
        assert!(r.postponement_engaged);
    }

    #[test]
    fn not_applicable_no_postponement() {
        let mut i = combat_zone_base();
        i.service_type = ServiceType::NotApplicable;
        let r = check(&i);
        assert!(!r.postponement_engaged);
        assert_eq!(r.total_postponement_days, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7508(a)") && f.contains("combat zone")));
    }

    #[test]
    fn military_spouse_eligible() {
        let mut i = combat_zone_base();
        i.taxpayer_category = TaxpayerCategory::MilitarySpouse;
        let r = check(&i);
        assert!(r.postponement_engaged);
        assert_eq!(r.total_postponement_days, 545);
    }

    #[test]
    fn hospitalization_outside_us_not_capped() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_hospitalization = 3000;
        i.hospitalization_inside_united_states = false;
        let r = check(&i);
        assert!(!r.hospitalization_capped);
        assert_eq!(r.total_postponement_days, 365 + 3000 + 180);
    }

    #[test]
    fn hospitalization_inside_us_capped_at_5_years() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_hospitalization = 3000;
        i.hospitalization_inside_united_states = true;
        let r = check(&i);
        assert!(r.hospitalization_capped);
        assert_eq!(r.total_postponement_days, 365 + 1825 + 180);
    }

    #[test]
    fn hospitalization_inside_us_at_1825_day_boundary_not_capped() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_hospitalization = 1825;
        i.hospitalization_inside_united_states = true;
        let r = check(&i);
        assert!(!r.hospitalization_capped);
        assert_eq!(r.total_postponement_days, 365 + 1825 + 180);
    }

    #[test]
    fn hospitalization_inside_us_at_1826_day_capped() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_hospitalization = 1826;
        i.hospitalization_inside_united_states = true;
        let r = check(&i);
        assert!(r.hospitalization_capped);
        assert_eq!(r.total_postponement_days, 365 + 1825 + 180);
    }

    #[test]
    fn zero_service_days_only_180_day_bonus() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_service = 0;
        let r = check(&i);
        assert!(r.postponement_engaged);
        assert_eq!(r.total_postponement_days, 180);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&combat_zone_base());
        assert!(r.citation.contains("§ 7508(a)(1)-(3)"));
        assert!(r.citation.contains("(b)(1)-(2)"));
        assert!(r.citation.contains("(c)(1)-(2)"));
        assert!(r.citation.contains("§ 301.7508-1"));
        assert!(r.citation.contains("Notice 2003-21"));
        assert!(r.citation.contains("Form 15109"));
        assert!(r.citation.contains("Pub. 3"));
    }

    #[test]
    fn note_pins_three_qualifying_categories() {
        let r = check(&combat_zone_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7508(a)")
            && n.contains("combat zone")
            && n.contains("contingency operation")
            && n.contains("hazardous duty area")
            && n.contains("180 days")));
    }

    #[test]
    fn note_pins_hospitalization_5_year_cap() {
        let r = check(&combat_zone_base());
        assert!(r.notes.iter().any(|n| n.contains("hospitalization extension")
            && n.contains("5 years")
            && n.contains("1825 days")));
    }

    #[test]
    fn note_pins_military_spouse_extension() {
        let r = check(&combat_zone_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7508(b)") && n.contains("military spouse")));
    }

    #[test]
    fn note_pins_sinai_peninsula_qhda() {
        let r = check(&combat_zone_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7508(c)") && n.contains("Sinai Peninsula")));
    }

    #[test]
    fn note_pins_postponed_acts() {
        let r = check(&combat_zone_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 301.7508-1")
            && n.contains("§ 6511")
            && n.contains("§ 6212")
            && n.contains("§ 6213")
            && n.contains("Form 15109")));
    }

    #[test]
    fn service_type_truth_table() {
        for (service, exp_engaged) in [
            (ServiceType::CombatZone, true),
            (ServiceType::ContingencyOperation, true),
            (ServiceType::QualifiedHazardousDutyArea, true),
            (ServiceType::NotApplicable, false),
        ] {
            let mut i = combat_zone_base();
            i.service_type = service;
            let r = check(&i);
            assert_eq!(r.postponement_engaged, exp_engaged);
        }
    }

    #[test]
    fn hospitalization_cap_only_inside_us_invariant() {
        let mut i_outside = combat_zone_base();
        i_outside.days_of_qualifying_hospitalization = 3000;
        i_outside.hospitalization_inside_united_states = false;
        let r_outside = check(&i_outside);
        assert!(!r_outside.hospitalization_capped);

        let mut i_inside = combat_zone_base();
        i_inside.days_of_qualifying_hospitalization = 3000;
        i_inside.hospitalization_inside_united_states = true;
        let r_inside = check(&i_inside);
        assert!(r_inside.hospitalization_capped);
    }

    #[test]
    fn bonus_180_days_always_180() {
        let r = check(&combat_zone_base());
        assert_eq!(r.bonus_180_days, 180);
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_service = u32::MAX;
        i.days_of_qualifying_hospitalization = u32::MAX;
        let r = check(&i);
        assert!(r.postponement_engaged);
    }

    #[test]
    fn military_spouse_truth_table() {
        for category in [TaxpayerCategory::ServiceMember, TaxpayerCategory::MilitarySpouse] {
            let mut i = combat_zone_base();
            i.taxpayer_category = category;
            let r = check(&i);
            assert!(r.postponement_engaged);
        }
    }

    #[test]
    fn hospitalization_zero_no_capping() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_hospitalization = 0;
        i.hospitalization_inside_united_states = true;
        let r = check(&i);
        assert!(!r.hospitalization_capped);
        assert_eq!(r.total_postponement_days, 365 + 0 + 180);
    }

    #[test]
    fn small_hospitalization_inside_us_not_capped() {
        let mut i = combat_zone_base();
        i.days_of_qualifying_hospitalization = 100;
        i.hospitalization_inside_united_states = true;
        let r = check(&i);
        assert!(!r.hospitalization_capped);
        assert_eq!(r.total_postponement_days, 365 + 100 + 180);
    }
}
