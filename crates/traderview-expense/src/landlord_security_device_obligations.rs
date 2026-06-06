//! Mandatory landlord-provided security devices obligations.
//! When does a residential landlord have an affirmative statutory
//! duty to install and maintain locks, deadbolts, door viewers,
//! and sliding-door security devices? Distinct from siblings
//! `lock_change_between_tenancies` (between-tenancy lock changes),
//! `dv_survivor_lock_change` (DV-survivor lock changes), and
//! `tenant_smart_lock_biometric_consent` (biometric smart lock
//! refusal right). Trader-landlord operational concern in TX, CA,
//! and other states with explicit security-device statutes.
//!
//! Failure to install or maintain required security devices
//! exposes landlord to statutory damages, attorney fees, and
//! potential negligence-per-se claims when tenant suffers
//! break-in or assault.
//!
//! **Three regimes**:
//!
//! **Texas — Tex. Prop. Code § 92.151 et seq. (Subchapter D —
//! Security Devices)**. Most detailed statutory framework in the
//! country. § 92.153(a) — landlord MUST install on each exterior
//! door: (1) keyless bolting device (operable only from inside)
//! and (2) door viewer. § 92.153(b) — keyed deadbolts and
//! doorknob locks NOT required on all exterior doors as long as
//! ONE DOOR has both keyed and keyless deadbolts. § 92.153(c) —
//! for dwellings whose construction was completed on or after
//! September 1, 1993 (or with calendar date January 1, 1995 or
//! later), landlord MUST install on each exterior sliding glass
//! door: sliding door pin lock OR sliding door handle latch OR
//! sliding door security bar. § 92.153(d) — security devices
//! MUST be installed at LANDLORD'S EXPENSE. § 92.153(e) — security
//! devices MUST BE OPERABLE throughout the time a tenant is in
//! possession. § 92.164 tenant remedies + § 92.165 statutory
//! damages.
//!
//! **California — Cal. Civ. Code § 1941.3 (Required Security
//! Devices)**. Landlord MUST install and maintain (a) operable
//! deadbolt locking system on main swinging entry doors, (b)
//! operable window security or locking devices on accessible
//! windows, (c) operable locking mechanisms on swinging garage
//! doors (if applicable). § 1941.3(b) — devices must be installed
//! and maintained in WORKING ORDER throughout tenancy at
//! LANDLORD'S EXPENSE. Failure constitutes breach of implied
//! warranty of habitability under § 1941.1 with attendant
//! tenant remedies (rent withholding, repair-and-deduct, lease
//! termination).
//!
//! **Default — common-law habitability**. Most states apply
//! common-law implied warranty of habitability framework
//! (Hilder v. St. Peter, 144 Vt. 150 (1984); Javins v. First
//! National Realty Corp., 428 F.2d 1071 (D.C. Cir. 1970))
//! requiring landlord to maintain unit in reasonably safe and
//! habitable condition, including functional locks on doors. No
//! statutory enumeration of specific security devices required.
//!
//! Citations: Tex. Prop. Code §§ 92.151, 92.153, 92.156, 92.164,
//! 92.165 (TX Subchapter D Security Devices + tenant remedies +
//! statutory damages); Cal. Civ. Code §§ 1941.1, 1941.3 (CA
//! required security devices + implied warranty of habitability);
//! Hilder v. St. Peter, 144 Vt. 150 (1984); Javins v. First
//! National Realty Corp., 428 F.2d 1071 (D.C. Cir. 1970).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Texas,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordSecurityDeviceInput {
    pub regime: Regime,
    /// Number of exterior doors on the dwelling.
    pub exterior_door_count: u32,
    /// Whether the dwelling has any exterior sliding glass door.
    pub has_sliding_glass_door: bool,
    /// Whether construction completed on or after September 1,
    /// 1993 (TX trigger for sliding-door requirement).
    pub construction_completed_after_1993: bool,
    // Texas-specific requirements:
    /// Whether keyless bolting device installed on each exterior
    /// door (TX § 92.153(a)).
    pub keyless_bolting_each_exterior_door: bool,
    /// Whether door viewer installed on each exterior door
    /// (TX § 92.153(a)).
    pub door_viewer_each_exterior_door: bool,
    /// Whether at least one exterior door has both keyed and
    /// keyless deadbolts (TX § 92.153(b)).
    pub one_door_with_keyed_and_keyless_deadbolts: bool,
    /// Whether sliding door has pin lock OR handle latch OR
    /// security bar (TX § 92.153(c)).
    pub sliding_door_pin_or_latch_or_bar: bool,
    // California-specific requirements:
    /// Whether operable deadbolt installed on main swinging entry
    /// doors (CA § 1941.3(a)(1)).
    pub deadbolt_on_main_entry_doors: bool,
    /// Whether operable window security devices on accessible
    /// windows (CA § 1941.3(a)(2)).
    pub window_security_devices_on_accessible_windows: bool,
    /// Whether garage door locking mechanism (CA § 1941.3(a)(3)
    /// if applicable).
    pub garage_door_locking_mechanism_if_applicable: bool,
    pub has_garage_door: bool,
    // Universal:
    /// Whether devices are operable throughout tenancy
    /// (TX § 92.153(e) + CA § 1941.3(b)).
    pub devices_operable_throughout_tenancy: bool,
    /// Whether devices were installed at landlord's expense
    /// (TX § 92.153(d) + CA § 1941.3(b)).
    pub installed_at_landlord_expense: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordSecurityDeviceResult {
    pub compliant: bool,
    pub habitability_breach_engaged: bool,
    pub negligence_per_se_exposure: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LandlordSecurityDeviceInput) -> LandlordSecurityDeviceResult {
    match input.regime {
        Regime::Texas => check_texas(input),
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_texas(input: &LandlordSecurityDeviceInput) -> LandlordSecurityDeviceResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.keyless_bolting_each_exterior_door {
        violations.push(
            "Tex. Prop. Code § 92.153(a) — landlord MUST install a keyless bolting device (operable only from inside) on EACH exterior door of the dwelling"
                .to_string(),
        );
    }

    if !input.door_viewer_each_exterior_door {
        violations.push(
            "Tex. Prop. Code § 92.153(a) — landlord MUST install a door viewer on EACH exterior door of the dwelling"
                .to_string(),
        );
    }

    if !input.one_door_with_keyed_and_keyless_deadbolts {
        violations.push(
            "Tex. Prop. Code § 92.153(b) — at least ONE exterior door MUST have both keyed and keyless deadbolts (other doors require only keyless deadbolts)"
                .to_string(),
        );
    }

    if input.has_sliding_glass_door
        && input.construction_completed_after_1993
        && !input.sliding_door_pin_or_latch_or_bar
    {
        violations.push(
            "Tex. Prop. Code § 92.153(c) — for dwellings completed on or after September 1, 1993, landlord MUST install on each exterior sliding glass door: sliding door pin lock OR sliding door handle latch OR sliding door security bar"
                .to_string(),
        );
    }

    if !input.installed_at_landlord_expense {
        violations.push(
            "Tex. Prop. Code § 92.153(d) — security devices required by Subsection (a), (b), or (c) MUST be installed at the LANDLORD'S EXPENSE"
                .to_string(),
        );
    }

    if !input.devices_operable_throughout_tenancy {
        violations.push(
            "Tex. Prop. Code § 92.153(e) — security devices required by this section MUST BE OPERABLE throughout the time a tenant is in possession of the dwelling"
                .to_string(),
        );
    }

    notes.push(
        "Tex. Prop. Code §§ 92.164, 92.165 — tenant remedies include statutory damages of one month's rent plus $500 + actual damages + attorney fees + court costs for landlord's failure to install or maintain required security devices"
            .to_string(),
    );
    notes.push(
        "Tex. Prop. Code § 92.151 — Subchapter D Security Devices definitions: 'doorknob lock' (lock in doorknob, operated by key from exterior + without key from interior); 'door viewer' (permanently installed device in exterior door allowing person inside to view person outside)"
            .to_string(),
    );

    if !input.construction_completed_after_1993 && input.has_sliding_glass_door {
        notes.push(
            "Tex. Prop. Code § 92.153(c) — construction completed before September 1, 1993 NOT subject to sliding-door security-device requirement (effective date for new construction trigger)"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    LandlordSecurityDeviceResult {
        compliant,
        habitability_breach_engaged: !compliant,
        negligence_per_se_exposure: !compliant,
        violations,
        citation: "Tex. Prop. Code §§ 92.151, 92.153, 92.156, 92.164, 92.165 (Subchapter D Security Devices)",
        notes,
    }
}

fn check_california(input: &LandlordSecurityDeviceInput) -> LandlordSecurityDeviceResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.deadbolt_on_main_entry_doors {
        violations.push(
            "Cal. Civ. Code § 1941.3(a)(1) — landlord MUST install and maintain operable DEADBOLT locking system on main swinging entry doors"
                .to_string(),
        );
    }

    if !input.window_security_devices_on_accessible_windows {
        violations.push(
            "Cal. Civ. Code § 1941.3(a)(2) — landlord MUST install and maintain operable WINDOW SECURITY OR LOCKING DEVICES on accessible windows"
                .to_string(),
        );
    }

    if input.has_garage_door && !input.garage_door_locking_mechanism_if_applicable {
        violations.push(
            "Cal. Civ. Code § 1941.3(a)(3) — landlord MUST install and maintain operable LOCKING MECHANISMS on swinging garage doors (if applicable)"
                .to_string(),
        );
    }

    if !input.devices_operable_throughout_tenancy || !input.installed_at_landlord_expense {
        violations.push(
            "Cal. Civ. Code § 1941.3(b) — security devices MUST be installed and maintained in WORKING ORDER throughout tenancy at LANDLORD'S EXPENSE"
                .to_string(),
        );
    }

    notes.push(
        "Cal. Civ. Code § 1941.1 — failure to install / maintain required security devices constitutes breach of implied warranty of habitability with attendant tenant remedies (rent withholding, repair-and-deduct, lease termination)"
            .to_string(),
    );
    notes.push(
        "Cal. Civ. Code § 1941.3 — Required Security Devices include deadbolts on main entry doors + window security devices on accessible windows + garage door locking mechanisms (if applicable)"
            .to_string(),
    );

    let compliant = violations.is_empty();
    LandlordSecurityDeviceResult {
        compliant,
        habitability_breach_engaged: !compliant,
        negligence_per_se_exposure: !compliant,
        violations,
        citation: "Cal. Civ. Code §§ 1941.1, 1941.3 (Required Security Devices + Implied Warranty of Habitability)",
        notes,
    }
}

fn check_default(input: &LandlordSecurityDeviceInput) -> LandlordSecurityDeviceResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "default rule — common-law implied warranty of habitability framework (Hilder v. St. Peter, 144 Vt. 150 (1984); Javins v. First National Realty Corp., 428 F.2d 1071 (D.C. Cir. 1970)) requires landlord to maintain unit in reasonably safe and habitable condition INCLUDING functional locks on doors"
            .to_string(),
        "default rule — no statutory enumeration of specific security devices required; common-law negligence + premises liability framework reaches landlord failure to provide reasonably secure entry"
            .to_string(),
    ];

    if !input.devices_operable_throughout_tenancy {
        violations.push(
            "common-law implied warranty of habitability — landlord must maintain functional locks on exterior doors throughout tenancy; failure exposes landlord to negligence-per-se liability for break-in / assault claims"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    LandlordSecurityDeviceResult {
        compliant,
        habitability_breach_engaged: !compliant,
        negligence_per_se_exposure: !compliant,
        violations,
        citation: "Hilder v. St. Peter, 144 Vt. 150 (1984); Javins v. First National Realty Corp., 428 F.2d 1071 (D.C. Cir. 1970); state-specific common-law implied warranty of habitability",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx_compliant() -> LandlordSecurityDeviceInput {
        LandlordSecurityDeviceInput {
            regime: Regime::Texas,
            exterior_door_count: 2,
            has_sliding_glass_door: true,
            construction_completed_after_1993: true,
            keyless_bolting_each_exterior_door: true,
            door_viewer_each_exterior_door: true,
            one_door_with_keyed_and_keyless_deadbolts: true,
            sliding_door_pin_or_latch_or_bar: true,
            deadbolt_on_main_entry_doors: false,
            window_security_devices_on_accessible_windows: false,
            garage_door_locking_mechanism_if_applicable: false,
            has_garage_door: false,
            devices_operable_throughout_tenancy: true,
            installed_at_landlord_expense: true,
        }
    }

    fn ca_compliant() -> LandlordSecurityDeviceInput {
        LandlordSecurityDeviceInput {
            regime: Regime::California,
            exterior_door_count: 2,
            has_sliding_glass_door: false,
            construction_completed_after_1993: false,
            keyless_bolting_each_exterior_door: false,
            door_viewer_each_exterior_door: false,
            one_door_with_keyed_and_keyless_deadbolts: false,
            sliding_door_pin_or_latch_or_bar: false,
            deadbolt_on_main_entry_doors: true,
            window_security_devices_on_accessible_windows: true,
            garage_door_locking_mechanism_if_applicable: true,
            has_garage_door: true,
            devices_operable_throughout_tenancy: true,
            installed_at_landlord_expense: true,
        }
    }

    fn default_compliant() -> LandlordSecurityDeviceInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn tx_full_compliance_passes() {
        let r = check(&tx_compliant());
        assert!(r.compliant);
        assert!(!r.habitability_breach_engaged);
    }

    #[test]
    fn tx_missing_keyless_bolting_violates() {
        let mut i = tx_compliant();
        i.keyless_bolting_each_exterior_door = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.153(a)") && v.contains("keyless bolting device")));
    }

    #[test]
    fn tx_missing_door_viewer_violates() {
        let mut i = tx_compliant();
        i.door_viewer_each_exterior_door = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.153(a)") && v.contains("door viewer")));
    }

    #[test]
    fn tx_missing_keyed_deadbolt_pair_violates() {
        let mut i = tx_compliant();
        i.one_door_with_keyed_and_keyless_deadbolts = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.153(b)") && v.contains("ONE exterior door")));
    }

    #[test]
    fn tx_missing_sliding_door_device_violates_after_1993() {
        let mut i = tx_compliant();
        i.sliding_door_pin_or_latch_or_bar = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.153(c)") && v.contains("sliding door pin lock OR")));
    }

    #[test]
    fn tx_pre_1993_construction_no_sliding_door_requirement() {
        let mut i = tx_compliant();
        i.construction_completed_after_1993 = false;
        i.sliding_door_pin_or_latch_or_bar = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("construction completed before September 1, 1993")));
    }

    #[test]
    fn tx_no_sliding_door_no_requirement() {
        let mut i = tx_compliant();
        i.has_sliding_glass_door = false;
        i.sliding_door_pin_or_latch_or_bar = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn tx_tenant_expense_violates() {
        let mut i = tx_compliant();
        i.installed_at_landlord_expense = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.153(d)") && v.contains("LANDLORD'S EXPENSE")));
    }

    #[test]
    fn tx_non_operable_violates() {
        let mut i = tx_compliant();
        i.devices_operable_throughout_tenancy = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.153(e)") && v.contains("OPERABLE")));
    }

    #[test]
    fn tx_statutory_damages_note_present() {
        let r = check(&tx_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§§ 92.164, 92.165") && n.contains("one month's rent plus $500")));
    }

    #[test]
    fn tx_definitions_note_describes_door_viewer() {
        let r = check(&tx_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 92.151") && n.contains("door viewer")));
    }

    #[test]
    fn tx_citation_pins_subchapter_d_sections() {
        let r = check(&tx_compliant());
        assert!(r
            .citation
            .contains("§§ 92.151, 92.153, 92.156, 92.164, 92.165"));
        assert!(r.citation.contains("Subchapter D"));
    }

    #[test]
    fn ca_full_compliance_passes() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
    }

    #[test]
    fn ca_missing_deadbolt_violates() {
        let mut i = ca_compliant();
        i.deadbolt_on_main_entry_doors = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1941.3(a)(1)") && v.contains("DEADBOLT")));
    }

    #[test]
    fn ca_missing_window_security_violates() {
        let mut i = ca_compliant();
        i.window_security_devices_on_accessible_windows = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1941.3(a)(2)") && v.contains("WINDOW SECURITY")));
    }

    #[test]
    fn ca_missing_garage_lock_violates_when_garage() {
        let mut i = ca_compliant();
        i.garage_door_locking_mechanism_if_applicable = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1941.3(a)(3)") && v.contains("garage doors")));
    }

    #[test]
    fn ca_no_garage_no_garage_lock_required() {
        let mut i = ca_compliant();
        i.has_garage_door = false;
        i.garage_door_locking_mechanism_if_applicable = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_non_operable_violates() {
        let mut i = ca_compliant();
        i.devices_operable_throughout_tenancy = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1941.3(b)") && v.contains("WORKING ORDER")));
    }

    #[test]
    fn ca_implied_warranty_note_present() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1941.1") && n.contains("implied warranty of habitability")));
    }

    #[test]
    fn ca_citation_pins_1941_1_and_1941_3() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 1941.1, 1941.3"));
    }

    #[test]
    fn default_compliant_when_operable() {
        let r = check(&default_compliant());
        assert!(r.compliant);
    }

    #[test]
    fn default_non_operable_violates_common_law() {
        let mut i = default_compliant();
        i.devices_operable_throughout_tenancy = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("common-law") && v.contains("negligence-per-se")));
    }

    #[test]
    fn default_hilder_javins_note_present() {
        let r = check(&default_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Hilder v. St. Peter")
                && n.contains("Javins v. First National Realty")));
    }

    #[test]
    fn default_citation_references_common_law_cases() {
        let r = check(&default_compliant());
        assert!(r.citation.contains("Hilder v. St. Peter"));
        assert!(r.citation.contains("Javins v. First National Realty"));
    }

    #[test]
    fn tx_unique_keyless_bolting_requirement_invariant() {
        let mut i_tx = tx_compliant();
        i_tx.keyless_bolting_each_exterior_door = false;
        let r_tx = check(&i_tx);
        assert!(!r_tx.compliant);

        for regime in [Regime::California, Regime::Default] {
            let mut i = tx_compliant();
            i.regime = regime;
            i.keyless_bolting_each_exterior_door = false;
            i.deadbolt_on_main_entry_doors = true;
            i.window_security_devices_on_accessible_windows = true;
            let r = check(&i);
            let kb_violations: Vec<_> = r
                .violations
                .iter()
                .filter(|v| v.contains("§ 92.153(a)"))
                .collect();
            assert!(
                kb_violations.is_empty(),
                "regime {:?} should not require TX keyless bolting",
                regime
            );
        }
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = tx_compliant();
            i.regime = regime;
            i.deadbolt_on_main_entry_doors = true;
            i.window_security_devices_on_accessible_windows = true;
            i.garage_door_locking_mechanism_if_applicable = true;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn tx_all_violations_simultaneous() {
        let mut i = tx_compliant();
        i.keyless_bolting_each_exterior_door = false;
        i.door_viewer_each_exterior_door = false;
        i.one_door_with_keyed_and_keyless_deadbolts = false;
        i.sliding_door_pin_or_latch_or_bar = false;
        i.installed_at_landlord_expense = false;
        i.devices_operable_throughout_tenancy = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 6);
    }

    #[test]
    fn habitability_breach_engages_with_any_violation() {
        let mut i = tx_compliant();
        i.keyless_bolting_each_exterior_door = false;
        let r = check(&i);
        assert!(r.habitability_breach_engaged);
        assert!(r.negligence_per_se_exposure);
    }

    #[test]
    fn habitability_breach_not_engaged_when_compliant() {
        let r = check(&tx_compliant());
        assert!(!r.habitability_breach_engaged);
        assert!(!r.negligence_per_se_exposure);
    }

    #[test]
    fn tx_clean_compliance_no_violations() {
        let r = check(&tx_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_clean_compliance_no_violations() {
        let r = check(&ca_compliant());
        assert!(r.violations.is_empty());
    }
}
