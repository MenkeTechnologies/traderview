//! Mandatory landlord-paid mid-tenancy rekeying obligations.
//! When tenant requests landlord to rekey locks during ongoing
//! tenancy (vs between-tenancy rekey covered by `lock_change_
//! between_tenancies` and DV-survivor rekey under `dv_survivor_
//! lock_change`), what notice, timing, and cost-allocation
//! requirements attach? Trader-landlord operational concern when
//! tenant loses keys, roommate departs, burglary occurs, or
//! tenant requests security upgrade.
//!
//! Failure to comply with statutory rekeying timeframe exposes
//! landlord to actual damages + punitive damages + civil
//! penalty + one month's rent + court costs + attorney fees
//! (TX framework).
//!
//! **Three regimes**:
//!
//! **Texas — Tex. Prop. Code § 92.156 (Subchapter D Security
//! Devices)**. Most explicit mid-tenancy rekeying statute.
//! Landlord MUST perform additional rekeying or change of
//! security device at tenant's request, at tenant's expense
//! (unlimited number of requests). § 92.156 7-day reasonable
//! window for rekeying after tenant request. Landlord must
//! pay for rekeying for landlord master key changes. Does
//! NOT apply to interior doors. § 92.164 + § 92.165 remedies:
//! actual damages + punitive damages + $500 civil penalty +
//! one month's rent + court costs + attorney fees.
//!
//! **California — Cal. Civ. Code § 1954 + § 1941.3**. Limited
//! mid-tenancy rekeying framework. Entry rules under § 1954
//! apply when landlord enters to rekey. § 1941.3(b) security
//! devices must be maintained in working order at landlord's
//! expense. Common-law reasonable-time obligation for tenant-
//! requested rekey.
//!
//! **Default — common-law quiet enjoyment + reasonable time**.
//! Most states impose reasonable-time obligation on landlord to
//! rekey at tenant's request when reasonable security concern
//! exists. Cost allocation typically follows lease + common-
//! law principle: tenant pays for lost-key rekey, landlord
//! pays for security-upgrade or master-key rekey.
//!
//! Citations: Tex. Prop. Code §§ 92.156, 92.157, 92.158, 92.164,
//! 92.165 (TX Subchapter D Security Devices + tenant rekeying
//! request + 7-day window + remedies); Cal. Civ. Code §§ 1954,
//! 1941.3 (CA entry rules + security device maintenance);
//! common-law quiet enjoyment covenant.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Texas,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RekeyReason {
    /// Tenant lost keys (tenant typically pays).
    KeyLost,
    /// Roommate departed (tenant typically pays).
    RoommateDeparted,
    /// Tenant security concern after suspicious activity.
    TenantSecurityConcern,
    /// Burglary occurred (tenant may pay or landlord depending
    /// on lease).
    BurglaryOccurred,
    /// Landlord master key change (LANDLORD pays under TX §
    /// 92.156).
    LandlordMasterKeyChange,
    /// Landlord security upgrade (landlord pays).
    LandlordSecurityUpgrade,
    /// Interior door lock change (TX § 92.156 EXCLUDED from
    /// statutory framework).
    InteriorDoorLock,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordMidTenancyRekeyingInput {
    pub regime: Regime,
    pub rekey_reason: RekeyReason,
    /// Whether the tenant requested rekeying.
    pub tenant_requested_rekeying: bool,
    /// Hours since tenant request (for 7-day = 168-hour TX
    /// window).
    pub hours_since_tenant_request: u32,
    /// Whether the landlord completed rekeying within statutory
    /// or reasonable window.
    pub rekeying_completed_within_window: bool,
    /// Whether the landlord provided new keys to tenant.
    pub new_keys_provided_to_tenant: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordMidTenancyRekeyingResult {
    pub compliant: bool,
    pub rekeying_right_engaged: bool,
    pub required_window_hours: u32,
    pub tenant_pays_for_rekeying: bool,
    pub landlord_pays_for_rekeying: bool,
    pub tx_remedies_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LandlordMidTenancyRekeyingInput) -> LandlordMidTenancyRekeyingResult {
    match input.regime {
        Regime::Texas => check_texas(input),
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_texas(input: &LandlordMidTenancyRekeyingInput) -> LandlordMidTenancyRekeyingResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Tex. Prop. Code § 92.156 — landlord MUST perform additional rekeying at tenant's request (unlimited requests) within reasonable window (typically 7 days = 168 hours); does NOT apply to closet doors or other interior doors"
            .to_string(),
        "Tex. Prop. Code §§ 92.164 + 92.165 — remedies for noncompliance: actual damages + punitive damages + $500 civil penalty + ONE MONTH's rent + court costs + attorney fees"
            .to_string(),
    ];

    if matches!(input.rekey_reason, RekeyReason::InteriorDoorLock) {
        return LandlordMidTenancyRekeyingResult {
            compliant: true,
            rekeying_right_engaged: false,
            required_window_hours: 0,
            tenant_pays_for_rekeying: false,
            landlord_pays_for_rekeying: false,
            tx_remedies_engaged: false,
            violations,
            citation: "Tex. Prop. Code §§ 92.156, 92.157, 92.158, 92.164, 92.165",
            notes,
        };
    }

    let required_window = 168u32;
    let rekeying_engaged = input.tenant_requested_rekeying;

    let landlord_pays = matches!(
        input.rekey_reason,
        RekeyReason::LandlordMasterKeyChange | RekeyReason::LandlordSecurityUpgrade
    );
    let tenant_pays = !landlord_pays;

    if rekeying_engaged
        && input.hours_since_tenant_request > required_window
        && !input.rekeying_completed_within_window
    {
        violations.push(format!(
            "Tex. Prop. Code § 92.156 — landlord failed to complete rekeying within reasonable 7-day (168-hour) window after tenant request ({} hours elapsed)",
            input.hours_since_tenant_request
        ));
    }

    if rekeying_engaged && !input.new_keys_provided_to_tenant {
        violations.push(
            "Tex. Prop. Code § 92.156 — landlord MUST provide new keys to tenant after rekeying"
                .to_string(),
        );
    }

    let tx_remedies = !violations.is_empty();
    let compliant = violations.is_empty();

    LandlordMidTenancyRekeyingResult {
        compliant,
        rekeying_right_engaged: rekeying_engaged,
        required_window_hours: required_window,
        tenant_pays_for_rekeying: tenant_pays,
        landlord_pays_for_rekeying: landlord_pays,
        tx_remedies_engaged: tx_remedies,
        violations,
        citation: "Tex. Prop. Code §§ 92.156, 92.157, 92.158, 92.164, 92.165",
        notes,
    }
}

fn check_california(input: &LandlordMidTenancyRekeyingInput) -> LandlordMidTenancyRekeyingResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code §§ 1954 + 1941.3 — limited mid-tenancy rekeying framework; entry rules under § 1954 apply when landlord enters to rekey; § 1941.3(b) security devices must be maintained in working order at landlord's expense"
            .to_string(),
        "California — common-law reasonable-time obligation for tenant-requested rekey (no specific statutory window like TX)"
            .to_string(),
    ];

    let required_window = 240u32;
    let rekeying_engaged = input.tenant_requested_rekeying;

    let landlord_pays = matches!(
        input.rekey_reason,
        RekeyReason::LandlordMasterKeyChange
            | RekeyReason::LandlordSecurityUpgrade
            | RekeyReason::BurglaryOccurred
    );
    let tenant_pays = !landlord_pays;

    if rekeying_engaged
        && input.hours_since_tenant_request > required_window
        && !input.rekeying_completed_within_window
    {
        violations.push(format!(
            "Cal. Civ. Code § 1941.3 + common-law reasonable-time — landlord failed to rekey within reasonable time after tenant request ({} hours elapsed)",
            input.hours_since_tenant_request
        ));
    }

    let compliant = violations.is_empty();
    LandlordMidTenancyRekeyingResult {
        compliant,
        rekeying_right_engaged: rekeying_engaged,
        required_window_hours: required_window,
        tenant_pays_for_rekeying: tenant_pays,
        landlord_pays_for_rekeying: landlord_pays,
        tx_remedies_engaged: false,
        violations,
        citation: "Cal. Civ. Code §§ 1954, 1941.3",
        notes,
    }
}

