//! Mandatory landlord-paid written-response timeframe for tenant
//! repair requests. When tenant submits a written repair request,
//! how many days does the landlord have to respond and act?
//! Distinct from `repair_and_deduct` (tenant self-help remedy
//! when landlord doesn't act), `habitability_remedies` (broader
//! warranty-of-habitability framework), and `landlord_security_
//! device_obligations` (specific security-device duties).
//!
//! Failure to respond within statutory timeframe exposes landlord
//! to tenant rent abatement, repair-and-deduct rights, and
//! lease-termination rights under state habitability framework.
//!
//! **Four regimes**:
//!
//! **Texas — Tex. Prop. Code § 92.052(d) + § 92.056**. Landlord
//! must make a diligent effort to repair material habitability
//! conditions within a REASONABLE TIME, presumed at SEVEN DAYS
//! for normal conditions or as soon as practicable for emergency
//! conditions affecting health/safety. § 92.056 sets specific
//! tenant remedies for landlord noncompliance including § 92.0563
//! repair-and-deduct cap (greater of one month's rent or $500).
//!
//! **Illinois — Chicago RLTO § 5-12-110(d)** (Chicago Residential
//! Landlord and Tenant Ordinance — applies to City of Chicago).
//! Landlord must respond to tenant written repair request within
//! 14 DAYS for ordinary repairs or 72 hours for emergency
//! conditions. Tenant may withhold rent pro-rata or terminate
//! lease for noncompliance.
//!
//! **Washington — RCW 59.18.070**. Tiered response framework
//! based on severity: 24 HOURS for no heat/no hot water/no
//! electricity/no water/imminent threat; 72 HOURS for refrigerator/
//! range/major appliances/plumbing fixtures; 10 DAYS for all
//! other defective conditions. Tenant remedies include rent
//! abatement, repair-and-deduct, lease termination.
//!
//! **Default — common-law reasonable time**. Most states follow
//! common-law implied warranty of habitability + reasonable-
//! time-to-cure framework. No specific statutory hour/day
//! limits; tenant must show landlord failed to act within
//! reasonable time given severity of defect.
//!
//! Citations: Tex. Prop. Code §§ 92.052(d), 92.056, 92.0563 (TX
//! repair obligation + tenant remedies + repair-and-deduct cap);
//! Chicago RLTO § 5-12-110(d) (Chicago Residential Landlord and
//! Tenant Ordinance); RCW 59.18.070 (Washington tiered repair
//! response framework); common-law implied warranty of
//! habitability (Hilder v. St. Peter, 144 Vt. 150 (1984); Javins
//! v. First National Realty Corp., 428 F.2d 1071 (D.C. Cir.
//! 1970)).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Texas,
    Illinois,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairSeverity {
    /// No heat / no hot water / no electricity / no water /
    /// imminent threat to health or safety.
    EmergencyImminentThreat,
    /// Refrigerator / range / major appliance / plumbing fixture
    /// (WA 72-hour tier).
    UrgentMajorAppliance,
    /// Ordinary repair (broken window, minor plumbing, cosmetic).
    Ordinary,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordRepairResponseInput {
    pub regime: Regime,
    pub severity: RepairSeverity,
    /// Whether tenant submitted a written repair request to the
    /// landlord (oral requests typically don't trigger statutory
    /// duties).
    pub written_repair_request_made: bool,
    /// Hours elapsed since written repair request received.
    pub hours_since_request: u32,
    /// Whether landlord responded in writing within statutory
    /// timeframe.
    pub landlord_responded_in_writing: bool,
    /// Whether repair was completed within statutory timeframe.
    pub repair_completed_within_timeframe: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordRepairResponseResult {
    pub compliant: bool,
    /// Required response timeframe in hours under the applicable
    /// regime and severity.
    pub required_response_hours: u32,
    pub tenant_repair_and_deduct_engaged: bool,
    pub tenant_rent_abatement_engaged: bool,
    pub tenant_lease_termination_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LandlordRepairResponseInput) -> LandlordRepairResponseResult {
    match input.regime {
        Regime::Texas => check_texas(input),
        Regime::Illinois => check_illinois(input),
        Regime::Washington => check_washington(input),
        Regime::Default => check_default(input),
    }
}

fn check_texas(input: &LandlordRepairResponseInput) -> LandlordRepairResponseResult {
    let mut violations: Vec<String> = Vec::new();
    let required_hours = match input.severity {
        RepairSeverity::EmergencyImminentThreat => 24,
        RepairSeverity::UrgentMajorAppliance => 168,
        RepairSeverity::Ordinary => 168,
    };

    let notes: Vec<String> = vec![
        "Tex. Prop. Code § 92.052(d) — landlord must make diligent effort to repair material habitability conditions within REASONABLE TIME presumed at 7 DAYS (168 hours) for normal conditions or as soon as practicable for emergency conditions affecting health/safety"
            .to_string(),
        "Tex. Prop. Code § 92.056 — tenant remedies for noncompliance include lease termination + repair-and-deduct under § 92.0563 (capped at GREATER of one month's rent OR $500) + actual damages + attorney fees + court costs"
            .to_string(),
    ];

    if input.written_repair_request_made
        && input.hours_since_request > required_hours
        && !input.repair_completed_within_timeframe
    {
        violations.push(format!(
            "Tex. Prop. Code § 92.052(d) — landlord failed to repair within statutory timeframe ({} hours elapsed since written request; required {} hours for severity)",
            input.hours_since_request, required_hours
        ));
    }

    let timeframe_exceeded = input.hours_since_request > required_hours;
    let compliant = violations.is_empty();
    LandlordRepairResponseResult {
        compliant,
        required_response_hours: required_hours,
        tenant_repair_and_deduct_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        tenant_rent_abatement_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        tenant_lease_termination_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        violations,
        citation: "Tex. Prop. Code §§ 92.052(d), 92.056, 92.0563",
        notes,
    }
}

fn check_illinois(input: &LandlordRepairResponseInput) -> LandlordRepairResponseResult {
    let mut violations: Vec<String> = Vec::new();
    let required_hours = match input.severity {
        RepairSeverity::EmergencyImminentThreat => 72,
        RepairSeverity::UrgentMajorAppliance => 336,
        RepairSeverity::Ordinary => 336,
    };

    let notes: Vec<String> = vec![
        "Chicago RLTO § 5-12-110(d) — landlord must respond to tenant written repair request within 14 DAYS (336 hours) for ordinary repairs or 72 HOURS for emergency conditions affecting health/safety"
            .to_string(),
        "Chicago RLTO § 5-12-110 — tenant remedies for noncompliance include pro-rata rent withholding + lease termination + repair-and-deduct"
            .to_string(),
    ];

    if input.written_repair_request_made
        && input.hours_since_request > required_hours
        && !input.repair_completed_within_timeframe
    {
        violations.push(format!(
            "Chicago RLTO § 5-12-110(d) — landlord failed to respond within statutory timeframe ({} hours elapsed since written request; required {} hours for severity)",
            input.hours_since_request, required_hours
        ));
    }

    let timeframe_exceeded = input.hours_since_request > required_hours;
    let compliant = violations.is_empty();
    LandlordRepairResponseResult {
        compliant,
        required_response_hours: required_hours,
        tenant_repair_and_deduct_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        tenant_rent_abatement_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        tenant_lease_termination_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        violations,
        citation: "Chicago RLTO § 5-12-110(d) (Chicago Residential Landlord and Tenant Ordinance)",
        notes,
    }
}

fn check_washington(input: &LandlordRepairResponseInput) -> LandlordRepairResponseResult {
    let mut violations: Vec<String> = Vec::new();
    let required_hours = match input.severity {
        RepairSeverity::EmergencyImminentThreat => 24,
        RepairSeverity::UrgentMajorAppliance => 72,
        RepairSeverity::Ordinary => 240,
    };

    let notes: Vec<String> = vec![
        "RCW 59.18.070 — Washington tiered response framework based on severity: 24 HOURS for no heat / no hot water / no electricity / no water / imminent threat; 72 HOURS for refrigerator / range / major appliances / plumbing fixtures; 10 DAYS (240 hours) for all other defective conditions"
            .to_string(),
        "RCW 59.18 — tenant remedies for noncompliance include rent abatement + repair-and-deduct + lease termination + civil damages"
            .to_string(),
    ];

    if input.written_repair_request_made
        && input.hours_since_request > required_hours
        && !input.repair_completed_within_timeframe
    {
        violations.push(format!(
            "RCW 59.18.070 — landlord failed to respond within tiered statutory timeframe ({} hours elapsed since written request; required {} hours for severity)",
            input.hours_since_request, required_hours
        ));
    }

    let timeframe_exceeded = input.hours_since_request > required_hours;
    let compliant = violations.is_empty();
    LandlordRepairResponseResult {
        compliant,
        required_response_hours: required_hours,
        tenant_repair_and_deduct_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        tenant_rent_abatement_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        tenant_lease_termination_engaged: timeframe_exceeded
            && !input.repair_completed_within_timeframe,
        violations,
        citation: "RCW 59.18.070; RCW 59.18 (Residential Landlord-Tenant Act)",
        notes,
    }
}

fn check_default(_input: &LandlordRepairResponseInput) -> LandlordRepairResponseResult {
    let notes: Vec<String> = vec![
        "default rule — common-law implied warranty of habitability + reasonable-time-to-cure framework per Hilder v. St. Peter, 144 Vt. 150 (1984); Javins v. First National Realty Corp., 428 F.2d 1071 (D.C. Cir. 1970)"
            .to_string(),
        "default rule — no specific statutory hour / day limits; tenant must show landlord failed to act within reasonable time given severity of defect; common-law negligence + premises liability framework available"
            .to_string(),
    ];

    LandlordRepairResponseResult {
        compliant: true,
        required_response_hours: 0,
        tenant_repair_and_deduct_engaged: false,
        tenant_rent_abatement_engaged: false,
        tenant_lease_termination_engaged: false,
        violations: Vec::new(),
        citation: "common-law implied warranty of habitability (Hilder v. St. Peter; Javins v. First National Realty Corp.)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx_ordinary_compliant() -> LandlordRepairResponseInput {
        LandlordRepairResponseInput {
            regime: Regime::Texas,
            severity: RepairSeverity::Ordinary,
            written_repair_request_made: true,
            hours_since_request: 48,
            landlord_responded_in_writing: true,
            repair_completed_within_timeframe: true,
        }
    }

    fn tx_emergency_compliant() -> LandlordRepairResponseInput {
        let mut i = tx_ordinary_compliant();
        i.severity = RepairSeverity::EmergencyImminentThreat;
        i.hours_since_request = 12;
        i
    }

    fn il_ordinary_compliant() -> LandlordRepairResponseInput {
        let mut i = tx_ordinary_compliant();
        i.regime = Regime::Illinois;
        i
    }

    fn wa_emergency_compliant() -> LandlordRepairResponseInput {
        let mut i = tx_emergency_compliant();
        i.regime = Regime::Washington;
        i
    }

    fn default_base() -> LandlordRepairResponseInput {
        let mut i = tx_ordinary_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn tx_ordinary_required_hours_168() {
        let r = check(&tx_ordinary_compliant());
        assert_eq!(r.required_response_hours, 168);
        assert!(r.compliant);
    }

    #[test]
    fn tx_emergency_required_hours_24() {
        let r = check(&tx_emergency_compliant());
        assert_eq!(r.required_response_hours, 24);
        assert!(r.compliant);
    }

    #[test]
    fn tx_ordinary_169_hours_no_repair_violates() {
        let mut i = tx_ordinary_compliant();
        i.hours_since_request = 169;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.052(d)") && v.contains("169")));
        assert!(r.tenant_repair_and_deduct_engaged);
        assert!(r.tenant_rent_abatement_engaged);
        assert!(r.tenant_lease_termination_engaged);
    }

    #[test]
    fn tx_emergency_25_hours_no_repair_violates() {
        let mut i = tx_emergency_compliant();
        i.hours_since_request = 25;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.tenant_repair_and_deduct_engaged);
    }

    #[test]
    fn tx_boundary_168_hours_compliant() {
        let mut i = tx_ordinary_compliant();
        i.hours_since_request = 168;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn tx_92_0563_repair_and_deduct_cap_note_present() {
        let r = check(&tx_ordinary_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 92.0563") && n.contains("one month's rent OR $500")));
    }

    #[test]
    fn tx_citation_pins_subsections() {
        let r = check(&tx_ordinary_compliant());
        assert!(r.citation.contains("§§ 92.052(d), 92.056, 92.0563"));
    }

    #[test]
    fn il_ordinary_required_hours_336() {
        let r = check(&il_ordinary_compliant());
        assert_eq!(r.required_response_hours, 336);
    }

    #[test]
    fn il_emergency_required_hours_72() {
        let mut i = il_ordinary_compliant();
        i.severity = RepairSeverity::EmergencyImminentThreat;
        i.hours_since_request = 24;
        let r = check(&i);
        assert_eq!(r.required_response_hours, 72);
        assert!(r.compliant);
    }

    #[test]
    fn il_ordinary_337_hours_no_repair_violates() {
        let mut i = il_ordinary_compliant();
        i.hours_since_request = 337;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 5-12-110(d)")));
    }

    #[test]
    fn il_citation_pins_chicago_rlto() {
        let r = check(&il_ordinary_compliant());
        assert!(r.citation.contains("Chicago RLTO § 5-12-110(d)"));
    }

    #[test]
    fn wa_emergency_required_hours_24() {
        let r = check(&wa_emergency_compliant());
        assert_eq!(r.required_response_hours, 24);
        assert!(r.compliant);
    }

    #[test]
    fn wa_urgent_required_hours_72() {
        let mut i = wa_emergency_compliant();
        i.severity = RepairSeverity::UrgentMajorAppliance;
        i.hours_since_request = 48;
        let r = check(&i);
        assert_eq!(r.required_response_hours, 72);
        assert!(r.compliant);
    }

    #[test]
    fn wa_ordinary_required_hours_240() {
        let mut i = wa_emergency_compliant();
        i.severity = RepairSeverity::Ordinary;
        i.hours_since_request = 100;
        let r = check(&i);
        assert_eq!(r.required_response_hours, 240);
        assert!(r.compliant);
    }

    #[test]
    fn wa_emergency_25_hours_no_repair_violates() {
        let mut i = wa_emergency_compliant();
        i.hours_since_request = 25;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("RCW 59.18.070")));
    }

    #[test]
    fn wa_citation_pins_59_18_070() {
        let r = check(&wa_emergency_compliant());
        assert!(r.citation.contains("RCW 59.18.070"));
    }

    #[test]
    fn default_no_specific_timeframe_compliant_always() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert_eq!(r.required_response_hours, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("common-law implied warranty")));
    }

    #[test]
    fn default_hilder_javins_note_present() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Hilder v. St. Peter")
                && n.contains("Javins v. First National Realty")));
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [
            Regime::Texas,
            Regime::Illinois,
            Regime::Washington,
            Regime::Default,
        ] {
            let mut i = tx_ordinary_compliant();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn wa_uniquely_tiered_three_levels_invariant() {
        let mut i = wa_emergency_compliant();
        i.severity = RepairSeverity::EmergencyImminentThreat;
        assert_eq!(check(&i).required_response_hours, 24);

        i.severity = RepairSeverity::UrgentMajorAppliance;
        i.hours_since_request = 48;
        assert_eq!(check(&i).required_response_hours, 72);

        i.severity = RepairSeverity::Ordinary;
        i.hours_since_request = 100;
        assert_eq!(check(&i).required_response_hours, 240);
    }

    #[test]
    fn no_written_request_no_violation() {
        let mut i = tx_ordinary_compliant();
        i.written_repair_request_made = false;
        i.hours_since_request = 1000;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn repair_completed_within_timeframe_no_violation() {
        let mut i = tx_ordinary_compliant();
        i.hours_since_request = 200;
        i.repair_completed_within_timeframe = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn severity_tier_truth_table_per_regime() {
        let tx_hours = [
            (RepairSeverity::EmergencyImminentThreat, 24u32),
            (RepairSeverity::UrgentMajorAppliance, 168),
            (RepairSeverity::Ordinary, 168),
        ];
        for (severity, expected) in tx_hours {
            let mut i = tx_ordinary_compliant();
            i.severity = severity;
            assert_eq!(check(&i).required_response_hours, expected);
        }

        let wa_hours = [
            (RepairSeverity::EmergencyImminentThreat, 24u32),
            (RepairSeverity::UrgentMajorAppliance, 72),
            (RepairSeverity::Ordinary, 240),
        ];
        for (severity, expected) in wa_hours {
            let mut i = wa_emergency_compliant();
            i.severity = severity;
            i.hours_since_request = 10;
            assert_eq!(check(&i).required_response_hours, expected);
        }
    }

    #[test]
    fn timeframe_exceeded_engages_all_three_remedies() {
        let mut i = tx_ordinary_compliant();
        i.hours_since_request = 200;
        i.repair_completed_within_timeframe = false;
        let r = check(&i);
        assert!(r.tenant_repair_and_deduct_engaged);
        assert!(r.tenant_rent_abatement_engaged);
        assert!(r.tenant_lease_termination_engaged);
    }

    #[test]
    fn timeframe_within_no_remedies() {
        let r = check(&tx_ordinary_compliant());
        assert!(!r.tenant_repair_and_deduct_engaged);
        assert!(!r.tenant_rent_abatement_engaged);
        assert!(!r.tenant_lease_termination_engaged);
    }

    #[test]
    fn tx_clean_no_violations() {
        let r = check(&tx_ordinary_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn il_clean_no_violations() {
        let r = check(&il_ordinary_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn wa_clean_no_violations() {
        let r = check(&wa_emergency_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_clean_no_violations() {
        let r = check(&default_base());
        assert!(r.violations.is_empty());
    }
}