fn check_default(input: &LandlordMidTenancyRekeyingInput) -> LandlordMidTenancyRekeyingResult {
    let notes: Vec<String> = vec![
        "default rule — common-law quiet enjoyment covenant imposes reasonable-time obligation on landlord to rekey at tenant's request when reasonable security concern exists"
            .to_string(),
        "default rule — cost allocation typically follows lease + common-law principle: tenant pays for lost-key rekey, landlord pays for security-upgrade or master-key rekey"
            .to_string(),
    ];

    let landlord_pays = matches!(
        input.rekey_reason,
        RekeyReason::LandlordMasterKeyChange | RekeyReason::LandlordSecurityUpgrade
    );
    let tenant_pays = !landlord_pays;

    LandlordMidTenancyRekeyingResult {
        compliant: true,
        rekeying_right_engaged: input.tenant_requested_rekeying,
        required_window_hours: 0,
        tenant_pays_for_rekeying: tenant_pays,
        landlord_pays_for_rekeying: landlord_pays,
        tx_remedies_engaged: false,
        violations: Vec::new(),
        citation: "common-law quiet enjoyment + lease",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx_compliant_tenant_request() -> LandlordMidTenancyRekeyingInput {
        LandlordMidTenancyRekeyingInput {
            regime: Regime::Texas,
            rekey_reason: RekeyReason::KeyLost,
            tenant_requested_rekeying: true,
            hours_since_tenant_request: 48,
            rekeying_completed_within_window: true,
            new_keys_provided_to_tenant: true,
        }
    }

    fn ca_compliant_tenant_request() -> LandlordMidTenancyRekeyingInput {
        let mut i = tx_compliant_tenant_request();
        i.regime = Regime::California;
        i
    }

    fn default_base() -> LandlordMidTenancyRekeyingInput {
        let mut i = tx_compliant_tenant_request();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn tx_compliant_passes() {
        let r = check(&tx_compliant_tenant_request());
        assert!(r.compliant);
        assert!(r.rekeying_right_engaged);
        assert_eq!(r.required_window_hours, 168);
    }

    #[test]
    fn tx_tenant_pays_for_key_lost() {
        let r = check(&tx_compliant_tenant_request());
        assert!(r.tenant_pays_for_rekeying);
        assert!(!r.landlord_pays_for_rekeying);
    }

    #[test]
    fn tx_landlord_pays_for_master_key_change() {
        let mut i = tx_compliant_tenant_request();
        i.rekey_reason = RekeyReason::LandlordMasterKeyChange;
        let r = check(&i);
        assert!(r.landlord_pays_for_rekeying);
        assert!(!r.tenant_pays_for_rekeying);
    }

    #[test]
    fn tx_landlord_pays_for_security_upgrade() {
        let mut i = tx_compliant_tenant_request();
        i.rekey_reason = RekeyReason::LandlordSecurityUpgrade;
        let r = check(&i);
        assert!(r.landlord_pays_for_rekeying);
    }

    #[test]
    fn tx_past_7_day_window_violates() {
        let mut i = tx_compliant_tenant_request();
        i.hours_since_tenant_request = 200;
        i.rekeying_completed_within_window = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.tx_remedies_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 92.156") && v.contains("7-day") && v.contains("200")));
    }

    #[test]
    fn tx_at_168_boundary_compliant() {
        let mut i = tx_compliant_tenant_request();
        i.hours_since_tenant_request = 168;
        i.rekeying_completed_within_window = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn tx_169_hours_violates() {
        let mut i = tx_compliant_tenant_request();
        i.hours_since_tenant_request = 169;
        i.rekeying_completed_within_window = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn tx_no_new_keys_violates() {
        let mut i = tx_compliant_tenant_request();
        i.new_keys_provided_to_tenant = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("MUST provide new keys")));
    }

    #[test]
    fn tx_interior_door_excluded() {
        let mut i = tx_compliant_tenant_request();
        i.rekey_reason = RekeyReason::InteriorDoorLock;
        i.tenant_requested_rekeying = true;
        i.hours_since_tenant_request = 1000;
        i.rekeying_completed_within_window = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.rekeying_right_engaged);
    }

    #[test]
    fn tx_remedies_note_describes_one_month_rent_and_500() {
        let r = check(&tx_compliant_tenant_request());
        assert!(r.notes.iter().any(|n| n.contains("§§ 92.164 + 92.165")
            && n.contains("$500 civil penalty")
            && n.contains("ONE MONTH")));
    }

    #[test]
    fn tx_citation_pins_subchapter_d_sections() {
        let r = check(&tx_compliant_tenant_request());
        assert!(r
            .citation
            .contains("§§ 92.156, 92.157, 92.158, 92.164, 92.165"));
    }

    #[test]
    fn ca_compliant_within_reasonable_time() {
        let r = check(&ca_compliant_tenant_request());
        assert!(r.compliant);
        assert_eq!(r.required_window_hours, 240);
    }

    #[test]
    fn ca_burglary_landlord_pays() {
        let mut i = ca_compliant_tenant_request();
        i.rekey_reason = RekeyReason::BurglaryOccurred;
        let r = check(&i);
        assert!(r.landlord_pays_for_rekeying);
    }

    #[test]
    fn ca_past_reasonable_time_violates() {
        let mut i = ca_compliant_tenant_request();
        i.hours_since_tenant_request = 300;
        i.rekeying_completed_within_window = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1941.3")));
    }

    #[test]
    fn ca_citation_pins_subsections() {
        let r = check(&ca_compliant_tenant_request());
        assert!(r.citation.contains("§§ 1954, 1941.3"));
    }

    #[test]
    fn default_compliant_always() {
        let r = check(&default_base());
        assert!(r.compliant);
    }

    #[test]
    fn default_tenant_pays_for_key_lost() {
        let r = check(&default_base());
        assert!(r.tenant_pays_for_rekeying);
    }

    #[test]
    fn default_landlord_pays_for_security_upgrade() {
        let mut i = default_base();
        i.rekey_reason = RekeyReason::LandlordSecurityUpgrade;
        let r = check(&i);
        assert!(r.landlord_pays_for_rekeying);
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::Texas, Regime::California, Regime::Default] {
            let mut i = tx_compliant_tenant_request();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn tx_unique_remedies_invariant() {
        let mut i_tx = tx_compliant_tenant_request();
        i_tx.hours_since_tenant_request = 200;
        i_tx.rekeying_completed_within_window = false;
        let r_tx = check(&i_tx);
        assert!(r_tx.tx_remedies_engaged);

        let mut i_ca = ca_compliant_tenant_request();
        i_ca.hours_since_tenant_request = 300;
        i_ca.rekeying_completed_within_window = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.tx_remedies_engaged);
    }

    #[test]
    fn rekey_reason_cost_allocation_truth_table() {
        for reason in [
            RekeyReason::KeyLost,
            RekeyReason::RoommateDeparted,
            RekeyReason::TenantSecurityConcern,
            RekeyReason::BurglaryOccurred,
            RekeyReason::LandlordMasterKeyChange,
            RekeyReason::LandlordSecurityUpgrade,
        ] {
            let mut i = tx_compliant_tenant_request();
            i.rekey_reason = reason;
            let r = check(&i);
            let landlord_pays = matches!(
                reason,
                RekeyReason::LandlordMasterKeyChange | RekeyReason::LandlordSecurityUpgrade
            );
            assert_eq!(r.landlord_pays_for_rekeying, landlord_pays);
            assert_eq!(r.tenant_pays_for_rekeying, !landlord_pays);
        }
    }

    #[test]
    fn no_tenant_request_no_engagement() {
        let mut i = tx_compliant_tenant_request();
        i.tenant_requested_rekeying = false;
        let r = check(&i);
        assert!(!r.rekeying_right_engaged);
    }

    #[test]
    fn tx_clean_no_violations() {
        let r = check(&tx_compliant_tenant_request());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_clean_no_violations() {
        let r = check(&ca_compliant_tenant_request());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_clean_no_violations() {
        let r = check(&default_base());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn tx_uniquely_explicit_7_day_window_invariant() {
        let r_tx = check(&tx_compliant_tenant_request());
        assert_eq!(r_tx.required_window_hours, 168);

        let r_ca = check(&ca_compliant_tenant_request());
        assert_eq!(r_ca.required_window_hours, 240);

        let r_default = check(&default_base());
        assert_eq!(r_default.required_window_hours, 0);
    }
}
